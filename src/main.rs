#![no_std]
#![feature(start)]

use core::{
    hint::unreachable_unchecked,
    sync::atomic::{AtomicU8, Ordering},
};
use mos_hardware::{
    c64::{self, COLOR_RAM},
    cia::GameController,
    vic2::{
        CharsetBank, ControlXFlags, ScreenBank, BLACK, BROWN, GRAY1, LIGHT_GREEN, LIGHT_RED, RED,
        YELLOW,
    },
};

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
    unsafe { *MAP.get_unchecked(((x as u16) + (y as u16) * (MAP_WIDTH as u16)) as usize) }
}

fn write_map(x: u8, y: u8, val: u8) {
    unsafe { *MAP.get_unchecked_mut(((x as u16) + (y as u16) * (MAP_WIDTH as u16)) as usize) = val }
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

static mut ANIMATION_COUNTER: AtomicU8 = AtomicU8::new(0);
static mut DRAW_TO_SCREEN_2: AtomicU8 = AtomicU8::new(0);
static mut NEW_FRAME: AtomicU8 = AtomicU8::new(0);

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

        c64::hardware_raster_irq(251);

        loop {
            while NEW_FRAME.load(Ordering::SeqCst) == 1 {}

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

                let screen = &mut *SCREEN_2;

                screen.copy_from_slice(&MAP);
            }

            NEW_FRAME.store(1, Ordering::SeqCst);
        }
    }
}

fn set_screen_buffer() {
    let vic2 = c64::vic2();

    let bank = match unsafe {
        (
            ANIMATION_COUNTER.load(Ordering::SeqCst),
            DRAW_TO_SCREEN_2.load(Ordering::SeqCst),
        )
    } {
        (0, 0) => CharsetBank::AT_2000.bits() | ScreenBank::AT_0C00.bits(),
        (1, 0) => CharsetBank::AT_2800.bits() | ScreenBank::AT_0C00.bits(),
        (2, 0) => CharsetBank::AT_3000.bits() | ScreenBank::AT_0C00.bits(),
        (3, 0) => CharsetBank::AT_3800.bits() | ScreenBank::AT_0C00.bits(),
        (0, 1) => CharsetBank::AT_2000.bits() | ScreenBank::AT_0C00.bits(),
        (1, 1) => CharsetBank::AT_2800.bits() | ScreenBank::AT_0C00.bits(),
        (2, 1) => CharsetBank::AT_3000.bits() | ScreenBank::AT_0C00.bits(),
        (3, 1) => CharsetBank::AT_3800.bits() | ScreenBank::AT_0C00.bits(),
        _ => unsafe { unreachable_unchecked() },
    };

    unsafe { vic2.screen_and_charset_bank.write(bank) };
}

#[no_mangle]
pub extern "C" fn called_every_frame() {
    let vic2 = c64::vic2();

    static mut FRAME_COUNT: u8 = 0;

    unsafe {
        vic2.border_color.write(LIGHT_GREEN);

        if FRAME_COUNT == 4 {
            FRAME_COUNT = 0;

            let animation_counter = ANIMATION_COUNTER.load(Ordering::SeqCst) + 1;
            ANIMATION_COUNTER.store(
                if animation_counter == 4 {
                    0
                } else {
                    animation_counter
                },
                Ordering::SeqCst,
            );

            if animation_counter == 4 {
                // Was the main loop too slow?
                if NEW_FRAME.load(Ordering::SeqCst) == 0 {
                    loop {
                        vic2.border_color.write(BROWN);
                        vic2.border_color.write(BLACK);
                    }
                }

                NEW_FRAME.store(0, Ordering::SeqCst);

                DRAW_TO_SCREEN_2.store(
                    if DRAW_TO_SCREEN_2.load(Ordering::SeqCst) == 1 {
                        0
                    } else {
                        1
                    },
                    Ordering::SeqCst,
                );
            }

            set_screen_buffer();
        }

        FRAME_COUNT += 1;
        vic2.border_color.write(BLACK);
    }
}

