#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(unused_imports, dead_code, unused_macros))]

#[macro_use]
extern crate didios;

use core::panic::PanicInfo;
use didios::exit_qemu;

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    panic!();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("ok");

    unsafe { exit_qemu(); }
    loop {}
}
