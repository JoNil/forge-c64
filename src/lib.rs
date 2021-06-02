#![no_std]

extern "C" {
    fn __chrout(c: u8);
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn main() {
    __chrout(b'H');
    __chrout(b'E');
    __chrout(b'L');
    __chrout(b'L');
    __chrout(b'O');
    __chrout(b' ');
    __chrout(b'F');
    __chrout(b'R');
    __chrout(b'O');
    __chrout(b'M');
    __chrout(b' ');
    __chrout(b'R');
    __chrout(b'U');
    __chrout(b'S');
    __chrout(b'T');
}
