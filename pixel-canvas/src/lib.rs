//! A pixel surface that drawers write into and presenters show, knowing nothing of what is drawn.
//!
//! - `Canvas` wraps a slice of `0x00RRGGBB` pixels, the form softbuffer and minifb present, and
//!   implements the embedded-graphics draw-target trait, so fonts, shapes, and images from any
//!   crate built on that trait draw into it.
//! - `Scaled` wraps any draw target and draws each pixel as a square, for what should stay
//!   blocky at a higher resolution, a bitmap font for one.
//! - `Ratio` carries a display's scale factor exactly, as a fraction, from the host to whichever
//!   drawer decides how to use it.
//! - Only `embedded-graphics-core` is a dependency: the trait, the colors, and the geometry, with
//!   no fonts and no shapes, which belong to the drawers.

use core::convert::Infallible;
use core::num::NonZeroU32;

use embedded_graphics_core::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics_core::prelude::{DrawTarget, OriginDimensions, Pixel, Point, Size};
use embedded_graphics_core::primitives::Rectangle;

/// Convert a color to the buffer's `0x00RRGGBB` form.
pub fn to_pixel(color: Rgb888) -> u32 {
    (u32::from(color.r()) << 16) | (u32::from(color.g()) << 8) | u32::from(color.b())
}

/// A scale as an exact fraction, `num / den`, kept reduced so equal scales compare equal.
///
/// - Every platform's scale factor is a fraction, Wayland's over 120 and DPI over 96, so a ratio
///   carries it exactly and with no floating point, from the host to the drawer.
/// - A drawer decides what to do with it: a bitmap font rounds it to a whole step, and a drawer
///   that rotates or curves converts it once, at its transform.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ratio {
    /// The numerator.
    num: u32,
    /// The denominator, never 0.
    den: NonZeroU32,
}

impl Ratio {
    /// The scale of 1.
    pub const ONE: Ratio = Ratio {
        num: 1,
        den: NonZeroU32::MIN,
    };

    /// The fraction `num / den`, reduced, or `None` when `den` is 0.
    pub fn new(num: u32, den: u32) -> Option<Ratio> {
        let divisor = gcd(num, den);
        Some(Ratio {
            num: num / divisor,
            den: NonZeroU32::new(den / divisor)?,
        })
    }

    /// The numerator, after reduction.
    pub fn num(self) -> u32 {
        self.num
    }

    /// The denominator, after reduction.
    pub fn den(self) -> u32 {
        self.den.get()
    }

    /// The whole number the ratio equals, or `None` when it has a fractional part.
    pub fn whole(self) -> Option<u32> {
        (self.den() == 1).then_some(self.num)
    }

    /// The nearest whole number, a half rounding up, and at least 1, for whole-step drawers.
    ///
    /// The remainder is compared with what is left of the denominator rather than doubled, so
    /// nothing overflows, and the quotient gains 1 only when the denominator is at least 2, which
    /// leaves it at most half of `u32::MAX`.
    pub fn round(self) -> u32 {
        let den = self.den();
        let (quotient, remainder) = (self.num / den, self.num % den);
        let nearest = if remainder >= den - remainder {
            quotient + 1
        } else {
            quotient
        };
        nearest.max(1)
    }
}

impl From<u32> for Ratio {
    /// The whole number `n` as the ratio `n / 1`.
    fn from(n: u32) -> Ratio {
        Ratio {
            num: n,
            den: NonZeroU32::MIN,
        }
    }
}

/// The greatest common divisor of `a` and `b`, at least 1 so it always divides.
fn gcd(a: u32, b: u32) -> u32 {
    let (mut a, mut b) = (a, b);
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a.max(1)
}

/// A pixel buffer that drawers write into, one `0x00RRGGBB` value per pixel.
///
/// - The buffer is row-major, `width` pixels to a row.
/// - A pixel falling outside the buffer is dropped, so drawing clips at the edges, and a slice
///   shorter than `width * height` clips the rows it lacks.
pub struct Canvas<'a> {
    /// The pixels, `width * height` of them.
    pixels: &'a mut [u32],
    /// The buffer's width in pixels.
    width: u32,
    /// The buffer's height in pixels.
    height: u32,
}

