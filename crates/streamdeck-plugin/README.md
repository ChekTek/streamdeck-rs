# streamdeck-plugin

Unofficial [Stream Deck](https://www.elgato.com/en/welcome-to-stream-deck) **plugin** SDK for Rust.

This crate is not published to crates.io. Depend on it by path from this workspace. The Rust crate name stays `streamdeck`:

```toml
[dependencies]
streamdeck-plugin = { path = "../streamdeck-plugin" }
```

```rust
use streamdeck::{KeyDownEvent, SingletonAction, StreamDeck};
```

This crate clones the application layer of Elgato’s [`@elgato/streamdeck`](https://github.com/elgatosf/streamdeck/tree/main/packages/plugin) package (3.0). Stream Deck launches your plugin as a long-running process, passes WebSocket registration arguments, and this SDK talks JSON over `ws://127.0.0.1:{port}`.

It does **not** talk HID to the hardware. That is the Stream Deck app’s job. Scaffolding, linking, and packing live in [`streamdeck-plugin-cli`](../streamdeck-plugin-cli) (`cargo run -p streamdeck-plugin-cli`).

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

fn main() -> streamdeck::Result<()> {
    streamdeck::block_on(async {
        StreamDeck::new()?
            .register_action(SayHelloAction)?
            .connect()
            .await
    })
}
```

Register every action **before** `connect()`. `connect()` runs until Stream Deck closes the socket.

`streamdeck::block_on` starts Tokio with 8 MiB worker stacks. Default Tokio stacks are 2 MiB; rasterizing a key image (`resvg`, font shaping) on a worker can SIGSEGV. Prefer `tokio::task::spawn_blocking` for that work anyway. Panics in action callbacks are logged and do not stop the plugin.

After `StreamDeck::new()`, process-wide accessors match the TypeScript `streamDeck.*` namespaces: `streamdeck::logger()`, `streamdeck::settings()`, `streamdeck::devices()`, `streamdeck::system()`, `streamdeck::ui()`, `streamdeck::profiles()`, `streamdeck::i18n()`, `streamdeck::info()`.

`Action` is an enum of `Key`, `Dial`, and `NeoInfobar` (the Stream Deck Neo infobar, controller `"Neo"`). Shared calls (settings, resources, `send_to_property_inspector`) live on `Action`. Controller-specific calls live on the variant: `show_alert` on keys and dials, `set_title` / `set_image` on keys and dials, and `set_feedback` / `set_feedback_layout` on dials and infobars. Use `as_key()`, `as_dial()`, or `as_neo_infobar()` to reach them.

Registration JSON keeps unknown `DeviceType`, `Language`, `Platform`, `Controller`, and `BarSubType` values instead of failing startup. Inbound WebSocket frames keep unknown event names as `PluginEvent::Unknown` (raw JSON) instead of dropping the message. An unknown `Controller` on `willAppear` is logged and still creates the action. `-info` parse errors include a JSON path (`devices[4].type`). The log file is opened from `-pluginUUID` **before** `-info` is parsed, and a successful handshake writes an INFO `connected, listening` line.

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

Logs go to `logs/{pluginUUID}.log` under the plugin folder (UUID is taken from the `.sdPlugin` directory name, then `-pluginUUID`). The logger is created before `-info` is parsed so a bad registration payload is written to that file. Set `STREAMDECK_LOG` or `RUST_LOG` (`trace`, `debug`, `info`, `warn`, `error`) to also print to stderr and to lower the level. `setImage` payloads are redacted at `trace`.

## Settings events

By default, `on_did_receive_settings` and `on_did_receive_global_settings` fire only when settings change in the property inspector. Replies to `get_settings()` / `get_global_settings()` do not fire them, and `get_settings()` reads from a cache filled by `willAppear` and `didReceiveSettings`. This needs Stream Deck 7.1 and `Software.MinimumVersion` ≥ 7.1; otherwise `connect()` returns `ERR_NOT_SUPPORTED`.

To also fire them for `get_settings()` / `get_global_settings()` replies (the pre-3.0 behavior), opt in before `connect()`:

```rust
let plugin = StreamDeck::new()?;
plugin.settings().set_use_legacy_settings_behavior(true)?;
```

This replaces `set_use_experimental_message_identifiers`, with the opposite meaning.

## Feature version gates

Some APIs require a minimum Stream Deck version, matching `@elgato/streamdeck` 3.0:

- Deep links and profile pages: 6.5
- `deviceDidChange`: 7.0
- Resources and the default settings behavior: 7.1

## License

MIT
