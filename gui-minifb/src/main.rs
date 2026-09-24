//! Display "Hello, World" in a window until Return is pressed or the window is closed, then exit.
//!
//! The window twin whose loop is the program's own: minifb opens the window, and each pass of the
//! loop draws the frame through the `pixel-canvas` member when the size changes, presents it, and
//! polls the keys.
//!
//! - It is the bare-metal shape: poll, draw, present, repeat, with nothing calling back in.
//! - minifb reports no display scale factor, so the frame is drawn at scale 1 and the platform
//!   enlarges it when the display's scale is above 1, as a Wayland compositor does.

mod renderer;

use std::error::Error;

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use pixel_canvas::{Canvas, Ratio};
use renderer::render;

/// A boxed error, so one type carries minifb's errors out of `main`.
type BoxError = Box<dyn Error>;

/// The window's width in pixels when it opens.
const WIDTH: usize = 800;

/// The window's height in pixels when it opens.
const HEIGHT: usize = 600;

/// The frame's scale, 1 since minifb reports no scale factor to derive one from.
const SCALE: Ratio = Ratio::ONE;

/// How often the loop runs, in passes per second, so an idle window does not spin.
const FPS: usize = 60;

/// Open the window, then loop until Return or a close: redraw on a size change, present, poll.
///
/// - The buffer outlives each pass, so the frame is drawn only when the size changes and is
///   presented again as it is otherwise.
/// - A window with no area, a minimized one for instance, has nothing to draw or present, so the
///   pass only polls.
fn main() -> Result<(), BoxError> {
    let options = WindowOptions {
        resize: true,
        ..WindowOptions::default()
    };
    let mut window = Window::new(env!("CARGO_PKG_NAME"), WIDTH, HEIGHT, options)?;
    window.set_target_fps(FPS);
    let mut pixels = Vec::new();
    let mut drawn = (0, 0);
    while window.is_open() && !return_pressed(&window) {
        let (width, height) = window.get_size();
        let (Ok(canvas_width), Ok(canvas_height)) = (u32::try_from(width), u32::try_from(height))
        else {
            window.update();
            continue;
        };
        if width == 0 || height == 0 {
            window.update();
            continue;
        }
        if drawn != (width, height) {
            pixels.resize(width * height, 0);
            render(
                &mut Canvas::new(&mut pixels, canvas_width, canvas_height),
                SCALE,
            );
            drawn = (width, height);
        }
        window.update_with_buffer(&pixels, width, height)?;
    }
    Ok(())
}

/// Whether Return, on the main keys or the keypad, was pressed since the last poll.
fn return_pressed(window: &Window) -> bool {
    [Key::Enter, Key::NumPadEnter]
        .into_iter()
        .any(|key| window.is_key_pressed(key, KeyRepeat::No))
}
