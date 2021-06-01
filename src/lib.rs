#![no_std]

#[no_mangle]
pub extern "C" fn factorial(n: u16) -> u16 {
    match n {
        0 => 1,
        _ => n * factorial(n - 1),
    }
}
