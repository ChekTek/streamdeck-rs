mod actions;

use streamdeck::StreamDeck;

use crate::actions::IncrementCounter;

fn main() -> streamdeck::Result<()> {
    // 8 MiB worker stacks. Heavy work in handlers (resvg, fonts) should still use
    // `tokio::task::spawn_blocking` so it does not run on the dispatch worker.
    streamdeck::block_on(async {
        StreamDeck::new()?
            .register_action(IncrementCounter)?
            .connect()
            .await
    })
}
