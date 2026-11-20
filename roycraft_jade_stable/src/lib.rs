use horizon_mod_sdk::{Mod, Registration, game, hud, input};

const TOGGLE: &str = "target-info";

struct Jade {
    visible: bool,
    shown_block: Option<String>,
}

impl Mod for Jade {
    fn init() -> Result<(Self, Registration), String> {
        input::register_key(TOGGLE, "Toggle target info", "key.keyboard.keypad.1")?;
        Ok((
            Self {
                visible: true,
                shown_block: None,
            },
            Registration::new().on_tick(),
        ))
    }

    fn tick(&mut self, _: u64) {
        for event in input::poll() {
            if matches!(event, input::Event::KeyDown(key) if key == TOGGLE) {
                self.visible = !self.visible;
            }
        }

        let block = if self.visible {
            game::raycast(4.5).map(|hit| hit.block)
        } else {
            None
        };
        if block == self.shown_block {
            return;
        }

        if let Some(id) = &block {
            let (source, path) = id.split_once(':').unwrap_or(("minecraft", id));
            let source = if source == "minecraft" {
                "Minecraft"
            } else {
                source
            };
            hud::set_panel(
                hud::Corner::TopLeft,
                &[
                    hud::Line {
                        text: path.replace('_', " "),
                        color: 0xFFFFFF,
                    },
                    hud::Line {
                        text: source.to_owned(),
                        color: 0xC2AFE8,
                    },
                ],
            );
        } else {
            hud::clear();
        }
        self.shown_block = block;
    }
}

horizon_mod_sdk::export_mod!(Jade);
