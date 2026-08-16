mod actions;

use streamdeck::StreamDeck;

use crate::actions::IncrementCounter;

#[tokio::main]
async fn main() -> streamdeck::Result<()> {
    StreamDeck::new()?
        .register_action(IncrementCounter)?
        .connect()
        .await
}
