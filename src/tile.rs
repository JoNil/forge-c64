const RESOURCE_BIT: u8 = 0x10;

pub const fn is_dir_left(tile: u8) -> bool {
    tile == 1 || tile == 5 || tile == 9
}

pub const fn is_dir_up(tile: u8) -> bool {
    tile == 2 || tile == 6 || tile == 10
}

pub const fn is_dir_right(tile: u8) -> bool {
    tile == 3 || tile == 7 || tile == 11
}

pub const fn is_dir_down(tile: u8) -> bool {
    tile == 4 || tile == 8 || tile == 12
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
