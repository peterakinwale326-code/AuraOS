use core::fmt::{self, Write};

const COM1: u16 = 0x3F8;

pub fn init() {
    unsafe {
        write_port(COM1 + 1, 0x00);
        write_port(COM1 + 3, 0x80);
        write_port(COM1, 0x03);
        write_port(COM1 + 1, 0x00);
        write_port(COM1 + 3, 0x03);
        write_port(COM1 + 2, 0xC7);
        write_port(COM1 + 4, 0x0B);
    }
}

pub fn write_fmt(args: fmt::Arguments<'_>) -> fmt::Result {
    SerialPort.write_fmt(args)
}

struct SerialPort;

impl Write for SerialPort {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for byte in text.bytes() {
            unsafe {
                while read_port(COM1 + 5) & 0x20 == 0 {}
                write_port(COM1, byte);
            }
        }
        Ok(())
    }
}

unsafe fn write_port(port: u16, value: u8) {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }
}

unsafe fn read_port(port: u16) -> u8 {
    let value: u8;
    unsafe {
        core::arch::asm!(
            "in al, dx",
            in("dx") port,
            out("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }
    value
}
