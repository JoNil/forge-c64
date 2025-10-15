use core::mem::transmute;

use crate::tileset::TILESET;

pub const MAX_ENTITIES: u16 = 64;

pub struct Entities {
    count: u8,
    x: [i8; MAX_ENTITIES as usize],
    y: [i8; MAX_ENTITIES as usize],
}

pub unsafe fn entities() -> *mut Entities {
    transmute(TILESET.as_mut_ptr())
}