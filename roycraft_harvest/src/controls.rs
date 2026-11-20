use mod_api::{
    ClientHost, KeyConflictContext, KeyRegistration, PayloadCodec, PayloadId, Registrar, ToServer,
};
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, OnceLock};
use std::time::{Duration, Instant};

const MOD: &str = "roycraft_harvest";
const HARVEST: &str = "harvest";
const HEARTBEAT: Duration = Duration::from_millis(200);
const MAX_AGE: Duration = Duration::from_secs(1);

#[derive(PayloadCodec)]
struct KeyState {
    down: bool,
}

static CHANNEL: OnceLock<ToServer<KeyState>> = OnceLock::new();
static HELD: LazyLock<Mutex<HashMap<u128, Instant>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

thread_local! {
    static SENT: Cell<Option<Instant>> = const { Cell::new(None) };
}

pub(crate) fn register(r: &mut dyn Registrar) {
    r.register_key_mapping(
        KeyRegistration::new(HARVEST, "key.keyboard.grave.accent")
            .label("key.roycraft_harvest.harvest")
            .context(KeyConflictContext::InGame),
    );
    let channel = r
        .payload_handlers()
        .registrar("1")
        .optional()
        .play_to_server_typed::<KeyState>(
            PayloadId("roycraft_harvest:held".into()),
            |value, cx| {
                let player = cx.player().map_err(|e| format!("{e:?}"))?;
                let now = Instant::now();
                let mut keys = HELD.lock().unwrap();
                keys.retain(|_, when| now.duration_since(*when) < MAX_AGE);
                if value.down {
                    keys.insert(player.uuid(), now);
                } else {
                    keys.remove(&player.uuid());
                }
                Ok(())
            },
        );
    let _ = CHANNEL.set(channel);
}

pub(crate) fn tick(host: &mut dyn ClientHost) {
    let Some(channel) = CHANNEL.get() else { return };
    let now = Instant::now();
    SENT.with(|sent| {
        let previous = sent.get();
        let down = host.playing() && host.is_down(MOD, HARVEST);
        if down && previous.is_none_or(|last| now.duration_since(last) >= HEARTBEAT) {
            let _ = channel.send(host, &KeyState { down });
            sent.set(Some(now));
        } else if !down && previous.is_some() {
            let _ = channel.send(host, &KeyState { down });
            sent.set(None);
        }
    });
}

pub(crate) fn held(player: u128) -> bool {
    HELD.lock()
        .unwrap()
        .get(&player)
        .is_some_and(|when| when.elapsed() < MAX_AGE)
}
