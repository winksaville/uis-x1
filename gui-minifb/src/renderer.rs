//! The "Hello, World" renderer: draws the frame, the picture the window shows, onto a canvas.
//!
//! - What it shows, the text and its colors, stands in for the core, which will send the
//!   renderer what to draw once it exists.
//! - The text is drawn with a bitmap font from embedded-graphics, scaled in whole steps so each
//!   font pixel stays an even, sharp square. A fractional scale rounds to the nearest whole step
//!   here, a choice for sharpness and simplicity: an antialiased resampler, or an outline font,
//!   could use it exactly at the cost of softer edges.
//! - The text starts at the top left corner, where the terminal twin's cursor starts.

use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};
use pixel_canvas::{Canvas, Ratio, Scaled};

/// The text shown.
pub const TEXT: &str = "Hello, World";

/// The background color.
pub const BACKGROUND: Rgb888 = Rgb888::new(0x00, 0x00, 0x00);

/// The text color.
pub const FOREGROUND: Rgb888 = Rgb888::new(0xe0, 0xe0, 0xe0);

/// Draw the frame: fill the canvas with the background, then the text at the top left at `scale`,
/// rounded to a whole step.
pub fn render(canvas: &mut Canvas<'_>, scale: Ratio) {
    let Ok(()) = canvas.clear(BACKGROUND);
    let style = MonoTextStyle::new(&FONT_10X20, FOREGROUND);
    let text = Text::with_baseline(TEXT, Point::zero(), style, Baseline::Top);
    let Ok(_) = text.draw(&mut Scaled::new(canvas, scale.round()));
}

#[cfg(test)]
mod tests {
    //! The frame drawn into plain buffers, with no window involved.
    //!
    //! Panicking on a setup failure is the right test behavior, so the crate's `expect_used` lint
    //! is allowed here.
    #![allow(clippy::expect_used)]

    use pixel_canvas::to_pixel;

    use super::*;

    /// The text's size in screen pixels at `scale`, rounded as `render` rounds it, one font cell
    /// per character.
    fn text_size(scale: Ratio) -> (u32, u32) {
        let scale = scale.round();
        let cell = FONT_10X20.character_size;
        let columns = TEXT.chars().count() as u32;
        (cell.width * columns * scale, cell.height * scale)
    }

    /// Draw the frame into a new `width` by `height` buffer at `scale` and return the buffer.
    fn frame(width: u32, height: u32, scale: Ratio) -> Vec<u32> {
        let mut pixels = vec![0x00ff_00ff; width as usize * height as usize];
        render(&mut Canvas::new(&mut pixels, width, height), scale);
        pixels
    }

    /// Count the pixels in the text color.
    fn lit(pixels: &[u32]) -> usize {
        pixels
            .iter()
            .filter(|&&p| p == to_pixel(FOREGROUND))
            .count()
    }

    /// The frame holds only the two colors, with text pixels present, and every text pixel lies
    /// inside the text's box at the top left.
    #[test]
    fn draws_the_text_at_the_top_left() {
        let (width, height) = (200, 60);
        let pixels = frame(width, height, Ratio::ONE);
        let (text_width, text_height) = text_size(Ratio::ONE);
        let (background, foreground) = (to_pixel(BACKGROUND), to_pixel(FOREGROUND));
        assert!(lit(&pixels) > 0, "no text pixels");
        for (index, &pixel) in pixels.iter().enumerate() {
            assert!(
                pixel == background || pixel == foreground,
                "stray color {pixel:#08x}"
            );
            let (x, y) = (index as u32 % width, index as u32 / width);
            if pixel == foreground {
                assert!(
                    x < text_width && y < text_height,
                    "text pixel at ({x}, {y})"
                );
            }
        }
    }

    /// At scale 2 each font pixel becomes a 2 by 2 square: four times the text pixels, and each
    /// scaled pixel matches the unscaled pixel it came from.
    #[test]
    fn scales_in_whole_steps() {
        let one = frame(200, 60, Ratio::ONE);
        let two = frame(400, 120, Ratio::from(2));
        assert_eq!(lit(&two), 4 * lit(&one));
        for y in 0..120 {
            for x in 0..400 {
                assert_eq!(
                    two[y * 400 + x],
                    one[(y / 2) * 200 + x / 2],
                    "at ({x}, {y})"
                );
            }
        }
    }

    /// A fractional scale draws as its nearest whole step: 5/4 as 1 and 3/2 as 2.
    #[test]
    fn rounds_a_fractional_scale() {
        let ratio = |num, den| Ratio::new(num, den).expect("nonzero denominator");
        assert_eq!(frame(400, 120, ratio(5, 4)), frame(400, 120, Ratio::ONE));
        assert_eq!(
            frame(400, 120, ratio(3, 2)),
            frame(400, 120, Ratio::from(2))
        );
    }

    /// A buffer smaller than the text holds the top left corner of the unclipped frame.
    #[test]
    fn clips_to_a_small_buffer() {
        let (full_width, full_height) = text_size(Ratio::from(3));
        let full = frame(full_width, full_height, Ratio::from(3));
        let (width, height) = (45, 30);
        let small = frame(width, height, Ratio::from(3));
        assert!(lit(&small) > 0, "no text pixels");
        for y in 0..height as usize {
            for x in 0..width as usize {
                assert_eq!(
                    small[y * width as usize + x],
                    full[y * full_width as usize + x],
                    "at ({x}, {y})"
                );
            }
        }
    }
}
