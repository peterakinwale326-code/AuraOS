#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::alloc::Layout;
use core::arch::asm;
use core::panic::PanicInfo;

mod cpu;
mod heap;
mod keyboard;
mod memory;
mod serial;
mod shell;
mod ui;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    disable_interrupts();
    serial::init();
    if serial::write_fmt(format_args!("AuraOS M1: kernel entered\r\n")).is_err() {
        halt_loop();
    }
    cpu::init();
    if serial::write_fmt(format_args!(
        "AuraOS M1: GDT, TSS, IST stacks and exception IDT loaded\r\n"
    ))
    .is_err()
    {
        halt_loop();
    }
    let usable_memory_bytes = match memory::init(boot_info) {
        Ok(bytes) => bytes,
        Err(error) => {
            let _ = serial::write_fmt(format_args!(
                "AuraOS: memory initialization failed: {error}\r\n"
            ));
            halt_loop();
        }
    };
    if serial::write_fmt(format_args!(
        "AuraOS M1: page mapper ready; {usable_memory_bytes} usable bytes\r\n"
    ))
    .is_err()
    {
        halt_loop();
    }
    if let Err(error) = heap::init() {
        let _ = serial::write_fmt(format_args!("AuraOS: heap initialization failed: {error}\r\n"));
        halt_loop();
    }
    if serial::write_fmt(format_args!(
        "AuraOS M1: {} KiB kernel heap online\r\n",
        heap::size_kib()
    ))
    .is_err()
    {
        halt_loop();
    }
    let terminal = match ui::Terminal::new(
        boot_info.physical_memory_offset,
        usable_memory_bytes,
    ) {
        Ok(terminal) => terminal,
        Err(error) => {
            let _ = serial::write_fmt(format_args!("AuraOS: text UI unavailable: {error}\r\n"));
            halt_loop();
        }
    };
    if serial::write_fmt(format_args!("AuraOS M1: AuraShell ready\r\n")).is_err() {
        halt_loop();
    }
    shell::run(terminal, usable_memory_bytes)
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    let _ = serial::write_fmt(format_args!("\r\nKERNEL PANIC: {info}\r\n"));
    halt_loop()
}

#[alloc_error_handler]
fn allocation_error(layout: Layout) -> ! {
    let _ = serial::write_fmt(format_args!(
        "\r\nKERNEL PANIC: allocation failed for {} bytes\r\n",
        layout.size()
    ));
    halt_loop()
}

fn halt_loop() -> ! {
    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

fn disable_interrupts() {
    unsafe {
        asm!("cli", options(nomem, nostack, preserves_flags));
    }
}
