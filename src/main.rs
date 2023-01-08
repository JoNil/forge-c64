#![no_std]
#![feature(start)]

use core::hint::unreachable_unchecked;
use map::MAP;
use mos_hardware::{
    c64::{self, COLOR_RAM},
    cia::GameController,
    petscii::Petscii,
    vic2::{
        CharsetBank, ControlXFlags, ScreenBank, BLACK, BROWN, GRAY1, LIGHT_GREEN, LIGHT_RED, RED,
        YELLOW,
    },
};
use tileset::TILESET;

mod map;
mod tileset;

const ANIMATION_COUNTER_MASK: u8 = 0x3f;
const RESOURCE_BIT: u8 = 0x10;
const MAP_WIDTH: u8 = 40;
const MAP_HEIGHT: u8 = 25;

fn has_resource(tile: u8) -> bool {
    tile & RESOURCE_BIT > 0
}

fn set_resource(tile: u8) -> u8 {
    tile | RESOURCE_BIT
}

fn clear_resource(tile: u8) -> u8 {
    tile & (!RESOURCE_BIT)
}

fn read_map(x: u8, y: u8) -> u8 {
    unsafe {
        MAP.as_ptr()
            .offset((x as isize) + (y as isize) * (MAP_WIDTH as isize))
            .read()
    }
}

fn write_map(x: u8, y: u8, val: u8) {
    unsafe {
        MAP.as_mut_ptr()
            .offset((x as isize) + (y as isize) * (MAP_WIDTH as isize))
            .write(val);
    }
}

fn write_map_color(x: u8, y: u8, color: u8) {
    unsafe {
        COLOR_RAM
            .offset((x as isize) + (y as isize) * (MAP_WIDTH as isize))
            .write(color);
    }
}

fn is_depositing_down(tile: u8) -> bool {
    let tile = tile & !RESOURCE_BIT;
    tile == 4 || tile == 8 || tile == 12
}

fn is_depositing_up(tile: u8) -> bool {
    let tile = tile & !RESOURCE_BIT;
    tile == 2 || tile == 6 || tile == 10
}

fn is_depositing_right(tile: u8) -> bool {
    let tile = tile & !RESOURCE_BIT;
    tile == 3 || tile == 7 || tile == 11
}

fn is_depositing_left(tile: u8) -> bool {
    let tile = tile & !RESOURCE_BIT;
    tile == 1 || tile == 5 || tile == 9
}

fn clear_screen(screen: &mut [u8; 1000]) {
    for c in screen {
        *c = 0x00;
    }
}

const SCREEN_1: *mut [u8; 1000] = 0x8800 as *mut [u8; 1000];
const SCREEN_2: *mut [u8; 1000] = 0x8C00 as *mut [u8; 1000];
// 0x9000-0x9fff, Free for cpu, vic sees original chars
const CHARSET_1: *mut [u8; 2048] = 0xa000 as *mut [u8; 2048];
const CHARSET_2: *mut [u8; 2048] = 0xa800 as *mut [u8; 2048];
const CHARSET_3: *mut [u8; 2048] = 0xb000 as *mut [u8; 2048];
const CHARSET_4: *mut [u8; 2048] = 0xb800 as *mut [u8; 2048];

static mut ANIMATION_COUNTER: u8 = 0;
static mut DRAW_TO_SCREEN_2: u8 = 0;
static mut NEW_FRAME: u8 = 0;

