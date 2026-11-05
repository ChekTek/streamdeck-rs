use serde::{Deserialize, Serialize};
use streamdeck::{Action, KeyDownEvent, SingletonAction, StreamDeck, WillAppearEvent};

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
        match &ev.action {
            Action::Key(key) => {
                let _ = key.set_title(count.to_string()).await;
            }
            Action::Dial(dial) => {
                let _ = dial.set_title(count.to_string()).await;
            }
            Action::NeoInfobar(_) => {}
        }
    }

    async fn on_key_down(&self, ev: KeyDownEvent<Self::Settings>) {
        let count = ev.payload.settings.count.saturating_add(1);
        let _ = ev.action.set_settings(CounterSettings { count }).await;
        let _ = ev.action.set_title(count.to_string()).await;
    }
}

fn main() -> streamdeck::Result<()> {
    streamdeck::block_on(async {
        StreamDeck::new()?
            .register_action(IncrementCounter)?
            .connect()
            .await
    })
}
