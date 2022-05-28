#![no_std]
#![feature(start)]

use core::ptr::{read_volatile, write_volatile};

extern "C" {
    fn __chrout(c: u8);
}

const SCREEN_1: *mut [u8; 1000] = 0x0400 as *mut [u8; 1000];
const SCREEN_2: *mut [u8; 1000] = 0x3400 as *mut [u8; 1000];
const CHARSET: *mut [u8; 2048] = 0x3800 as *mut [u8; 2048];

const VIC_Y_SCROLL: *mut u8 = 0xd011 as *mut u8;
const VIC_RASTER_LINE_HIGH_BIT: *mut u8 = 0xd011 as *mut u8;
const VIC_RASTER_LINE: *mut u8 = 0xd012 as *mut u8;
const VIC_X_SCROLL: *mut u8 = 0xd016 as *mut u8;
const VIC_MEMORY_PTRS: *mut u8 = 0xd018 as *mut u8;
const VIC_BORDER_COLOR: *mut u8 = 0xd020 as *mut u8;
const VIC_BGCOLOR: *mut u8 = 0xd021 as *mut u8;
const VIC_MULTI_COLOR_1: *mut u8 = 0xd022 as *mut u8;
const VIC_MULTI_COLOR_2: *mut u8 = 0xd023 as *mut u8;
const VIC_CR2: *mut u8 = 0xd016 as *mut u8;

fn clear_screen(screen: &mut [u8; 1000]) {
    for c in screen {
        *c = 0x20;
    }
}

static mut DRAW_TO_SCREEN_2: u8 = 0;

fn swap_screen_buffer() {
    unsafe {
        if DRAW_TO_SCREEN_2 == 1 {
            write_volatile(
                VIC_MEMORY_PTRS,
                (read_volatile(VIC_MEMORY_PTRS) & 0x0f) | 0xD0,
            );
            DRAW_TO_SCREEN_2 = 0;
        } else {
            write_volatile(
                VIC_MEMORY_PTRS,
                (read_volatile(VIC_MEMORY_PTRS) & 0x0f) | 0x10,
            );
            DRAW_TO_SCREEN_2 = 1;
        }
    }
}

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
    let tile = tile & ANIMATION_COUNTER_MASK & !RESOURCE_BIT;
    tile == 4 || tile == 8 || tile == 12
}

fn is_depositing_up(tile: u8) -> bool {
    let tile = tile & ANIMATION_COUNTER_MASK & !RESOURCE_BIT;
    tile == 2 || tile == 6 || tile == 10
}

fn is_depositing_right(tile: u8) -> bool {
    let tile = tile & ANIMATION_COUNTER_MASK & !RESOURCE_BIT;
    tile == 3 || tile == 7 || tile == 11
}

fn is_depositing_left(tile: u8) -> bool {
    let tile = tile & ANIMATION_COUNTER_MASK & !RESOURCE_BIT;
    tile == 1 || tile == 5 || tile == 9
}

#[start]
pub unsafe fn main(_argc: isize, _argv: *const *const u8) -> isize {
    clear_screen(&mut *SCREEN_1);
    clear_screen(&mut *SCREEN_2);

    write_volatile(VIC_BGCOLOR, 0);
    write_volatile(VIC_BORDER_COLOR, 0);
    write_volatile(VIC_MULTI_COLOR_1, 11);
    write_volatile(VIC_MULTI_COLOR_2, 7);
    write_volatile(VIC_CR2, read_volatile(VIC_CR2) | 0x10);
    write_volatile(VIC_MEMORY_PTRS, read_volatile(VIC_MEMORY_PTRS) | 0x0e);

    (&mut *CHARSET).copy_from_slice(&TILESET);

    // Clear animation counter
    for i in 0..MAP.len() {
        MAP[i] &= ANIMATION_COUNTER_MASK;
    }

    let mut animation_counter: u8 = 0b1000000;

    loop {
        while read_volatile(VIC_RASTER_LINE) != 251 {}

        swap_screen_buffer();

        //write_volatile(VIC_BORDER_COLOR, 5);

        {
            // Update map

            if animation_counter == 0b1100_0000 {
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
        }

        {
            // Copy map to screen

            let screen = if DRAW_TO_SCREEN_2 == 1 {
                &mut *SCREEN_2
            } else {
                &mut *SCREEN_1
            };

            for i in 0..screen.len() {
                screen[i] = MAP[i] | animation_counter;
            }
        }

        animation_counter = animation_counter.wrapping_add(0b1000000);

        //write_volatile(VIC_BORDER_COLOR, 0);
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
