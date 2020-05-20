use core::convert::Infallible;
use embedded_graphics::{drawable::Pixel, geometry::Size, pixelcolor::Bgr555, prelude::*};
use gba::{
    io::display::{DisplayControlSetting, DisplayMode, DISPCNT},
    vram::bitmap::Mode3,
    Color,
};

/// Empty struct representing GBA Display
pub struct GbaDisplay;

impl GbaDisplay {
    /// Creates a Mode3 GBA Display
    pub fn new() -> Self {
        DISPCNT.write(
            DisplayControlSetting::new()
                .with_mode(DisplayMode::Mode3)
                .with_bg2(true),
        );

        GbaDisplay
    }

    pub const fn width() -> u32 {
        Mode3::WIDTH as u32
    }

    pub const fn height() -> u32 {
        Mode3::HEIGHT as u32
    }
}

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
