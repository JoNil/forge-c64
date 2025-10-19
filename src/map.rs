use mos_hardware::c64::COLOR_RAM;

pub const MAP_WIDTH: u8 = 40;
pub const MAP_HEIGHT: u8 = 25;

pub static mut MAP: [u8; 1000] = *include_bytes!("../assets/map.bin");

pub fn read_map<const Y: isize>(x: u8) -> u8 {
    unsafe {
        MAP.as_ptr()
            .offset(Y * (MAP_WIDTH as isize))
            .offset(x as isize)
            .read()
    }
}

pub fn write_map<const Y: isize>(x: u8, val: u8) {
    unsafe {
        MAP.as_mut_ptr()
            .offset(Y * (MAP_WIDTH as isize))
            .offset(x as isize)
            .write(val);
    }
}

pub fn write_map_color<const Y: isize>(x: u8, color: u8) {
    unsafe {
        COLOR_RAM
            .offset(Y * (MAP_WIDTH as isize))
            .offset(x as isize)
            .write(color);
    }
}
