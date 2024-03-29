use core::hint::unreachable_unchecked;
use mos_hardware::{
    c64,
    vic2::{CharsetBank, ScreenBank},
};
use vcell::VolatileCell;

pub const SCREEN_1: *mut [u8; 960] = 0x8800 as *mut [u8; 960];
pub const SCREEN_2: *mut [u8; 960] = 0x8C00 as *mut [u8; 960];
pub const TEXT_SCREEN_1: *mut [u8; 40] = (0x8800 + 960) as *mut [u8; 40];
pub const TEXT_SCREEN_2: *mut [u8; 40] = (0x8C00 + 960) as *mut [u8; 40];
pub const CHARSET_1: *mut [u8; 2048] = 0xa000 as *mut [u8; 2048];
pub const CHARSET_2: *mut [u8; 2048] = 0xa800 as *mut [u8; 2048];
pub const CHARSET_3: *mut [u8; 2048] = 0xb000 as *mut [u8; 2048];
pub const CHARSET_4: *mut [u8; 2048] = 0xb800 as *mut [u8; 2048];

pub static mut DRAW_TO_SCREEN_2: VolatileCell<u8> = VolatileCell::new(0);
pub static mut ANIMATION_COUNTER: VolatileCell<u8> = VolatileCell::new(0);

pub fn set_vic2_buffer() {
    let vic2 = c64::vic2();

    let bank = match unsafe { ANIMATION_COUNTER.get() } {
        0 => CharsetBank::AT_2000.bits(),
        1 => CharsetBank::AT_2800.bits(),
        2 => CharsetBank::AT_3000.bits(),
        3 => CharsetBank::AT_3800.bits(),
        _ => unsafe { unreachable_unchecked() },
    } | match unsafe { DRAW_TO_SCREEN_2.get() } {
        0 => ScreenBank::AT_0C00.bits(),
        1 => ScreenBank::AT_0800.bits(),
        _ => unsafe { unreachable_unchecked() },
    };

    unsafe { vic2.screen_and_charset_bank.write(bank) };
}

pub fn current() -> *mut [u8; 960] {
    unsafe {
        if DRAW_TO_SCREEN_2.get() > 0 {
            SCREEN_2
        } else {
            SCREEN_1
        }
    }
}

pub fn current_text() -> *mut [u8; 40] {
    unsafe {
        if DRAW_TO_SCREEN_2.get() > 0 {
            TEXT_SCREEN_2
        } else {
            TEXT_SCREEN_1
        }
    }
}

pub fn clear(screen: &mut [u8; 960]) {
    for c in screen {
        *c = 0x00;
    }
}

pub fn clear_text(text_screen: &mut [u8; 40]) {
    for c in text_screen {
        *c = b' ';
    }
}
