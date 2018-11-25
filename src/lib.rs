#![no_std]

extern crate bootloader_precompiled;
extern crate volatile;
extern crate spin;
extern crate uart_16550;
extern crate x86_64;
extern crate pic8259_simple;
#[macro_use]
extern crate lazy_static;

#[cfg(test)]
extern crate array_init;
#[cfg(test)]
extern crate std;

pub mod gdt;
pub mod interrupts;
#[macro_use]
pub mod vga_buffer;
#[macro_use]
pub mod serial;

pub unsafe fn exit_qemu() {
    use x86_64::instructions::port::Port;

    let mut port = Port::<u32>::new(0xf4);
    port.write(0);
}
