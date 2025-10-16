use core::mem::transmute;

use crate::{
    map::{MAP, MAP_HEIGHT, MAP_WIDTH},
    tile::{clear_resource, has_resource},
    tileset::TILESET,
};

pub const MAX_ENTITIES: u16 = 64;

pub struct Entities {
    pub count: u8,
    pub x: [i8; MAX_ENTITIES as usize],
    pub y: [i8; MAX_ENTITIES as usize],
}

pub unsafe fn entities() -> *mut Entities {
    transmute(TILESET.as_mut_ptr())
}

#[inline(never)]
pub unsafe fn find_initial() {
    let mut i = 0;

    let entities = &mut *entities();

    for x in 0..(MAP_WIDTH as i8) {
        for y in 0..(MAP_HEIGHT as i8) {
            let tile = MAP[i];

            if has_resource(tile) {
                let index = entities.count;

                entities.x[index as usize] = x;
                entities.y[index as usize] = y;

                entities.count += 1;

                MAP[i] = clear_resource(tile);
            }

            i += 1;
        }
    }
}
