pub const RESOURCE_BIT: u8 = 0x10;
pub const DIR_MASK: u8 = 0b11;

pub const TILE_DIR_DOWN: u8 = 0b00;
pub const TILE_DIR_LEFT: u8 = 0b01;
pub const TILE_DIR_UP: u8 = 0b10;
pub const TILE_DIR_RIGHT: u8 = 0b11;

pub const fn is_dir_down(tile: u8) -> bool {
    tile != 0 && (tile & DIR_MASK) == TILE_DIR_DOWN
}

pub const fn is_dir_left(tile: u8) -> bool {
    (tile & DIR_MASK) == TILE_DIR_LEFT
}

pub const fn is_dir_up(tile: u8) -> bool {
    (tile & DIR_MASK) == TILE_DIR_UP
}

pub const fn is_dir_right(tile: u8) -> bool {
    (tile & DIR_MASK) == TILE_DIR_RIGHT
}

pub const fn has_resource(tile: u8) -> bool {
    tile & RESOURCE_BIT > 0
}

pub const fn set_resource(tile: u8) -> u8 {
    tile | RESOURCE_BIT
}

pub const fn clear_resource(tile: u8) -> u8 {
    tile & (!RESOURCE_BIT)
}
