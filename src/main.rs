#![no_std]
#![feature(start)]
#![forbid(unsafe_code)]

mod gba_display;
use gba_display::GbaDisplay;

use core::convert::{TryFrom, TryInto};

use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::Bgr555,
    prelude::*,
    primitives::{Circle, Rectangle, Triangle},
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

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    fatal!("{}", info);
    loop {}
}

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    debug!("Creating EG display");
    let mut display = GbaDisplay::new();
    eg_draw(&mut display).ok();

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
            display.clear(Bgr555::WHITE).ok();
            point = Point::try_from((WIDTH, HEIGHT)).unwrap() / 2;
        }
    }
}

extern "C" fn irq_handler(flags: IrqFlags) {
    if flags.vblank() {
        BIOS_IF.write(BIOS_IF.read().with_vblank(true)); // clear vblank flag
    }
}

fn eg_draw(display: &mut GbaDisplay) -> Result<(), core::convert::Infallible> {
    // Create styles used by the drawing operations.
    let thin_stroke = PrimitiveStyle::with_stroke(Bgr555::CYAN, 1);
    let thick_stroke = PrimitiveStyle::with_stroke(Bgr555::CYAN, 3);
    let fill = PrimitiveStyle::with_fill(Bgr555::CYAN);
    let text_style = TextStyle::new(Font6x8, Bgr555::CYAN);

    let yoffset = 10;
    display.clear(Bgr555::WHITE)?;

    // Draw a 3px wide outline around the display.
    let bottom_right = Point::zero() + display.size() - Point::new(1, 1);
    Rectangle::new(Point::zero(), bottom_right)
        .into_styled(thick_stroke)
        .draw(display)?;

    // Draw a triangle.
    Triangle::new(
        Point::new(16, 16 + yoffset),
        Point::new(16 + 16, 16 + yoffset),
        Point::new(16 + 8, yoffset),
    )
    .into_styled(thin_stroke)
    .draw(display)?;

    // Draw a filled square
    Rectangle::new(Point::new(52, yoffset), Point::new(52 + 16, 16 + yoffset))
        .into_styled(fill)
        .draw(display)?;

    // Draw a circle with a 3px wide stroke.
    let radius = 8;
    Circle::new(Point::new(88 + radius, yoffset + radius), radius as u32)
        .into_styled(thick_stroke)
        .draw(display)?;

    // Draw centered text.
    let text = "embedded-graphics";
    let width = text.len() as i32 * 6;
    Text::new(text, Point::new(64 - width / 2, 40))
        .into_styled(text_style)
        .draw(display)?;

    Ok(())
}
