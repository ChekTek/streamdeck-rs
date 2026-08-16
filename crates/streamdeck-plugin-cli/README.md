# streamdeck-plugin-cli

CLI for creating and managing **native** Stream Deck plugins written in Rust.

This crate is not published to crates.io. From the workspace root:

```bash
cargo run -p streamdeck-plugin-cli -- create
```

The CLI writes a path dependency on this workspace’s `crates/streamdeck-plugin`. Override that with `STREAMDECK_PLUGIN_PATH` if the CLI binary was built from a different checkout.

The executable is `streamdeck-plugin`. It does not replace Elgato’s Node CLI (`streamdeck` / `sd` from `@elgato/cli`).

## Commands

| Command | Description |
|---------|-------------|
| `create` | Interactive wizard: author, name, UUID, description. Scaffolds a Cargo project and `{uuid}.sdPlugin`, builds `--release`, copies the binary, enables developer mode, links, and restarts. |
| `link [path]` | Symlink (macOS) or junction (Windows) a `.sdPlugin` folder into Stream Deck’s plugins directory. |
| `unlink <uuid>` | Remove that link. `-d/--delete` uninstalls a non-linked plugin. |
| `list [-a]` | Linked plugins, or all installed plugins with `--all`. Also `-l`. |
| `restart\|r <uuid>` | `streamdeck://plugins/restart/<uuid>` |
| `stop\|s <uuid>` | `streamdeck://plugins/stop/<uuid>` |
| `dev [-d]` | Enable or disable Stream Deck developer mode. |
| `build [path]` | `cargo build --release` and copy the binary into `{uuid}.sdPlugin/bin/`. |
| `validate [path]` | Light checks: folder name, manifest JSON, UUID match, current-platform `CodePath`, referenced icons. |
| `pack\|bundle [path]` | Zip the `.sdPlugin` to `{uuid}.streamDeckPlugin`. Honors `.sdignore`. |

Plugin install paths:

- macOS: `~/Library/Application Support/com.elgato.StreamDeck/Plugins`
- Windows: `%APPDATA%/Elgato/StreamDeck/Plugins`

Override the plugins directory with `STREAMDECK_PLUGINS_DIR` (useful in tests).

## Scaffold layout

`streamdeck-plugin create` writes:

```
<dir>/
├── Cargo.toml
├── .gitignore
├── .vscode/
├── src/main.rs
├── src/actions/increment_counter.rs
└── <uuid>.sdPlugin/
    ├── manifest.json
    ├── .sdignore
    ├── imgs/
    ├── ui/increment-counter.html
    └── bin/                 # filled after build
```

The sample action UUID is `<plugin-uuid>.increment`. The manifest uses `CodePathMac` / `CodePathWin` (no `Nodejs` block), `SDKVersion` 3, and `Software.MinimumVersion` `7.1`. `OS` lists only the host platform used during `create`; add the other platform and its binary before packing a dual-OS plugin.

After create, if `cursor` or `code` is on `PATH`, the wizard offers to open the project.

## License

MIT. Placeholder plugin images and the property inspector HTML are adapted from [elgatosf/cli](https://github.com/elgatosf/cli) (MIT).
