# streamdeck-rs

Unofficial [Stream Deck](https://www.elgato.com/en/welcome-to-stream-deck) **plugin** SDK and CLI for Rust.

This is a private workspace; the crates are not published to crates.io.

| Crate | Role |
|-------|------|
| [`streamdeck-plugin`](crates/streamdeck-plugin) | Plugin SDK (`use streamdeck::...`) |
| [`streamdeck-plugin-cli`](crates/streamdeck-plugin-cli) | Binary **`streamdeck-plugin`** |

The CLI binary is `streamdeck-plugin`, not `streamdeck`. Elgato’s Node CLI (`@elgato/cli`) already owns `streamdeck` and `sd` on `PATH`.

## Quick start

Point the scaffold at this checkout, then run the CLI from the workspace:

```bash
export STREAMDECK_PLUGIN_PATH=$PWD/crates/streamdeck-plugin
cargo run -p streamdeck-plugin-cli -- create
```

The wizard scaffolds a native `.sdPlugin` (manifest, icons, increment-counter action, property inspector), builds the Rust binary, symlinks it into Stream Deck’s plugins folder, and restarts the plugin.

## Library usage

```rust
use streamdeck::{KeyDownEvent, SingletonAction, StreamDeck};

struct SayHelloAction;

impl SingletonAction for SayHelloAction {
    const UUID: &'static str = "com.elgato.hello-world.say-hello";
    type Settings = serde_json::Value;

    async fn on_key_down(&self, ev: KeyDownEvent<Self::Settings>) {
        let _ = ev.action.set_title("Hello world").await;
    }
}

#[tokio::main]
async fn main() -> streamdeck::Result<()> {
    StreamDeck::new()?
        .register_action(SayHelloAction)?
        .connect()
        .await
}
```

See [`crates/streamdeck-plugin/README.md`](crates/streamdeck-plugin/README.md) for the protocol, namespaces, and `.sdPlugin` layout. See [`crates/streamdeck-plugin-cli/README.md`](crates/streamdeck-plugin-cli/README.md) for CLI commands. [`examples/increment-counter`](examples/increment-counter) is an in-repo plugin used as a smoke test.

## License

MIT
