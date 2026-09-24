# Experiment with UIs

A Cargo workspace of Rust programs toward an actor-model UI, each member named for its stack:

- `tui-rustix`: a TUI on the alternate screen over rustix, installed as `tui-rustix`. It
  displays "Hello, World" until Return/Enter is pressed, then exits back to the terminal's
  original screen.
- `gui-winit-softbuffer`: a window drawn with winit and softbuffer, installed as
  `gui-winit-softbuffer`. It displays "Hello, World" until Return/Enter is pressed or the window
  is closed.

## Build

```
cargo build
```

## Test

```
cargo test
```

## Run

```
cargo run -p tui-rustix
cargo run -p gui-winit-softbuffer
```

## Install

```
cargo install --path tui-rustix --locked
cargo install --path gui-winit-softbuffer --locked
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
