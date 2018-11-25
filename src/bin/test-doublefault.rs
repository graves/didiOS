#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(unused_imports, dead_code, unused_macros))]
#![feature(abi_x86_interrupt)]

#[macro_use]
extern crate didios;
extern crate x86_64;
#[macro_use]
extern crate lazy_static;

use didios::exit_qemu;
use core::panic::PanicInfo;
use x86_64::structures::idt::{InterruptDescriptorTable, ExceptionStackFrame};

#[cfg(not(test))]
#[no_mangle]
#[allow(unconditional_recursion)]
pub extern "C" fn _start() -> ! {
    didios::gdt::init();
    init_idt();

    fn stack_overflow() {
        stack_overflow();
    }

    stack_overflow();

    serial_println!("failed");
    serial_println!("no exception occured");

    unsafe {
        exit_qemu();
    }

    loop {}
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);

        unsafe {
            idt.double_fault.set_handler_fn(doublefault_handler)
                .set_stack_index(didios::gdt::DOUBLEFAULT_IST_INDEX);
        }

        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut ExceptionStackFrame) {
    kprintln!("EXCEPTION BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn doublefault_handler(_stack_frame: &mut ExceptionStackFrame, _error_code: u64) {
    serial_println!("ok");

    unsafe {
        exit_qemu();
    }

    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("failed");
    serial_println!("{}", info);

    unsafe {
        exit_qemu();
    }

    loop {}
}
