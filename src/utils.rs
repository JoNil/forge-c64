#[macro_export]
macro_rules! read {
    ($var:expr) => {{
        unsafe { (&$var as *const u8).read_volatile() }
    }};
}

pub use read;
