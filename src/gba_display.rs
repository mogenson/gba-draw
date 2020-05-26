use core::convert::{Infallible, TryInto};
use embedded_graphics::{
    drawable::Pixel,
    geometry::Size,
    pixelcolor::{raw::RawU8, Bgr555, PixelColor},
    prelude::*,
};
use gba::{
    vram::{bitmap::Mode3, Tile8bpp},
    Color,
};

/// Empty struct representing GBA Display
pub struct GbaDisplay;

impl DrawTarget<Bgr555> for GbaDisplay {
    type Error = Infallible;

    /// Draw a `pixel` that has a color defined as `Bgr555`
    fn draw_pixel(&mut self, pixel: Pixel<Bgr555>) -> Result<(), Self::Error> {
        Mode3::write(
            pixel.0.x as usize,
            pixel.0.y as usize,
            Color(pixel.1.into_storage()),
        );
        Ok(())
    }

    /// Return size of drawable display
    fn size(&self) -> Size {
        Size::new(Mode3::WIDTH as u32, Mode3::HEIGHT as u32)
    }

    /// Clear display with supplied Bgr555 color
    fn clear(&mut self, color: Bgr555) -> Result<(), Self::Error> {
        Mode3::dma_clear_to(Color(color.into_storage()));
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PaletteColor(RawU8);

impl PaletteColor {
    /// Create a new color from palette entry index
    pub const fn new(index: u8) -> Self {
        Self(RawU8::new(index))
    }

    pub const TANSPARENT: Self = Self(RawU8::new(0));
}

impl PixelColor for PaletteColor {
    type Raw = RawU8;
}

impl From<RawU8> for PaletteColor {
    fn from(data: RawU8) -> Self {
        Self(data)
    }
}

impl From<PaletteColor> for RawU8 {
    fn from(value: PaletteColor) -> Self {
        value.0
    }
}

impl DrawTarget<PaletteColor> for Tile8bpp {
    type Error = Infallible;

    /// Draw a `pixel` that has a color defined as `Bgr555`
    fn draw_pixel(&mut self, pixel: Pixel<PaletteColor>) -> Result<(), Self::Error> {
        if let Ok((x @ 0..8, y @ 0..8)) = pixel.0.try_into() {
            let index: u32 = x + (y * 8); // index into [u8; 64] array
            let word: &mut u32 = &mut self.0[index as usize / 4];
            *word &= !(0xFF << ((index % 4) * 8)); // clear byte
            *word |= (pixel.1.into_storage() as u32) << ((index % 4) * 8); // set byte
        }
        Ok(())
    }

    /// Return size of drawable display
    fn size(&self) -> Size {
        Size::new(8, 8)
    }
}
