#![no_std]

extern "C" {
    fn __chrout(c: u8);
}

const SCREEN: *mut [u8; 1000] = 0x0400 as *mut [u8; 1000];

fn clear_screen() {
    unsafe {
        let screen = &mut *SCREEN;

        for c in screen {
            *c = 0x20;
        }
    }
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn main() {
    clear_screen();
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
