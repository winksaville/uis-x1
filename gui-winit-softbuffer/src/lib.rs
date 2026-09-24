//! The "Hello, World" frame for a window: what it shows and how it is drawn into a pixel buffer,
//! shared by the binary and its tests.
//!
//! - The text is drawn with a bitmap font from embedded-graphics, so the drawing needs nothing
//!   but a slice of pixels and runs the same with or without a window.
//! - Each font pixel is drawn as a square of whole screen pixels, so scaled text stays sharp.
//! - The text starts at the top left corner, where the terminal twin's cursor starts.

use std::convert::Infallible;

use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};

/// The text shown.
pub const TEXT: &str = "Hello, World";

/// The background color, in the buffer's `0x00RRGGBB` form.
pub const BACKGROUND: u32 = 0x0000_0000;

/// The text color, in the buffer's `0x00RRGGBB` form.
pub const FOREGROUND: u32 = 0x00e0_e0e0;

/// A pixel buffer that embedded-graphics draws into, each font pixel a `scale` by `scale` square.
///
/// - The buffer is row-major, `width` pixels to a row, one `0x00RRGGBB` value per pixel, the form
///   softbuffer presents.
/// - A pixel falling outside the buffer is dropped, so a window smaller than the text clips it.
pub struct Canvas<'a> {
    /// The pixels, `width * height` of them.
    pixels: &'a mut [u32],
    /// The buffer's width in pixels.
    width: u32,
    /// The buffer's height in pixels.
    height: u32,
    /// The side of the square each font pixel becomes, at least 1.
    scale: u32,
}

impl<'a> Canvas<'a> {
    /// Wrap a buffer of `width` by `height` pixels, drawing at `scale`, raised to 1 when 0.
    ///
    /// The drawing covers only as much of `pixels` as `width * height` reaches, and a buffer
    /// shorter than that clips the rows it lacks.
    pub fn new(pixels: &'a mut [u32], width: u32, height: u32, scale: u32) -> Canvas<'a> {
        Canvas {
            pixels,
            width,
            height,
            scale: scale.max(1),
        }
    }

    /// Set one screen pixel, dropping it when it is outside the buffer.
    fn set(&mut self, x: u32, y: u32, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }
        let index = y as usize * self.width as usize + x as usize;
        if let Some(pixel) = self.pixels.get_mut(index) {
            *pixel = color;
        }
    }
}

impl OriginDimensions for Canvas<'_> {
    /// The buffer's size in font pixels, so embedded-graphics clips at the buffer's edge.
    fn size(&self) -> Size {
        Size::new(self.width / self.scale, self.height / self.scale)
    }
}

impl DrawTarget for Canvas<'_> {
    type Color = Rgb888;
    type Error = Infallible;

    /// Draw each font pixel as a `scale` by `scale` square, dropping any outside the buffer.
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            let (Ok(x), Ok(y)) = (u32::try_from(point.x), u32::try_from(point.y)) else {
                continue;
            };
            let rgb =
                (u32::from(color.r()) << 16) | (u32::from(color.g()) << 8) | u32::from(color.b());
            for dy in 0..self.scale {
                for dx in 0..self.scale {
                    self.set(x * self.scale + dx, y * self.scale + dy, rgb);
                }
            }
        }
        Ok(())
    }
}

/// Draw the frame: fill the buffer with the background, then the text at the top left.
pub fn render(canvas: &mut Canvas<'_>) {
    let len = canvas.width as usize * canvas.height as usize;
    for pixel in canvas.pixels.iter_mut().take(len) {
        *pixel = BACKGROUND;
    }
    let style = MonoTextStyle::new(&FONT_10X20, rgb888(FOREGROUND));
    let Ok(_) = Text::with_baseline(TEXT, Point::zero(), style, Baseline::Top).draw(canvas);
}

/// The scale for a window's scale factor: the factor rounded to a whole number, at least 1.
pub fn scale_for(scale_factor: f64) -> u32 {
    scale_factor.round().max(1.0) as u32
}

/// The text's size in screen pixels at `scale`, one font cell per character.
pub fn text_size(scale: u32) -> (u32, u32) {
    let cell = FONT_10X20.character_size;
    let columns = TEXT.chars().count() as u32;
    (cell.width * columns * scale, cell.height * scale)
}

/// Convert a `0x00RRGGBB` value to an embedded-graphics color.
fn rgb888(value: u32) -> Rgb888 {
    Rgb888::new((value >> 16) as u8, (value >> 8) as u8, value as u8)
}

#[cfg(test)]
mod tests {
    //! The frame drawn into plain buffers, with no window involved.

    use super::*;

    /// Draw the frame into a new `width` by `height` buffer at `scale` and return the buffer.
    fn frame(width: u32, height: u32, scale: u32) -> Vec<u32> {
        let mut pixels = vec![0x00ff_00ff; width as usize * height as usize];
        render(&mut Canvas::new(&mut pixels, width, height, scale));
        pixels
    }

    /// Count the pixels in the text color.
    fn lit(pixels: &[u32]) -> usize {
        pixels.iter().filter(|&&p| p == FOREGROUND).count()
    }

    /// The frame holds only the two colors, with text pixels present, and every text pixel lies
    /// inside the text's box at the top left.
    #[test]
    fn draws_the_text_at_the_top_left() {
        let (width, height) = (200, 60);
        let pixels = frame(width, height, 1);
        let (text_width, text_height) = text_size(1);
        assert!(lit(&pixels) > 0, "no text pixels");
        for (index, &pixel) in pixels.iter().enumerate() {
            assert!(
                pixel == BACKGROUND || pixel == FOREGROUND,
                "stray color {pixel:#08x}"
            );
            let (x, y) = (index as u32 % width, index as u32 / width);
            if pixel == FOREGROUND {
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
