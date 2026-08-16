# streamdeck-plugin

Unofficial [Stream Deck](https://www.elgato.com/en/welcome-to-stream-deck) **plugin** SDK for Rust.

The crates.io package is `streamdeck-plugin`; the Rust crate name stays `streamdeck`:

```toml
[dependencies]
streamdeck-plugin = "0.1"
```

```rust
use streamdeck::{KeyDownEvent, SingletonAction, StreamDeck};
```

This crate clones the application layer of Elgato’s [`@elgato/streamdeck`](https://github.com/elgatosf/streamdeck/tree/main/packages/plugin) package. Stream Deck launches your plugin as a long-running process, passes WebSocket registration arguments, and this SDK talks JSON over `ws://127.0.0.1:{port}`.

It does **not** talk HID to the hardware. That is the Stream Deck app’s job. Scaffolding, linking, and packing live in [`streamdeck-plugin-cli`](https://crates.io/crates/streamdeck-plugin-cli) (`cargo install streamdeck-plugin-cli`).

Existing crates named `streamdeck` / `streamdeck-rs` on crates.io are HID drivers or thinner protocol wrappers. This library is the high-level action framework: `SingletonAction`, instance stores, settings, devices, UI, profiles, and version gates.

## Plugin model

```
Stream Deck app  --WebSocket JSON-->  your Rust binary (this SDK)
                 --HID-->             physical Stream Deck
```

At launch the app passes:

| Flag | Meaning |
|------|---------|
| `-port` | Local WebSocket port |
| `-pluginUUID` | Session UUID used when registering |
| `-registerEvent` | Handshake event name (usually `registerPlugin`) |
| `-info` | JSON `RegistrationInfo` |

The SDK connects, sends `{ "event": "<registerEvent>", "uuid": "<pluginUUID>" }`, then routes inbound events to registered actions.

## Usage

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

Register every action **before** `connect()`. `connect()` runs until Stream Deck closes the socket.

After `StreamDeck::new()`, process-wide accessors match the TypeScript `streamDeck.*` namespaces: `streamdeck::logger()`, `streamdeck::settings()`, `streamdeck::devices()`, `streamdeck::system()`, `streamdeck::ui()`, `streamdeck::profiles()`, `streamdeck::i18n()`, `streamdeck::info()`.

## Native `.sdPlugin` layout

A native plugin is a folder Stream Deck loads at startup. Point `CodePathMac` / `CodePathWin` at your binary (not Node):

```
com.example.increment.sdPlugin/
├── manifest.json
├── bin/
│   └── increment-counter          # CodePathMac
│   └── increment-counter.exe      # CodePathWin
├── imgs/
├── logs/                          # written at runtime
└── ui/                            # optional HTML property inspector
```

`manifest.json` should omit the `Nodejs` block. Use a file extension on Windows (`.exe`). The process working directory is the `.sdPlugin` folder.

Create this layout with `streamdeck-plugin create`, or see the repository example `examples/increment-counter`.

Install for development by copying or symlinking the folder to:

- macOS: `~/Library/Application Support/com.elgato.StreamDeck/Plugins/`
- Windows: `%appdata%\Elgato\StreamDeck\Plugins\`

```bash
streamdeck-plugin restart com.example.increment
```

## Logging

Logs go to `logs/{pluginUUID}.log` under the plugin folder (UUID is taken from the `.sdPlugin` directory name). Set `STREAMDECK_LOG` or `RUST_LOG` (`trace`, `debug`, `info`, `warn`, `error`) to also print to stderr and to lower the level.

## Feature version gates

Some APIs require a minimum Stream Deck version (and sometimes `SDKVersion` in the manifest), matching `@elgato/streamdeck` 2.1:

- Deep links and profile pages: 6.5
- Secrets: 6.9 and `SDKVersion` ≥ 3
- `deviceDidChange`: 7.0
- Resources and experimental settings message IDs: 7.1

## License

MIT
