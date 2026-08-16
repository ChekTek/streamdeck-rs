# Increment counter example

Minimal native Stream Deck plugin using `streamdeck-plugin` (`use streamdeck::...`). Each key press increments a per-instance count and updates the key title.

For a generated project with icons, a property inspector, and the same counter action, use `streamdeck-plugin create` from `streamdeck-plugin-cli`.

## Build

From the repository root:

```bash
cargo build -p increment-counter --release
streamdeck-plugin build examples/increment-counter
```

Or copy the binary by hand:

```bash
cp target/release/increment-counter \
  examples/increment-counter/com.example.increment.sdPlugin/bin/increment-counter
chmod +x examples/increment-counter/com.example.increment.sdPlugin/bin/increment-counter
```

On Windows, copy `increment-counter.exe` to `bin/increment-counter.exe`.

## Install

```bash
streamdeck-plugin link examples/increment-counter/com.example.increment.sdPlugin
streamdeck-plugin restart com.example.increment
```

Add the **Counter** action to a key and press it.

## Icons

This example’s `imgs/` paths are placeholders. Replace them with real artwork before shipping, or start from `streamdeck-plugin create`, which includes 1x/2x icons.
