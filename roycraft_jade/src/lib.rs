use mod_api::{ClientHost, CrosshairTarget, GuiRenderPostEvent};

use mod_api::{
    ClientTickPostEvent, Hooks, KeyConflictContext, KeyRegistration, Registrar, RegistrationContext,
};
use std::cell::Cell;

const MOD: &str = "roycraft_jade";
const TOGGLE: &str = "inspector";
thread_local! {
    static INSPECTOR_VISIBLE: Cell<bool> = const { Cell::new(true) };
}

#[derive(Default)]
struct Jade;

#[mod_api::horizon_mod]
impl Hooks for Jade {
    fn register(&self, r: &mut dyn Registrar, _: &RegistrationContext) {
        r.register_key_mapping(
            KeyRegistration::new(TOGGLE, "key.keyboard.keypad.1")
                .label("key.roycraft_jade.inspector")
                .context(KeyConflictContext::InGame),
        );
    }

    fn client_tick_post(&self, host: &mut dyn ClientHost, _: &mut ClientTickPostEvent) {
        if host.playing() && host.consume_click(MOD, TOGGLE) {
            INSPECTOR_VISIBLE.with(|visible| visible.set(!visible.get()));
        }
    }

    fn gui_render_post(&self, host: &dyn ClientHost, event: &mut GuiRenderPostEvent<'_>) {
        draw(host, event);
    }
}

pub(crate) fn draw(host: &dyn ClientHost, event: &mut GuiRenderPostEvent<'_>) {
    if event.gui_hidden || !host.playing() || !INSPECTOR_VISIBLE.get() {
        return;
    }
    let (id, kind, custom, health) = match host.crosshair_target() {
        Some(CrosshairTarget::Block { state, .. }) => (state.block().id(host), "block", None, None),
        Some(CrosshairTarget::Entity { entity, .. }) => {
            let id = entity
                .kind(host)
                .map_or_else(|| "minecraft:entity".into(), |kind| kind.id(host));
            let health = host
                .living(entity)
                .and_then(|living| Some((living.health(host)?, living.max_health(host)?)));
            (id, "entity", entity.name(host), health)
        }
        _ => return,
    };
    let Some((namespace, path)) = id.split_once(':') else {
        return;
    };
    let key = format!("{kind}.{namespace}.{}", path.replace('/', "."));
    let name = custom
        .or_else(|| host.translate(&key))
        .filter(|name| !name.is_empty() && name != &key)
        .unwrap_or_else(|| path.replace('_', " "));
    let source = if namespace == "minecraft" {
        "Minecraft"
    } else {
        namespace
    };
    let detail = match health {
        Some((current, max)) => format!("{source}  |  HP {current:.0}/{max:.0}"),
        None => source.to_owned(),
    };
    let batch = &mut event.batch;
    let width = batch.text_width(&name).max(batch.text_width(&detail)) + 20.0;
    let x = (batch.size()[0] - width) / 2.0;
    batch.fill([x, 8.0, width, 29.0], 0xEF15121C);
    batch.fill([x, 8.0, width, 1.0], 0xFF9675F0);
    batch.text(&name, [x + 10.0, 13.0], 0xFFF8F5FF, true);
    batch.text(&detail, [x + 10.0, 24.0], 0xFFC2AFE8, false);
}