static TILESET: [u8; 2048] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x55, 0x59, 0x59, 0x65, 0x65, 0x59, 0x59, 0x55,
    0x55, 0x55, 0x69, 0x69, 0x96, 0x96, 0x55, 0x55, 0x55, 0x65, 0x65, 0x59, 0x59, 0x65, 0x65, 0x55,
    0x55, 0x55, 0x96, 0x96, 0x69, 0x69, 0x55, 0x55, 0x95, 0x95, 0xa9, 0xa9, 0x55, 0x54, 0x54, 0x50,
    0x55, 0x5a, 0x6a, 0x65, 0x65, 0x25, 0x15, 0x05, 0x05, 0x15, 0x15, 0x55, 0x6a, 0x6a, 0x56, 0x56,
    0x50, 0x54, 0x58, 0x59, 0x59, 0xa9, 0xa5, 0x55, 0x50, 0x54, 0x54, 0x55, 0xa9, 0xa9, 0x95, 0x95,
    0x55, 0xa5, 0xa9, 0x59, 0x59, 0x58, 0x54, 0x50, 0x56, 0x56, 0x6a, 0x6a, 0x55, 0x15, 0x15, 0x05,
    0x05, 0x15, 0x25, 0x65, 0x65, 0x6a, 0x5a, 0x55, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x3c, 0x3c, 0x3c, 0x3c, 0x00, 0x00, 0x55, 0x59, 0x7d, 0x7d, 0x7d, 0x7d, 0x59, 0x55,
    0x55, 0x55, 0x7d, 0x7d, 0xbe, 0xbe, 0x55, 0x55, 0x55, 0x65, 0x7d, 0x7d, 0x7d, 0x7d, 0x65, 0x55,
    0x55, 0x55, 0xbe, 0xbe, 0x7d, 0x7d, 0x55, 0x55, 0x95, 0x95, 0xbd, 0xbd, 0x7d, 0x7c, 0x54, 0x50,
    0x55, 0x5a, 0x7e, 0x7d, 0x7d, 0x3d, 0x15, 0x05, 0x05, 0x15, 0x3d, 0x7d, 0x7e, 0x7e, 0x56, 0x56,
    0x50, 0x54, 0x7c, 0x7d, 0x7d, 0xbd, 0xa5, 0x55, 0x50, 0x54, 0x7c, 0x7d, 0xbd, 0xbd, 0x95, 0x95,
    0x55, 0xa5, 0xbd, 0x7d, 0x7d, 0x7c, 0x54, 0x50, 0x56, 0x56, 0x7e, 0x7e, 0x7d, 0x3d, 0x15, 0x05,
    0x05, 0x15, 0x3d, 0x7d, 0x7d, 0x7e, 0x5a, 0x55, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x55, 0x65, 0x65, 0x95, 0x95, 0x65, 0x65, 0x55,
    0x69, 0x69, 0x96, 0x96, 0x55, 0x55, 0x55, 0x55, 0x55, 0x59, 0x59, 0x56, 0x56, 0x59, 0x59, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x96, 0x96, 0x69, 0x69, 0x55, 0x65, 0x65, 0x95, 0x95, 0x64, 0x64, 0x50,
    0x69, 0x6a, 0x96, 0x95, 0x55, 0x15, 0x15, 0x05, 0x05, 0x19, 0x19, 0x56, 0x56, 0x59, 0x59, 0x55,
    0x50, 0x54, 0x54, 0x55, 0x56, 0x96, 0xa9, 0x69, 0x50, 0x64, 0x64, 0x95, 0x95, 0x65, 0x65, 0x55,
    0x69, 0xa9, 0x96, 0x56, 0x55, 0x54, 0x54, 0x50, 0x55, 0x59, 0x59, 0x56, 0x56, 0x19, 0x19, 0x05,
    0x05, 0x15, 0x15, 0x55, 0x95, 0x96, 0x6a, 0x69, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x3c, 0x3c, 0x3c, 0x3c, 0x00, 0x00, 0x55, 0x65, 0xf5, 0xf5, 0xf5, 0xf5, 0x65, 0x55,
    0x7d, 0x7d, 0xbe, 0xbe, 0x55, 0x55, 0x55, 0x55, 0x55, 0x59, 0x5f, 0x5f, 0x5f, 0x5f, 0x59, 0x55,
    0x55, 0x55, 0x55, 0x55, 0xbe, 0xbe, 0x7d, 0x7d, 0x55, 0x65, 0xf5, 0xf5, 0xf5, 0xf4, 0x64, 0x50,
    0x7d, 0x7e, 0xbe, 0xbd, 0x55, 0x15, 0x15, 0x05, 0x05, 0x19, 0x1f, 0x5f, 0x5f, 0x5f, 0x59, 0x55,
    0x50, 0x54, 0x54, 0x55, 0x7e, 0xbe, 0xbd, 0x7d, 0x50, 0x64, 0xf4, 0xf5, 0xf5, 0xf5, 0x65, 0x55,
    0x7d, 0xbd, 0xbe, 0x7e, 0x55, 0x54, 0x54, 0x50, 0x55, 0x59, 0x5f, 0x5f, 0x5f, 0x1f, 0x19, 0x05,
    0x05, 0x15, 0x15, 0x55, 0xbd, 0xbe, 0x7e, 0x7d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x55, 0x95, 0x95, 0x56, 0x56, 0x95, 0x95, 0x55,
    0x96, 0x96, 0x55, 0x55, 0x55, 0x55, 0x69, 0x69, 0x55, 0x56, 0x56, 0x95, 0x95, 0x56, 0x56, 0x55,
    0x69, 0x69, 0x55, 0x55, 0x55, 0x55, 0x96, 0x96, 0x69, 0xa9, 0x95, 0x55, 0x55, 0x94, 0x94, 0x50,
    0x96, 0x96, 0x55, 0x56, 0x56, 0x15, 0x15, 0x05, 0x05, 0x16, 0x16, 0x55, 0x55, 0x56, 0x6a, 0x69,
    0x50, 0x54, 0x54, 0x95, 0x95, 0x55, 0x96, 0x96, 0x50, 0x94, 0x94, 0x55, 0x55, 0x95, 0xa9, 0x69,
    0x96, 0x96, 0x55, 0x95, 0x95, 0x54, 0x54, 0x50, 0x69, 0x6a, 0x56, 0x55, 0x55, 0x16, 0x16, 0x05,
    0x05, 0x15, 0x15, 0x56, 0x56, 0x55, 0x96, 0x96, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x3c, 0x3c, 0x3c, 0x3c, 0x00, 0x00, 0x55, 0x95, 0xd5, 0xd6, 0xd6, 0xd5, 0x95, 0x55,
    0xbe, 0xbe, 0x55, 0x55, 0x55, 0x55, 0x69, 0x69, 0x55, 0x56, 0x57, 0x97, 0x97, 0x57, 0x56, 0x55,
    0x69, 0x69, 0x55, 0x55, 0x55, 0x55, 0xbe, 0xbe, 0x7d, 0xbd, 0xd5, 0xd5, 0xd5, 0xd4, 0x94, 0x50,
    0xbe, 0xbe, 0x55, 0x55, 0x55, 0x15, 0x15, 0x05, 0x05, 0x16, 0x17, 0x57, 0x57, 0x57, 0x56, 0x55,
    0x50, 0x54, 0x54, 0x55, 0x55, 0x55, 0xbe, 0xbe, 0x50, 0x94, 0xd4, 0xd5, 0xd5, 0xd5, 0xbd, 0x7d,
    0xbe, 0xbe, 0xd5, 0xd5, 0xd5, 0xd4, 0x54, 0x50, 0x7d, 0x7e, 0x57, 0x57, 0x57, 0x17, 0x16, 0x05,
    0x05, 0x15, 0x17, 0x57, 0x57, 0x57, 0xbe, 0xbe, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x55, 0x56, 0x56, 0x59, 0x59, 0x56, 0x56, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x69, 0x69, 0x96, 0x96, 0x55, 0x95, 0x95, 0x65, 0x65, 0x95, 0x95, 0x55,
    0x96, 0x96, 0x69, 0x69, 0x55, 0x55, 0x55, 0x55, 0x96, 0x96, 0x69, 0x69, 0x55, 0x54, 0x54, 0x50,
    0x55, 0x56, 0x56, 0x59, 0x59, 0x16, 0x16, 0x05, 0x05, 0x15, 0x15, 0x55, 0x69, 0x69, 0x96, 0x96,
    0x50, 0x94, 0x94, 0x65, 0x65, 0x95, 0x95, 0x55, 0x50, 0x54, 0x54, 0x55, 0x69, 0x69, 0x96, 0x96,
    0x55, 0x95, 0x95, 0x65, 0x65, 0x94, 0x94, 0x50, 0x96, 0x96, 0x69, 0x69, 0x55, 0x15, 0x15, 0x05,
    0x05, 0x16, 0x16, 0x59, 0x59, 0x56, 0x56, 0x55, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x3c, 0x3c, 0x3c, 0x3c, 0x00, 0x00, 0x55, 0x56, 0x5f, 0x5f, 0x5f, 0x5f, 0x56, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x7d, 0x7d, 0xbe, 0xbe, 0x55, 0x95, 0xf5, 0xf5, 0xf5, 0xf5, 0x95, 0x55,
    0xbe, 0xbe, 0x7d, 0x7d, 0x55, 0x55, 0x55, 0x55, 0xbe, 0xbe, 0x7d, 0x7d, 0x55, 0x54, 0x54, 0x50,
    0x55, 0x56, 0x5f, 0x5f, 0x5f, 0x1f, 0x16, 0x05, 0x05, 0x15, 0x15, 0x55, 0x7d, 0x7d, 0xbe, 0xbe,
    0x50, 0x94, 0xf4, 0xf5, 0xf5, 0xf5, 0x95, 0x55, 0x50, 0x54, 0x54, 0x55, 0x7d, 0x7d, 0xbe, 0xbe,
    0x55, 0x95, 0xf5, 0xf5, 0xf5, 0xf4, 0x94, 0x50, 0xbe, 0xbe, 0x7d, 0x7d, 0x55, 0x15, 0x15, 0x05,
    0x05, 0x16, 0x1f, 0x5f, 0x5f, 0x5f, 0x56, 0x55, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

static mut MAP: [u8; 1000] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x0c, 0x01, 0x01, 0x01, 0x09, 0x00, 0x00, 0x00, 0x07, 0x03, 0x03, 0x03,
    0x03, 0x13, 0x03, 0x03, 0x03, 0x03, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00,
    0x00, 0x07, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x12, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x02, 0x0b, 0x03, 0x03, 0x03, 0x03, 0x03,
    0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x00, 0x00, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00,
    0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x0b, 0x03, 0x03, 0x03, 0x0a, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x06, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
    0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x06, 0x01, 0x01, 0x01, 0x01, 0x01, 0x11, 0x01, 0x01, 0x01, 0x05, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x11,
    0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x51, 0x41, 0x41, 0x41, 0x41, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x81, 0x81, 0x81, 0x81, 0x81, 0x81, 0x81, 0x91,
    0x81, 0x81, 0x81, 0x81, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xc1, 0xc1, 0xc1, 0xc1, 0xc1, 0xc1, 0xd1, 0xc1, 0xc1, 0xc1, 0xc1, 0xc1, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

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
