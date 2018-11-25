#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(unused_imports, dead_code, unused_macros))]
#![feature(abi_x86_interrupt)]

#[macro_use]
extern crate didios;
extern crate x86_64;
#[macro_use]
extern crate lazy_static;

use core::panic::PanicInfo;
use x86_64::structures::idt::{InterruptDescriptorTable, ExceptionStackFrame};
use didios::interrupts::{PICS, TIMER_INTERRUPT_ID};

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    kprintln!("Hello World{}", "!");

    didios::gdt::init();
    init_idt();
    unsafe { PICS.lock().initialize() }
    x86_64::instructions::interrupts::enable();

    kprintln!("It did not crash :)");
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

        let timer_interrupt_id = usize::from(TIMER_INTERRUPT_ID);
        idt[timer_interrupt_id].set_handler_fn(timer_interrupt_handler);

        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut ExceptionStackFrame) {
    kprintln!("EXCEPTION BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn doublefault_handler(stack_frame: &mut ExceptionStackFrame, _error_code: u64) {
    kprintln!("EXCEPTION DOUBLEFAULT\n{:#?}", stack_frame);
    loop {}
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut ExceptionStackFrame) {
    kprint!(".");
    unsafe { PICS.lock().notify_end_of_interrupt(TIMER_INTERRUPT_ID) }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("{}", info);

    loop {}
}