impl<'a> Canvas<'a> {
    /// Wrap a buffer of `width` by `height` pixels.
    pub fn new(pixels: &'a mut [u32], width: u32, height: u32) -> Canvas<'a> {
        Canvas {
            pixels,
            width,
            height,
        }
    }

    /// Set one pixel, dropping it when it is outside the buffer.
    fn set(&mut self, point: Point, value: u32) {
        let (Ok(x), Ok(y)) = (u32::try_from(point.x), u32::try_from(point.y)) else {
            return;
        };
        if x >= self.width || y >= self.height {
            return;
        }
        let index = y as usize * self.width as usize + x as usize;
        if let Some(pixel) = self.pixels.get_mut(index) {
            *pixel = value;
        }
    }
}

impl OriginDimensions for Canvas<'_> {
    /// The buffer's size in pixels.
    fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}

impl DrawTarget for Canvas<'_> {
    type Color = Rgb888;
    type Error = Infallible;

    /// Draw each pixel, dropping any outside the buffer.
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            self.set(point, to_pixel(color));
        }
        Ok(())
    }

    /// Fill the whole buffer with one color, in one pass over the slice.
    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let len = self.width as usize * self.height as usize;
        let value = to_pixel(color);
        for pixel in self.pixels.iter_mut().take(len) {
            *pixel = value;
        }
        Ok(())
    }
}

/// A draw target that draws each pixel as a `scale` by `scale` square on the target it wraps.
///
/// Its size is the wrapped target's size in whole squares, for drawers that lay out by it, and a
/// square crossing the wrapped target's edge is clipped there.
pub struct Scaled<'a, T> {
    /// The target the squares are drawn on.
    target: &'a mut T,
    /// The side of each square, at least 1.
    scale: u32,
}

impl<'a, T> Scaled<'a, T> {
    /// Wrap `target`, drawing at `scale`, raised to 1 when 0.
    pub fn new(target: &'a mut T, scale: u32) -> Scaled<'a, T> {
        Scaled {
            target,
            scale: scale.max(1),
        }
    }
}

impl<T: OriginDimensions> OriginDimensions for Scaled<'_, T> {
    /// The wrapped target's size in whole squares.
    fn size(&self) -> Size {
        self.target.size() / self.scale
    }
}

