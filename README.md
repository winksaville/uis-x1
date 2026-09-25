# Experiment with UIs

A Cargo workspace of Rust programs toward an actor-model UI, each program named for its stack, and
the libraries they share:

- `tui-rustix`: a TUI on the alternate screen over rustix, installed as `tui-rustix`. It
  displays "Hello, World" until Return/Enter is pressed, then exits back to the terminal's
  original screen.
- `gui-winit-softbuffer`: a window drawn with winit and softbuffer, installed as
  `gui-winit-softbuffer`. It displays "Hello, World" until Return/Enter is pressed or the window
  is closed.
- `gui-minifb`: the same window with minifb, whose loop is the program's own, installed as
  `gui-minifb`. It displays "Hello, World" until Return/Enter is pressed or the window is closed.
- `pixel-canvas`: the library the window programs share, a pixel surface that fonts, shapes, and
  other drawers write into through the embedded-graphics draw-target trait.

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
cargo run -p gui-minifb
```

## Install

```
cargo install --path tui-rustix --locked
cargo install --path gui-winit-softbuffer --locked
cargo install --path gui-minifb --locked
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
