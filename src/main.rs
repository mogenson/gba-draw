#![no_std]
#![feature(start)]
#![forbid(unsafe_code)]

mod gba_display;
use gba_display::GbaDisplay;

use core::convert::{TryFrom, TryInto};

use embedded_graphics::{
    fonts::{Font12x16, Text},
    image::Image,
    pixelcolor::Bgr555,
    prelude::*,
    primitives::{Circle, Rectangle},
    style::{PrimitiveStyle, TextStyle},
};

use gba::{
    debug, fatal,
    io::{
        display::{DisplayStatusSetting, DISPSTAT},
        irq::{set_irq_handler, IrqEnableSetting, IrqFlags, BIOS_IF, IE, IME},
        keypad::read_key_input,
    },
};

use tinytga::Tga;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    fatal!("{}", info);
    loop {}
}

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    debug!("Creating EG display");
    let mut display = GbaDisplay::new();
    draw_tga(&mut display).ok();
    draw_text(&mut display).ok();

    debug!("Enabling interrupts");
    set_irq_handler(irq_handler);
    DISPSTAT.write(DisplayStatusSetting::new().with_vblank_irq_enable(true));
    IE.write(IrqFlags::new().with_vblank(true));
    IME.write(IrqEnableSetting::IRQ_YES);

    const WIDTH: u32 = GbaDisplay::width();
    const HEIGHT: u32 = GbaDisplay::height();
    let mut point = Point::try_from((WIDTH, HEIGHT)).unwrap() / 2;

    debug!("Starting main loop");
    loop {
        // sleep until vblank interrupt
        gba::bios::vblank_interrupt_wait();

        // read buttons input
        let input = read_key_input();

        // adjust game state and wait for vblank
        point += Point::new(input.x_tribool() as i32, input.y_tribool() as i32);

        if let Ok((0..=WIDTH, 0..=HEIGHT)) = point.try_into() {
            Circle::new(point, 1)
                .into_styled(PrimitiveStyle::with_fill(Bgr555::CYAN))
                .draw(&mut display)
                .ok();
        } else {
            draw_tga(&mut display).ok();
            draw_text(&mut display).ok();
            point = Point::try_from((WIDTH, HEIGHT)).unwrap() / 2;
        }
    }
}

extern "C" fn irq_handler(flags: IrqFlags) {
    if flags.vblank() {
        BIOS_IF.write(BIOS_IF.read().with_vblank(true)); // clear vblank flag
    }
}

fn draw_tga(display: &mut GbaDisplay) -> Result<(), core::convert::Infallible> {
    let tga = Tga::from_slice(include_bytes!("../assets/amy.tga")).unwrap();
    let image: Image<Tga, Bgr555> = Image::new(&tga, Point::zero());
    image.draw(display)?;
    Ok(())
}

fn draw_text(display: &mut GbaDisplay) -> Result<(), core::convert::Infallible> {
    Text::new("Dirty Fucking Amy", Point::new(20, 20))
        .into_styled(TextStyle::new(Font12x16, Bgr555::CYAN))
        .draw(display)?;
    Rectangle::new(Point::new(15, 15), Size::new(212, 24))
        .into_styled(PrimitiveStyle::with_stroke(Bgr555::CYAN, 3))
        .draw(display)?;
    Ok(())
}
