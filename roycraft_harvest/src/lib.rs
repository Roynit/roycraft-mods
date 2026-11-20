use mod_api::{self, GameMode, Hooks, PlayerDestroyedBlockEvent, PlayerRef, ServerHost};
use std::cell::Cell;

mod cluster;
mod controls;
use cluster::{Pos, cluster};

thread_local! {
    static SWEEPING: Cell<bool> = const { Cell::new(false) };
}

struct Sweep;
impl Sweep {
    fn enter() -> Option<Self> {
        SWEEPING.with(|active| (!active.replace(true)).then_some(Self))
    }
}
impl Drop for Sweep {
    fn drop(&mut self) {
        SWEEPING.with(|active| active.set(false));
    }
}

#[derive(Default)]
struct Harvest;

#[derive(Clone, Copy)]
enum HarvestKind {
    Tree,
    Ore { minimum: u8 },
}

fn vanilla_item(host: &dyn ServerHost, player: PlayerRef) -> Option<String> {
    let slot = u32::try_from(player.selected_slot(host)?).ok()?;
    let (_, stack) = player
        .inventory_slots(host)?
        .into_iter()
        .find(|(index, _)| *index == slot)?;
    stack
        .item()
        .id(host)
        .strip_prefix("minecraft:")
        .map(str::to_owned)
}

fn ore_kind(block: &str) -> Option<(&str, u8)> {
    let name = block.strip_prefix("minecraft:")?;
    let family = name.strip_prefix("deepslate_").unwrap_or(name);
    let tier = match family {
        "coal_ore" | "nether_quartz_ore" | "nether_gold_ore" => 0,
        "iron_ore" | "copper_ore" | "lapis_ore" => 1,
        "gold_ore" | "redstone_ore" | "diamond_ore" | "emerald_ore" => 2,
        "ancient_debris" => 3,
        _ => return None,
    };
    Some((family, tier))
}

fn pick_tier(item: &str) -> Option<u8> {
    Some(match item {
        "wooden_pickaxe" | "golden_pickaxe" => 0,
        "stone_pickaxe" => 1,
        "iron_pickaxe" => 2,
        "diamond_pickaxe" | "netherite_pickaxe" => 3,
        _ => return None,
    })
}

impl HarvestKind {
    fn accepts(self, tool: &str) -> bool {
        match self {
            Self::Tree => matches!(
                tool,
                "wooden_axe"
                    | "golden_axe"
                    | "stone_axe"
                    | "iron_axe"
                    | "diamond_axe"
                    | "netherite_axe"
            ),
            Self::Ore { minimum } => pick_tier(tool).is_some_and(|tier| tier >= minimum),
        }
    }
}

#[mod_api::horizon_mod]
impl Hooks for Harvest {
    fn register(&self, r: &mut dyn mod_api::Registrar, _: &mod_api::RegistrationContext) {
        controls::register(r);
    }

    fn client_tick_post(
        &self,
        host: &mut dyn mod_api::ClientHost,
        _: &mut mod_api::ClientTickPostEvent,
    ) {
        controls::tick(host);
    }

    fn player_destroyed_block(
        &self,
        host: &mut dyn ServerHost,
        event: &mut PlayerDestroyedBlockEvent,
    ) {
        let Some(_sweep) = Sweep::enter() else { return };
        let Some(player) = host.players().acting_player() else {
            return;
        };
        if player.game_mode(host) != Some(GameMode::Survival) || !controls::held(player.uuid()) {
            return;
        }
        let Some(level) = player.level(host) else {
            return;
        };
        let origin = event.pos;
        let base = event.state.block();
        let block_id = base.id(host);
        let tree = block_id
            .strip_prefix("minecraft:")
            .is_some_and(|id| id.ends_with("_log") && !id.starts_with("stripped_"));
        let kind = if tree {
            HarvestKind::Tree
        } else if let Some((_, minimum)) = ore_kind(&block_id) {
            HarvestKind::Ore { minimum }
        } else {
            return;
        };
        if !vanilla_item(host, player).is_some_and(|tool| kind.accepts(&tool)) {
            return;
        }
        let same = |pos: Pos| {
            let Some(state) = level.block_state(host, pos) else {
                return false;
            };
            if tree {
                return pos[1] >= origin[1] && state.block() == base;
            }
            let id = state.block().id(host);
            ore_kind(&id)
                .zip(ore_kind(&block_id))
                .is_some_and(|(a, b)| a.0 == b.0)
        };
        let extra = cluster(
            origin,
            if tree { 48 } else { 32 },
            if tree { 16 } else { 8 },
            same,
        );
        for pos in extra {
            if !vanilla_item(host, player).is_some_and(|tool| kind.accepts(&tool)) {
                break;
            }
            // Recheck the block after any other listener ran, and break via the normal player path.
            let matches = level.block_state(host, pos).is_some_and(|state| {
                if tree {
                    state.block() == base
                } else {
                    ore_kind(&state.block().id(host))
                        .zip(ore_kind(&block_id))
                        .is_some_and(|(a, b)| a.0 == b.0)
                }
            });
            if !matches || !player.destroy_block(host, pos, false) {
                break;
            }
        }
    }
}
