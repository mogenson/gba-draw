#![no_std]
#![feature(start)]
#![forbid(unsafe_code)]
#![feature(exclusive_range_pattern)]
#![feature(bindings_after_at)]

mod gba_display;
use gba_display::{GbaDisplay, PaletteColor};

use core::convert::{Infallible, TryFrom, TryInto};

use embedded_graphics::{
    egtriangle,
    fonts::{Font6x8, Text},
    image::Image,
    pixelcolor::Bgr555,
    prelude::*,
    primitive_style,
    primitives::Rectangle,
    style::{PrimitiveStyle, TextStyle},
};

use gba::{
    debug, fatal,
    io::{
        display::{DisplayControlSetting, DisplayMode, DisplayStatusSetting, DISPCNT, DISPSTAT},
        irq::{set_irq_handler, IrqEnableSetting, IrqFlags, BIOS_IF, IE, IF, IME},
        keypad::read_key_input,
    },
    oam::{write_obj_attributes, OBJAttr0, OBJAttr1, OBJAttr2, ObjectAttributes},
    palram::index_palram_obj_8bpp,
    vram::{bitmap::Mode3, get_8bpp_character_block, Tile8bpp},
    Color,
};

use tinytga::Tga;

const COLORS: [Bgr555; 8] = [
    Bgr555::BLACK,
    Bgr555::RED,
    Bgr555::GREEN,
    Bgr555::BLUE,
    Bgr555::YELLOW,
    Bgr555::MAGENTA,
    Bgr555::CYAN,
    Bgr555::WHITE,
];

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    fatal!("{}", info);
    loop {}
}

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    debug!("Set up display");
    DISPCNT.write(
        DisplayControlSetting::new()
            .with_mode(DisplayMode::Mode3) // bitmap
            .with_bg2(true) // use background
            .with_obj(true) // use sprites
            .with_oam_memory_1d(true) // 1 dimensional vram mapping
            .with_force_vblank(true), // disable display
    );

    debug!("Register palette");
    register_palette();

    debug!("Draw cursor");
    draw_cursor().ok();

    debug!("Create display");
    let mut display = GbaDisplay;
    draw_background(&mut display).ok();
    draw_hud(&mut display).ok();

    debug!("Enable interrupts");
    set_irq_handler(irq_handler);
    DISPSTAT.write(DisplayStatusSetting::new().with_vblank_irq_enable(true));
    IE.write(IrqFlags::new().with_vblank(true));
    IME.write(IrqEnableSetting::IRQ_YES);

    const WIDTH: u32 = Mode3::WIDTH as u32;
    const HEIGHT: u32 = Mode3::HEIGHT as u32;
    let mut point = Point::try_from((WIDTH, HEIGHT)).unwrap() / 2;

    debug!("Start main loop");
    DISPCNT.write(DISPCNT.read().with_force_vblank(false)); // enable display

    let mut color_index = 0;

    loop {
        // sleep until vblank interrupt
        gba::bios::vblank_interrupt_wait();

        // read buttons input
        let input = read_key_input();

        // clear
        if input.start() {
            draw_background(&mut display).ok();
            draw_hud(&mut display).ok();
            continue;
        }

        // cycle cursor
        if input.b() {
            color_index += 1;
            if color_index >= COLORS.len() {
                color_index = 0;
            }
        }

        // adjust game state and wait for vblank
        let offset = Point::new(input.x_tribool() as i32, input.y_tribool() as i32);
        point += offset;

        if let Ok((x @ 0..WIDTH, y @ 0..HEIGHT)) = point.try_into() {
            move_cursor(color_index as u16, x as u16, y as u16);
            if input.a() {
                Pixel(Point::new(x as i32, y as i32), COLORS[color_index])
                    .draw(&mut display)
                    .ok();
            }
        } else {
            point -= offset; // undo
        }

        // wait for button to be released
        while read_key_input().b() {
            gba::bios::vblank_interrupt_wait();
        }
    }
}

extern "C" fn irq_handler(flags: IrqFlags) {
    if flags.vblank() {
        BIOS_IF.write(BIOS_IF.read().with_vblank(true)); // clear vblank flag
        IF.write(IF.read().with_vblank(true));
    }
}

fn draw_background(display: &mut GbaDisplay) -> Result<(), Infallible> {
    let tga = Tga::from_slice(include_bytes!("../assets/amy.tga")).unwrap();
    let image: Image<Tga, Bgr555> = Image::new(&tga, Point::zero());
    image.draw(display)?;
    Ok(())
}

fn draw_hud(display: &mut GbaDisplay) -> Result<(), Infallible> {
    Rectangle::new(Point::new(0, 0), Point::new(72, 24))
        .into_styled(PrimitiveStyle::with_fill(Bgr555::WHITE))
        .draw(display)?;
    Text::new("A: Draw", Point::new(1, 1))
        .into_styled(TextStyle::new(Font6x8, Bgr555::RED))
        .draw(display)?;
    Text::new("B: Color", Point::new(1, 9))
        .into_styled(TextStyle::new(Font6x8, Bgr555::GREEN))
        .draw(display)?;
    Text::new("Start: Clear", Point::new(1, 17))
        .into_styled(TextStyle::new(Font6x8, Bgr555::BLUE))
        .draw(display)?;
    Ok(())
}

fn register_palette() {
    // slot 0 is for transparency
    for (i, color) in COLORS.iter().enumerate() {
        index_palram_obj_8bpp(i as u8 + 1).write(Color(color.into_storage()));
    }
}

fn draw_cursor() -> Result<(), Infallible> {
    let mut tile = Tile8bpp([PaletteColor::TANSPARENT.into_storage().into(); 16]);

    for i in 1..=COLORS.len() {
        let color = PaletteColor::new(i as u8);
        egtriangle!(
            points = [(0, 0), (7, 4), (4, 7)],
            style = primitive_style!(stroke_color = color, fill_color = color, stroke_width = 1)
        )
        .draw(&mut tile)?;

        get_8bpp_character_block(5).index(i).write(tile);
    }

    Ok(())
}

fn move_cursor(index: u16, x: u16, y: u16) {
    write_obj_attributes(
        0,
        ObjectAttributes {
            attr0: OBJAttr0::new().with_row_coordinate(y).with_is_8bpp(true),
            attr1: OBJAttr1::new().with_col_coordinate(x),
            attr2: OBJAttr2::new().with_tile_id(514 + (index * 2)),
        },
    );
}
