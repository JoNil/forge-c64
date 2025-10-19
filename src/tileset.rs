use core::mem;

use static_assertions::const_assert;

use crate::{entity::Entities, spawner::Spawners};

const TILESET_COUNT: usize = 2048;
pub static mut TILESET: [u8; TILESET_COUNT] = *include_bytes!("../assets/tileset.bin");

pub struct State {
    pub entities: Entities,
    pub spawners: Spawners,
}

const_assert!(mem::size_of::<State>() < TILESET_COUNT);

pub unsafe fn state() -> *mut State {
    mem::transmute(TILESET.as_mut_ptr())
}
