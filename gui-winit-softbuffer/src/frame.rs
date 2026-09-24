//! The "Hello, World" frame: what the window shows and how it is drawn onto a canvas.
//!
//! - The text is drawn with a bitmap font from embedded-graphics, scaled in whole steps so each
//!   font pixel stays a sharp square.
//! - The text starts at the top left corner, where the terminal twin's cursor starts.

use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};
use pixel_canvas::{Canvas, Scaled};

/// The text shown.
pub const TEXT: &str = "Hello, World";

/// The background color.
pub const BACKGROUND: Rgb888 = Rgb888::new(0x00, 0x00, 0x00);

/// The text color.
pub const FOREGROUND: Rgb888 = Rgb888::new(0xe0, 0xe0, 0xe0);

/// Draw the frame: fill the canvas with the background, then the text at the top left at `scale`.
pub fn render(canvas: &mut Canvas<'_>, scale: u32) {
    let Ok(()) = canvas.clear(BACKGROUND);
    let style = MonoTextStyle::new(&FONT_10X20, FOREGROUND);
    let text = Text::with_baseline(TEXT, Point::zero(), style, Baseline::Top);
    let Ok(_) = text.draw(&mut Scaled::new(canvas, scale));
}

/// The scale for a window's scale factor: the factor rounded to a whole number, at least 1.
pub fn scale_for(scale_factor: f64) -> u32 {
    scale_factor.round().max(1.0) as u32
}

#[cfg(test)]
mod tests {
    //! The frame drawn into plain buffers, with no window involved.

    use pixel_canvas::to_pixel;

    use super::*;

    /// The text's size in screen pixels at `scale`, one font cell per character.
    fn text_size(scale: u32) -> (u32, u32) {
        let cell = FONT_10X20.character_size;
        let columns = TEXT.chars().count() as u32;
        (cell.width * columns * scale, cell.height * scale)
    }

    /// Draw the frame into a new `width` by `height` buffer at `scale` and return the buffer.
    fn frame(width: u32, height: u32, scale: u32) -> Vec<u32> {
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
        let pixels = frame(width, height, 1);
        let (text_width, text_height) = text_size(1);
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
        let one = frame(200, 60, 1);
        let two = frame(400, 120, 2);
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

    /// A buffer smaller than the text holds the top left corner of the unclipped frame.
    #[test]
    fn clips_to_a_small_buffer() {
        let (full_width, full_height) = text_size(3);
        let full = frame(full_width, full_height, 3);
        let (width, height) = (45, 30);
        let small = frame(width, height, 3);
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

    /// The scale factor rounds to a whole number and never drops below 1.
    #[test]
    fn scale_rounds_to_a_whole_number() {
        assert_eq!(scale_for(0.5), 1);
        assert_eq!(scale_for(1.0), 1);
        assert_eq!(scale_for(1.25), 1);
        assert_eq!(scale_for(1.5), 2);
        assert_eq!(scale_for(2.0), 2);
    }
}
