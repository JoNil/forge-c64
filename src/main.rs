#![no_std]
#![no_main]

use map::MAP;
use mos_hardware::{
    c64::{self, COLOR_RAM},
    cia::{CIA2DirectionA, CIA2PortA},
    vic2::{
        CharsetBank, ControlXFlags, ScreenBank, BLACK, BROWN, GRAY1, LIGHT_GREEN, LIGHT_RED, RED,
        YELLOW,
    },
};
use screen::{
    ANIMATION_COUNTER, CHARSET_1, CHARSET_2, CHARSET_3, CHARSET_4, DRAW_TO_SCREEN_2, SCREEN_1,
    SCREEN_2, TEXT_SCREEN_1, TEXT_SCREEN_2,
};
use text_writer::MapTextWriter;
use tileset::TILESET;
use ufmt::uwrite;
use vcell::VolatileCell;

mod map;
mod screen;
mod text_writer;
mod tileset;

const SCRATCH: *mut [u8; 4096] = (0x9000) as *mut [u8; 4096];

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

fn read_map<const Y: isize>(x: u8) -> u8 {
    unsafe {
        MAP.as_ptr()
            .offset(Y * (MAP_WIDTH as isize))
            .offset(x as isize)
            .read()
    }
}

fn read_scratch<const O: isize>(offset: u8) -> u8 {
    unsafe {
        (SCRATCH as *const u8)
            .offset(O)
            .offset(offset as isize)
            .read()
    }
}

fn write_map<const Y: isize>(x: u8, val: u8) {
    unsafe {
        MAP.as_mut_ptr()
            .offset(Y * (MAP_WIDTH as isize))
            .offset(x as isize)
            .write(val);
    }
}

fn write_map_color<const Y: isize>(x: u8, color: u8) {
    unsafe {
        COLOR_RAM
            .offset(Y * (MAP_WIDTH as isize))
            .offset(x as isize)
            .write(color);
    }
}

fn read_map_xy(x: u8, y: u8) -> u8 {
    unsafe {
        MAP.as_ptr()
            .offset((x as isize) + (y as isize) * (MAP_WIDTH as isize))
            .read()
    }
}

fn write_map_xy(x: u8, y: u8, val: u8) {
    unsafe {
        MAP.as_mut_ptr()
            .offset((x as isize) + (y as isize) * (MAP_WIDTH as isize))
            .write(val);
    }
}

/*fn is_dir_down(tile: u8) -> bool {
    tile & 0b1100 > 0
}

fn is_dir_up(tile: u8) -> bool {
    (tile + 2) & 0b1100 > 0
}

fn is_dir_right(tile: u8) -> bool {
    (tile + 1) & 0b1100 > 0
}

fn is_dir_left(tile: u8) -> bool {
    (tile + 3) & 0b1100 > 0
}*/

fn is_dir_down(tile: u8) -> bool {
    let tile = tile & !RESOURCE_BIT;
    tile == 4 || tile == 8 || tile == 12
}

fn is_dir_up(tile: u8) -> bool {
    let tile = tile & !RESOURCE_BIT;
    tile == 2 || tile == 6 || tile == 10
}

fn is_dir_right(tile: u8) -> bool {
    let tile = tile & !RESOURCE_BIT;
    tile == 3 || tile == 7 || tile == 11
}

fn is_dir_left(tile: u8) -> bool {
    let tile = tile & !RESOURCE_BIT;
    tile == 1 || tile == 5 || tile == 9
}

fn came_from_down(tile: u8) -> bool {
    let tile = tile & !RESOURCE_BIT;
    tile == 2 || tile == 6 || tile == 10
}

fn came_from_up(tile: u8) -> bool {
    let tile = tile & !RESOURCE_BIT;
    tile == 4 || tile == 8 || tile == 12
}

fn came_from_right(tile: u8) -> bool {
    let tile = tile & !RESOURCE_BIT;
    tile == 1 || tile == 5 || tile == 9
}

fn came_from_left(tile: u8) -> bool {
    let tile = tile & !RESOURCE_BIT;
    tile == 3 || tile == 7 || tile == 11
}

static mut NEW_FRAME: VolatileCell<u8> = VolatileCell::new(0);
static mut FRAME_COUNTER: VolatileCell<u8> = VolatileCell::new(0);

