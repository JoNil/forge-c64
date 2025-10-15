// TODO: Reuse this memory for something usefull after the copy
pub static TILESET: [u8; 2048] = *include_bytes!("tileset.bin");
