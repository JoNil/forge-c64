#![no_std]
#![no_main]

extern crate mos_alloc;

use map::{write_map_color, MAP, MAP_HEIGHT, MAP_WIDTH};
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
use tileset::TILESET;
use vcell::VolatileCell;

use crate::{
    entity::update_entites, screen::current_text, spawner::update_spawners,
    text::digit_to_screen_code, tile::ANIMATION_COUNTER_MASK, tileset::state,
};

mod entity;
mod map;
mod scratch;
mod screen;
mod spawner;
mod text;
mod tile;
mod tileset;

static mut NEW_FRAME: VolatileCell<u8> = VolatileCell::new(0);
static mut FRAME_COUNTER: VolatileCell<u8> = VolatileCell::new(0);

#[inline(never)]
fn copy_screen() {
    unsafe {
        let screen = screen::current() as *mut u8;
        let map = MAP.as_mut_ptr();

        for i in 0..960 {
            *screen.offset(i) = *map.offset(i)
        }
    }
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
        cia2.data_direction_port_a
            .write(CIA2DirectionA::VA15 | CIA2DirectionA::VA14);
        cia2.port_a.write(CIA2PortA::VA14);

        (&mut *CHARSET_1)[0..512].copy_from_slice(&TILESET[8 * (0 * 64)..8 * (64 + 0 * 64)]);
        (&mut *CHARSET_2)[0..512].copy_from_slice(&TILESET[8 * (1 * 64)..8 * (64 + 1 * 64)]);
        (&mut *CHARSET_3)[0..512].copy_from_slice(&TILESET[8 * (2 * 64)..8 * (64 + 2 * 64)]);
        (&mut *CHARSET_4)[0..512].copy_from_slice(&TILESET[8 * (3 * 64)..8 * (64 + 3 * 64)]);

        for i in 0..1000 {
            COLOR_RAM.offset(i).write(LIGHT_RED);
        }

        screen::set_vic2_buffer();

        for i in 0..MAP.len() {
            MAP[i] &= ANIMATION_COUNTER_MASK;
        }

        for x in 0..MAP_WIDTH {
            write_map_color::<{ (MAP_HEIGHT - 1) as isize }>(x, RED);
        }

        {
            let state = &mut *state();
            state.spawners.count = 0;
            state.entities.count = 0;
        }

        entity::find_initial();
        spawner::find_initial();

        c64::hardware_raster_irq(247);

        loop {
            while NEW_FRAME.get() > 0 {}

            let start = FRAME_COUNTER.get() as u16;

            update_entites();
            update_spawners();

            copy_screen();

            {
                let mut end = FRAME_COUNTER.get() as u16;
                if end < start {
                    end += 255;
                }
                let time = end - start;

                let entity_count = (*state()).entities.count;
                let spawner_count = (*state()).spawners.count;

                let text = &mut *current_text();

                text[0] = digit_to_screen_code(time as u8);
                text[2] = digit_to_screen_code(entity_count);
                text[4] = digit_to_screen_code(spawner_count);
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
