# difft-web

`difft-web` uses [difftastic](https://github.com/Wilfred/difftastic) (via `difftastic-lib`) to produce structured diffs and render them as HTML.

## Targets

- `x86_64-unknown-linux-gnu` (native test/build)
- `wasm32-wasip1` (WASI renderer used by the static demo)

## Commands

```bash
# Run tests
cargo test

# Regenerate structured demo examples (Rust, Haskell, TypeScript)
cargo run --bin generate_examples

# Build WASI renderer binary
cargo build --release --target wasm32-wasip1 --bin demo_wasi
```

## Static demo

The static site files live in `docs/`.

- `docs/examples.json`: structured diffs generated from difftastic
- `docs/index.html`: demo page
- `docs/app.js`: browser WASI bootstrap
- `docs/styles.css`: demo styling

The demo executes `demo_wasi.wasm` in-browser and injects the rendered diff HTML into the page.