#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let vic2 = c64::vic2();
    let cia2 = c64::cia2();

    unsafe {
        vic2.border_color.write(BLACK);
        vic2.background_color0.write(BLACK);
        vic2.background_color1.write(GRAY1);
        vic2.background_color2.write(YELLOW);
        vic2.control_x.modify(|v| v | ControlXFlags::MULTICOLOR);

        clear_screen(&mut *SCREEN_1);
        clear_screen(&mut *SCREEN_2);

        // Set VIC2 memory at 0x8000–0xBFFF
        cia2.data_direction_port_a.write(0b11);
        cia2.port_a.write(GameController::from_bits(0b01).unwrap());

        (&mut *CHARSET_1)[0..256].copy_from_slice(&TILESET[8 * (0 * 64)..8 * (32 + 0 * 64)]);
        (&mut *CHARSET_2)[0..256].copy_from_slice(&TILESET[8 * (1 * 64)..8 * (32 + 1 * 64)]);
        (&mut *CHARSET_3)[0..256].copy_from_slice(&TILESET[8 * (2 * 64)..8 * (32 + 2 * 64)]);
        (&mut *CHARSET_4)[0..256].copy_from_slice(&TILESET[8 * (3 * 64)..8 * (32 + 3 * 64)]);

        for i in 0..1000 {
            COLOR_RAM.offset(i).write(LIGHT_RED);
        }

        set_screen_buffer();

        // Clear animation counter
        for i in 0..MAP.len() {
            MAP[i] &= ANIMATION_COUNTER_MASK;
        }

        for x in 0..MAP_WIDTH {
            write_map(x, MAP_HEIGHT - 1, 1);
        }
        for x in 0..MAP_WIDTH {
            write_map(x, MAP_HEIGHT - 2, 1);
        }

        for x in 0..MAP_WIDTH {
            write_map_color(x, MAP_HEIGHT - 1, RED);
        }

        c64::hardware_raster_irq(247);

        loop {
            while (&NEW_FRAME as *const u8).read_volatile() > 0 {}

            let start = (&FRAME_COUNTER as *const u8).read_volatile() as u16;

            {
                // Update map

                for x in 1u8..(MAP_WIDTH - 1) {
                    for y in 1u8..(MAP_HEIGHT - 1) {
                        let tile = read_map(x, y);

                        if !has_resource(tile) {
                            let down = read_map(x, y + 1);
                            let up = read_map(x, y - 1);
                            let left = read_map(x - 1, y);
                            let right = read_map(x + 1, y);

                            if has_resource(down) && is_depositing_up(down) {
                                write_map(x, y, set_resource(tile));
                                write_map(x, y + 1, clear_resource(down));
                            } else if has_resource(up) && is_depositing_down(up) {
                                write_map(x, y, set_resource(tile));
                                write_map(x, y - 1, clear_resource(up));
                            } else if has_resource(left) && is_depositing_right(left) {
                                write_map(x, y, set_resource(tile));
                                write_map(x - 1, y, clear_resource(left));
                            } else if has_resource(right) && is_depositing_left(right) {
                                write_map(x, y, set_resource(tile));
                                write_map(x + 1, y, clear_resource(right));
                            }
                        }
                    }
                }
            }

            {
                // Copy map to screen

                let screen = if DRAW_TO_SCREEN_2 == 1 {
                    &mut *SCREEN_2
                } else {
                    &mut *SCREEN_1
                };

                screen.copy_from_slice(&MAP);

                let mut end = (&FRAME_COUNTER as *const u8).read_volatile() as u16;
                if end < start {
                    end += 255;
                }
                let time = end - start;

                /*let text = format!("{time}");
                for (x, char) in text.chars().enumerate() {
                    write_map(
                        x as u8,
                        MAP_HEIGHT - 1,
                        Petscii::from_char(char).to_screen_code(),
                    );
                }*/
            }

            NEW_FRAME = 1;
        }
    }
}

fn set_screen_buffer() {
    let vic2 = c64::vic2();

    let bank = match unsafe { ANIMATION_COUNTER } {
        0 => CharsetBank::AT_2000.bits(),
        1 => CharsetBank::AT_2800.bits(),
        2 => CharsetBank::AT_3000.bits(),
        3 => CharsetBank::AT_3800.bits(),
        _ => unsafe { unreachable_unchecked() },
    } | match unsafe { DRAW_TO_SCREEN_2 } {
        0 => ScreenBank::AT_0C00.bits(),
        1 => ScreenBank::AT_0800.bits(),
        _ => unsafe { unreachable_unchecked() },
    };

    unsafe { vic2.screen_and_charset_bank.write(bank) };
}

static mut FRAME_COUNTER: u8 = 0;

#[no_mangle]
pub unsafe extern "C" fn called_every_frame() {
    let vic2 = c64::vic2();

    static mut FRAME_COUNT: u8 = 0;
    static mut STATE: u8 = 1;
    static mut NEXT_TEXT_CHARSET: u8 = 0;

    // Always set this as first thing we do. This enables us to to it in time for the first char on the char line
    vic2.screen_and_charset_bank.write(NEXT_TEXT_CHARSET);

    if STATE == 0 {
        vic2.raster_counter.write(247);
        STATE = 1;
    } else {
        vic2.border_color.write(LIGHT_GREEN);

        vic2.raster_counter.write(239);
        STATE = 0;

        if FRAME_COUNT == 5 {
            FRAME_COUNT = 0;

            let animation_counter = ANIMATION_COUNTER + 1;
            ANIMATION_COUNTER = if animation_counter == 4 {
                0
            } else {
                animation_counter
            };

            if animation_counter == 4 {
                // Was the main loop too slow?
                if NEW_FRAME == 0 {
                    loop {
                        vic2.border_color.write(BROWN);
                        vic2.border_color.write(BLACK);
                    }
                }

                NEW_FRAME = 0;

                DRAW_TO_SCREEN_2 = if DRAW_TO_SCREEN_2 == 1 { 0 } else { 1 };
            }
        }

        if DRAW_TO_SCREEN_2 == 0 {
            NEXT_TEXT_CHARSET = CharsetBank::AT_1000.bits() | ScreenBank::AT_0C00.bits();
        } else {
            NEXT_TEXT_CHARSET = CharsetBank::AT_1000.bits() | ScreenBank::AT_0800.bits();
        }

        set_screen_buffer();

        FRAME_COUNT += 1;
        FRAME_COUNTER += 1;

        vic2.border_color.write(BLACK);
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    let vic2 = c64::vic2();
    loop {
        unsafe {
            vic2.border_color.write(RED);
            vic2.border_color.write(BLACK);
        }
    }
}