impl<T: DrawTarget + OriginDimensions> DrawTarget for Scaled<'_, T> {
    type Color = T::Color;
    type Error = T::Error;

    /// Draw each pixel as a square on the wrapped target.
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let side = Size::new(self.scale, self.scale);
        let scale = self.scale as i32;
        for Pixel(point, color) in pixels {
            let square = Rectangle::new(point * scale, side);
            self.target.fill_solid(&square, color)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //! The ratio's arithmetic, and the canvas and the scaled adapter drawn into plain buffers.
    //!
    //! Panicking on a setup failure is the right test behavior, so the crate's `expect_used` lint
    //! is allowed here.
    #![allow(clippy::expect_used)]

    use super::*;

    /// The color a fresh test buffer holds, so an untouched pixel is told from a drawn one.
    const UNTOUCHED: u32 = 0x00ff_00ff;

    /// A color to draw with.
    const RED: Rgb888 = Rgb888::new(0xff, 0x00, 0x00);

    /// A new `width` by `height` buffer holding `UNTOUCHED`.
    fn buffer(width: u32, height: u32) -> Vec<u32> {
        vec![UNTOUCHED; width as usize * height as usize]
    }

    /// A ratio reduces on construction, so equal scales compare equal, and a zero denominator
    /// is refused.
    #[test]
    fn ratio_reduces() {
        let ratio = Ratio::new(150, 120).expect("nonzero denominator");
        assert_eq!((ratio.num(), ratio.den()), (5, 4));
        assert_eq!(Ratio::new(240, 120), Some(Ratio::from(2)));
        assert_eq!(Ratio::new(0, 120), Some(Ratio::from(0)));
        assert_eq!(Ratio::new(1, 0), None);
        assert_eq!(Ratio::ONE, Ratio::from(1));
    }

    /// A ratio is whole only when it has no fractional part.
    #[test]
    fn ratio_whole() {
        assert_eq!(Ratio::from(3).whole(), Some(3));
        assert_eq!(Ratio::new(5, 4).and_then(Ratio::whole), None);
    }

    /// A ratio rounds to the nearest whole number, a half up, and never below 1, even at the
    /// extremes.
    #[test]
    fn ratio_rounds() {
        let cases = [
            ((0, 1), 1),
            ((1, 3), 1),
            ((5, 4), 1),
            ((3, 2), 2),
            ((7, 4), 2),
            ((2, 1), 2),
            ((5, 2), 3),
            ((u32::MAX, 1), u32::MAX),
            ((u32::MAX, 2), u32::MAX / 2 + 1),
            ((u32::MAX, u32::MAX), 1),
        ];
        for ((num, den), expected) in cases {
            let ratio = Ratio::new(num, den).expect("nonzero denominator");
            assert_eq!(ratio.round(), expected, "{num}/{den}");
        }
    }

    /// Colors convert to `0x00RRGGBB`.
    #[test]
    fn converts_colors() {
        assert_eq!(to_pixel(Rgb888::new(0x12, 0x34, 0x56)), 0x0012_3456);
    }

    /// A pixel lands at its row and column and nowhere else.
    #[test]
    fn draws_a_pixel_in_place() {
        let mut pixels = buffer(4, 3);
        let Ok(()) = Canvas::new(&mut pixels, 4, 3).draw_iter([Pixel(Point::new(2, 1), RED)]);
        for (index, &pixel) in pixels.iter().enumerate() {
            let expected = if index == 4 + 2 {
                to_pixel(RED)
            } else {
                UNTOUCHED
            };
            assert_eq!(pixel, expected, "at index {index}");
        }
    }

    /// Pixels outside the buffer, on any side, are dropped.
    #[test]
    fn clips_at_the_edges() {
        let mut pixels = buffer(4, 3);
        let outside = [(-1, 0), (0, -1), (4, 0), (0, 3)].map(|(x, y)| Pixel(Point::new(x, y), RED));
        let Ok(()) = Canvas::new(&mut pixels, 4, 3).draw_iter(outside);
        assert!(pixels.iter().all(|&p| p == UNTOUCHED));
    }

    /// A slice shorter than the size clips the rows it lacks without panicking.
    #[test]
    fn clips_to_a_short_slice() {
        let mut pixels = buffer(4, 2);
        let mut canvas = Canvas::new(&mut pixels, 4, 3);
        let Ok(()) = canvas.draw_iter([Pixel(Point::new(1, 2), RED)]);
        let Ok(()) = canvas.clear(RED);
        assert!(pixels.iter().all(|&p| p == to_pixel(RED)));
    }

    /// A scaled pixel is a square of the scale's side at the scaled position.
    #[test]
    fn scales_a_pixel_to_a_square() {
        let mut pixels = buffer(9, 9);
        let mut canvas = Canvas::new(&mut pixels, 9, 9);
        let Ok(()) = Scaled::new(&mut canvas, 3).draw_iter([Pixel(Point::new(1, 2), RED)]);
        for (index, &pixel) in pixels.iter().enumerate() {
            let (x, y) = (index % 9, index / 9);
            let inside = (3..6).contains(&x) && (6..9).contains(&y);
            let expected = if inside { to_pixel(RED) } else { UNTOUCHED };
            assert_eq!(pixel, expected, "at ({x}, {y})");
        }
    }

    /// The scaled size is the target's size in whole squares.
    #[test]
    fn scaled_size_is_in_whole_squares() {
        let mut pixels = buffer(10, 7);
        let mut canvas = Canvas::new(&mut pixels, 10, 7);
        assert_eq!(Scaled::new(&mut canvas, 3).size(), Size::new(3, 2));
        assert_eq!(Scaled::new(&mut canvas, 0).size(), Size::new(10, 7));
    }
}
