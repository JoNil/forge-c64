use crate::{
    map::{MAP, MAP_HEIGHT, MAP_WIDTH},
    tile::{
        clear_resource, has_resource, is_dir_down, is_dir_left, is_dir_right, is_dir_up,
        set_resource,
    },
    tileset::state,
};

pub const MAX_ENTITIES: usize = 64;

pub struct Entities {
    pub count: u8,
    pub x: [i8; MAX_ENTITIES],
    pub y: [i8; MAX_ENTITIES],
    pub i: [usize; MAX_ENTITIES],
}

#[inline(never)]
pub unsafe fn update_entites() {
    let entities = &mut (&mut *state()).entities;

    for i in 0..(entities.count as usize) {
        let mut x = entities.x[i];
        let mut y = entities.y[i];

        let mut map_i = entities.i[i];

        let tile = clear_resource(MAP[map_i]);
        MAP[map_i] = tile;

        if is_dir_down(tile) {
            map_i += MAP_WIDTH as usize;

            if !has_resource(MAP[map_i]) {
                y += 1;
                entities.y[i] = y;
                entities.i[i] = map_i;
            } else {
                map_i -= MAP_WIDTH as usize;
            }
        } else if is_dir_up(tile) {
            map_i -= MAP_WIDTH as usize;

            if !has_resource(MAP[map_i]) {
                y -= 1;
                entities.y[i] = y;
                entities.i[i] = map_i;
            } else {
                map_i += MAP_WIDTH as usize;
            }
        } else if is_dir_left(tile) {
            map_i -= 1;

            if !has_resource(MAP[map_i]) {
                x -= 1;
                entities.x[i] = x;
                entities.i[i] = map_i;
            } else {
                map_i += 1;
            }
        } else if is_dir_right(tile) {
            map_i += 1;

            if !has_resource(MAP[map_i]) {
                x += 1;
                entities.x[i] = x;
                entities.i[i] = map_i;
            } else {
                map_i -= 1;
            }
        }

        let new_tile = MAP[map_i];
        MAP[map_i] = set_resource(new_tile);
    }
}

#[inline(never)]
pub unsafe fn find_initial() {
    let entities = &mut (&mut *state()).entities;

    let mut i = 0;

    for x in 0..(MAP_WIDTH as i8) {
        for y in 0..(MAP_HEIGHT as i8) {
            let tile = MAP[i];

            if has_resource(tile) {
                let index = entities.count as usize;

                entities.x[index] = x;
                entities.y[index] = y;
                entities.i[index] = i;

                entities.count += 1;

                MAP[i] = clear_resource(tile);
            }

            i += 1;
        }
    }
}
