// This memory is used for entities after initilization
pub static mut TILESET: [u8; 2048] = *include_bytes!("tileset.bin");
