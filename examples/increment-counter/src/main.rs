use serde::{Deserialize, Serialize};
use streamdeck::{KeyDownEvent, SingletonAction, StreamDeck, WillAppearEvent};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CounterSettings {
    #[serde(default)]
    count: u32,
}

struct IncrementCounter;

impl SingletonAction for IncrementCounter {
    const UUID: &'static str = "com.example.increment.counter";
    type Settings = CounterSettings;

    async fn on_will_appear(&self, ev: WillAppearEvent<Self::Settings>) {
        let count = ev.payload.settings.count;
        if let Some(key) = ev.action.as_key() {
            let _ = key.set_title(count.to_string()).await;
        }
    }

    async fn on_key_down(&self, ev: KeyDownEvent<Self::Settings>) {
        let count = ev.payload.settings.count.saturating_add(1);
        let _ = ev.action.set_settings(CounterSettings { count }).await;
        let _ = ev.action.set_title(count.to_string()).await;
    }
}

#[tokio::main]
async fn main() -> streamdeck::Result<()> {
    StreamDeck::new()?
        .register_action(IncrementCounter)?
        .connect()
        .await
}
