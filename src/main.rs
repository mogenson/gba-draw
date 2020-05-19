#![no_std]
#![feature(start)]
#![forbid(unsafe_code)]

use gba::{
    debug, fatal,
    io::{
        display::{DisplayControlSetting, DisplayMode, DisplayStatusSetting, DISPCNT, DISPSTAT},
        irq::{set_irq_handler, IrqEnableSetting, IrqFlags, BIOS_IF, IE, IME},
        keypad::read_key_input,
    },
    vram::bitmap::Mode3,
    Color,
};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    fatal!("{}", info);
    loop {}
}

const WHITE: Color = Color::from_rgb(31, 31, 31);
const RED: Color = Color::from_rgb(31, 0, 0);

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    debug!("starting");

    // setup display
    DISPCNT.write(
        DisplayControlSetting::new()
            .with_mode(DisplayMode::Mode3)
            .with_bg2(true),
    );
    DISPSTAT.write(DisplayStatusSetting::new().with_vblank_irq_enable(true));
    Mode3::dma_clear_to(WHITE);

    let mut px = Mode3::WIDTH / 2;
    let mut py = Mode3::HEIGHT / 2;
    let mut color = RED;

    // enable interrupts
    set_irq_handler(irq_handler);
    IE.write(IrqFlags::new().with_vblank(true));
    IME.write(IrqEnableSetting::IRQ_YES);

    loop {
        gba::bios::vblank_interrupt_wait();

        // read our keys for this frame
        let this_frame_keys = read_key_input();

        // adjust game state and wait for vblank
        px = px.wrapping_add(2 * this_frame_keys.x_tribool() as usize);
        py = py.wrapping_add(2 * this_frame_keys.y_tribool() as usize);
        if this_frame_keys.l() {
            color = Color(color.0.rotate_left(5));
        }
        if this_frame_keys.r() {
            color = Color(color.0.rotate_right(5));
        }

        // draw the new game and wait until the next frame starts.
        if px >= Mode3::WIDTH || py >= Mode3::HEIGHT {
            // out of bounds, reset the screen and position.
            Mode3::dma_clear_to(WHITE);
            px = Mode3::WIDTH / 2;
            py = Mode3::HEIGHT / 2;
        } else {
            // draw the new part of the line
            Mode3::write(px, py, color);
            Mode3::write(px, py + 1, color);
            Mode3::write(px + 1, py, color);
            Mode3::write(px + 1, py + 1, color);
        }
    }
}

extern "C" fn irq_handler(flags: IrqFlags) {
    if flags.vblank() {
        BIOS_IF.write(BIOS_IF.read().with_vblank(true)); // clear vblank flag
    }
}