#[inline(never)]
fn update_map() {
    'outer: for y in 0u8..(MAP_HEIGHT - 1) {
        for x in 0u8..MAP_WIDTH {
            let mut tile = read_map_xy(x, y);

            let tile_no_resource = tile & !RESOURCE_BIT;

            if tile_no_resource > 0 && tile_no_resource < 16 {
                let mut x = x;
                let mut y = y;

                let start_x = x;
                let start_y = y;

                //let mut i = 0;

                loop {
                    write_map_xy(x, y, 16);

                    let (next_x, next_y) = if came_from_down(tile) {
                        (x, y + 1)
                    } else if came_from_up(tile) {
                        (x, y - 1)
                    } else if came_from_left(tile) {
                        (x - 1, y)
                    } else if came_from_right(tile) {
                        (x + 1, y)
                    } else {
                        break 'outer;
                    };

                    let next_tile = read_map_xy(next_x, next_y) & !RESOURCE_BIT;
                    let next_tile_no_resource = next_tile & !RESOURCE_BIT;

                    let current_has_resource = tile != tile_no_resource;
                    let next_has_resource = next_tile != next_tile_no_resource;

                    if !current_has_resource && next_has_resource {
                        //write_map_xy(x, y, set_resource(tile));
                        //write_map_xy(next_x, next_y, clear_resource(next_tile));
                    }

                    if next_x == start_x && next_y == start_y {
                        break 'outer;
                    }

                    if !(next_tile_no_resource > 0 && next_tile_no_resource < 16) {
                        break 'outer;
                    }

                    //i += 1;

                    //if i == 6 {
                    //    break 'outer;
                    //}

                    x = next_x;
                    y = next_y;
                    tile = next_tile;
                }
            }
        }
    }

    /*for x in 1u8..(MAP_WIDTH - 1) {
        for y in 1u8..(MAP_HEIGHT - 1) {
            let tile = read_map_xy(x, y);

            if !has_resource(tile) {
                let down = read_map_xy(x, y + 1);
                let up = read_map_xy(x, y - 1);
                let left = read_map_xy(x - 1, y);
                let right = read_map_xy(x + 1, y);

                if has_resource(down) && is_dir_up(down) {
                    write_map_xy(x, y, set_resource(tile));
                    write_map_xy(x, y + 1, clear_resource(down));
                } else if has_resource(up) && is_dir_down(up) {
                    write_map_xy(x, y, set_resource(tile));
                    write_map_xy(x, y - 1, clear_resource(up));
                } else if has_resource(left) && is_dir_right(left) {
                    write_map_xy(x, y, set_resource(tile));
                    write_map_xy(x - 1, y, clear_resource(left));
                } else if has_resource(right) && is_dir_left(right) {
                    write_map_xy(x, y, set_resource(tile));
                    write_map_xy(x + 1, y, clear_resource(right));
                }
            }
        }
    }*/
}

#[no_mangle]
extern "C" fn main(_argc: core::ffi::c_int, _argv: *const *const u8) -> core::ffi::c_int {
    let vic2 = c64::vic2();
    let cia2 = c64::cia2();

    unsafe {
        vic2.border_color.write(BLACK);
        vic2.background_color0.write(BLACK);
        vic2.background_color1.write(GRAY1);
        vic2.background_color2.write(YELLOW);
        vic2.control_x.modify(|v| v | ControlXFlags::MULTICOLOR);

        screen::clear(&mut *SCREEN_1);
        screen::clear(&mut *SCREEN_2);
        screen::clear_text(&mut *TEXT_SCREEN_1);
        screen::clear_text(&mut *TEXT_SCREEN_2);

        // Set VIC2 memory at 0x8000–0xBFFF
        cia2.data_direction_port_a.write(CIA2DirectionA::VA15 | CIA2DirectionA::VA14);
        cia2.port_a.write(CIA2PortA::VA14);

        (&mut *CHARSET_1)[0..256].copy_from_slice(&TILESET[8 * (0 * 64)..8 * (32 + 0 * 64)]);
        (&mut *CHARSET_2)[0..256].copy_from_slice(&TILESET[8 * (1 * 64)..8 * (32 + 1 * 64)]);
        (&mut *CHARSET_3)[0..256].copy_from_slice(&TILESET[8 * (2 * 64)..8 * (32 + 2 * 64)]);
        (&mut *CHARSET_4)[0..256].copy_from_slice(&TILESET[8 * (3 * 64)..8 * (32 + 3 * 64)]);

        for i in 0..1000 {
            COLOR_RAM.offset(i).write(LIGHT_RED);
        }

        screen::set_vic2_buffer();

        // Clear animation counter
        for i in 0..MAP.len() {
            MAP[i] &= ANIMATION_COUNTER_MASK;
        }

        for x in 0..MAP_WIDTH {
            write_map::<{ (MAP_HEIGHT - 2) as isize }>(x, 1);
        }

        for x in 0..MAP_WIDTH {
            write_map_color::<{ (MAP_HEIGHT - 1) as isize }>(x, RED);
        }

        c64::hardware_raster_irq(247);

        loop {
            while NEW_FRAME.get() > 0 {}

            let mut w = MapTextWriter::new();

            screen::clear_text(&mut *screen::current_text());

            let start = FRAME_COUNTER.get() as u16;

            update_map();

            {
                // Copy map to screen
                let screen = screen::current();
                (*screen)[0..960].copy_from_slice(&MAP[0..960]);
            }

            {
                let mut end = FRAME_COUNTER.get() as u16;
                if end < start {
                    end += 255;
                }
                let time = end - start;
                uwrite!(&mut w, "{}", time).ok();
                uwrite!(&mut w, " HELLO WORLD").ok();
            }

            NEW_FRAME.set(1);
        }
    }
}

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

            let animation_counter = ANIMATION_COUNTER.get() + 1;
            ANIMATION_COUNTER.set(if animation_counter == 4 {
                0
            } else {
                animation_counter
            });

            if animation_counter == 4 {
                // Was the main loop too slow?
                if NEW_FRAME.get() == 0 {
                    loop {
                        vic2.border_color.write(BROWN);
                        vic2.border_color.write(BLACK);
                    }
                }

                NEW_FRAME.set(0);
                DRAW_TO_SCREEN_2.set(if DRAW_TO_SCREEN_2.get() > 0 { 0 } else { 1 });
            }
        }

        if DRAW_TO_SCREEN_2.get() == 0 {
            NEXT_TEXT_CHARSET = CharsetBank::AT_1000.bits() | ScreenBank::AT_0C00.bits();
        } else {
            NEXT_TEXT_CHARSET = CharsetBank::AT_1000.bits() | ScreenBank::AT_0800.bits();
        }

        screen::set_vic2_buffer();

        FRAME_COUNT += 1;
        *FRAME_COUNTER.as_ptr() += 1;

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
