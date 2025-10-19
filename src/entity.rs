use core::mem::transmute;

use crate::{
    map::{MAP, MAP_HEIGHT, MAP_WIDTH},
    tile::{
        clear_resource, has_resource, is_dir_down, is_dir_left, is_dir_right, is_dir_up,
        set_resource,
    },
    tileset::TILESET,
};

pub const MAX_ENTITIES: usize = 64;

pub struct Entities {
    pub count: u8,
    pub x: [i8; MAX_ENTITIES],
    pub y: [i8; MAX_ENTITIES],
    pub i: [usize; MAX_ENTITIES],
}

pub unsafe fn entities() -> *mut Entities {
    transmute(TILESET.as_mut_ptr())
}

#[inline(never)]
pub unsafe fn update_entites() {
    let entities = &mut *entities();

    for i in 0..(entities.count as usize) {
        let mut x = entities.x[i];
        let mut y = entities.y[i];

        let mut map_i = entities.i[i];

        let tile = clear_resource(MAP[map_i]);
        MAP[map_i] = tile;

        if is_dir_down(tile) {
            y += 1;
            map_i += MAP_WIDTH as usize;

            entities.y[i] = y;
            entities.i[i] = map_i;
        } else if is_dir_up(tile) {
            y -= 1;
            map_i -= MAP_WIDTH as usize;

            entities.y[i] = y;
            entities.i[i] = map_i;
        } else if is_dir_left(tile) {
            x -= 1;
            map_i -= 1;

            entities.x[i] = x;
            entities.i[i] = map_i;
        } else if is_dir_right(tile) {
            x += 1;
            map_i += 1;

            entities.x[i] = x;
            entities.i[i] = map_i;
        }

        let new_tile = MAP[map_i];
        MAP[map_i] = set_resource(new_tile);
    }
}

#[inline(never)]
pub unsafe fn find_initial() {
    let entities = &mut *entities();

    let mut i = 0;

    for x in 0..(MAP_WIDTH as i8) {
        for y in 0..(MAP_HEIGHT as i8) {
            let tile = MAP[i];

            if has_resource(tile) {
                let index = entities.count;

                entities.x[index as usize] = x;
                entities.y[index as usize] = y;
                entities.i[index as usize] = i;

                entities.count += 1;

                MAP[i] = clear_resource(tile);
            }

            i += 1;
        }
    }
}
