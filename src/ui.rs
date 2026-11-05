use core::fmt::{self, Write};
use x86_64::VirtAddr;

const VGA_BUFFER_PHYS: u64 = 0xB8000;
const WIDTH: usize = 80;
const HEIGHT: usize = 25;
const SHELL_TOP: usize = 7;
const SHELL_BOTTOM: usize = 22;

const BG: u8 = 0x1;
const PANEL: u8 = 0x9;
const ACCENT: u8 = 0xB;
const TEXT: u8 = 0xF;
const MUTED: u8 = 0x7;
const SUCCESS: u8 = 0xA;

pub struct Terminal {
    buffer: *mut u16,
    row: usize,
    column: usize,
    usable_memory_bytes: u64,
}

impl Terminal {
    pub fn new(physical_memory_offset: u64, usable_memory_bytes: u64) -> Result<Self, &'static str> {
        let virtual_address = physical_memory_offset
            .checked_add(VGA_BUFFER_PHYS)
            .and_then(|address| VirtAddr::try_new(address).ok())
            .ok_or("invalid VGA text buffer address")?;
        let mut terminal = Self {
            buffer: virtual_address.as_mut_ptr::<u16>(),
            row: SHELL_TOP,
            column: 0,
            usable_memory_bytes,
        };
        terminal.redraw_with_memory(usable_memory_bytes);
        Ok(terminal)
    }

    pub fn redraw(&mut self) {
        self.redraw_with_memory(self.usable_memory_bytes);
    }

    fn redraw_with_memory(&mut self, usable_memory_bytes: u64) {
        unsafe {
            self.fill(0, 0, WIDTH, HEIGHT, 0x20, BG);
            self.fill(0, 0, WIDTH, 1, 0x20, PANEL);
            self.fill(0, HEIGHT - 1, WIDTH, 1, 0x20, PANEL);
            self.fill(2, 3, 76, 1, 0x20, ACCENT);
            self.fill(2, 5, 76, 1, 0x20, PANEL);
            self.fill(0, SHELL_BOTTOM + 1, WIDTH, 1, 0x20, PANEL);

            self.draw_text(2, 0, "A U R A O S", TEXT, PANEL);
            self.draw_text(2, 2, "A fresh start for older hardware", MUTED, BG);
            self.draw_text(2, 4, "SYSTEM STATUS", ACCENT, BG);
            self.draw_text(4, 6, "KERNEL  ONLINE", SUCCESS, PANEL);
            self.draw_text(30, 6, "MEMORY", ACCENT, PANEL);
            self.draw_memory(39, 6, usable_memory_bytes);
            self.draw_text(56, 6, "AURA SHELL", ACCENT, PANEL);
            self.draw_text(2, 24, "PS/2 KEYBOARD  |  TYPE 'help' FOR COMMANDS", MUTED, PANEL);
        }
        self.row = SHELL_TOP;
        self.column = 0;
    }

    pub fn prompt(&mut self) {
        if self.column != 0 {
            self.new_line();
        }
        for byte in b"aura> " {
            self.write_character(*byte);
        }
    }

    pub fn write_character(&mut self, character: u8) {
        if self.column >= WIDTH - 2 {
            self.new_line();
        }
        self.put(self.column + 1, self.row, character, TEXT, BG);
        self.column += 1;
    }

    pub fn erase_character(&mut self) {
        if self.column > 0 {
            self.column -= 1;
            self.put(self.column + 1, self.row, b' ', TEXT, BG);
        }
    }

    pub fn new_line(&mut self) {
        self.row += 1;
        self.column = 0;
        if self.row > SHELL_BOTTOM {
            self.scroll_shell();
            self.row = SHELL_BOTTOM;
        }
    }

    pub fn write_line(&mut self, text: &str) {
        let _ = self.write_str(text);
        self.new_line();
    }

    pub fn write_line_bytes(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.write_character(byte);
        }
        self.new_line();
    }

    fn scroll_shell(&mut self) {
        unsafe {
            for row in SHELL_TOP..SHELL_BOTTOM {
                for column in 0..WIDTH {
                    let cell = self.buffer.add((row + 1) * WIDTH + column).read_volatile();
                    self.buffer.add(row * WIDTH + column).write_volatile(cell);
                }
            }
            self.fill(0, SHELL_BOTTOM, WIDTH, 1, b' ', BG);
        }
    }

    unsafe fn fill(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        character: u8,
        color: u8,
    ) {
        let right = x.saturating_add(width).min(WIDTH);
        let bottom = y.saturating_add(height).min(HEIGHT);
        let cell = u16::from(character) | (u16::from(color) << 8);

        for row in y.min(HEIGHT)..bottom {
            for column in x.min(WIDTH)..right {
                self.buffer
                    .add(row * WIDTH + column)
                    .write_volatile(cell);
            }
        }
    }

    unsafe fn draw_text(
        &mut self,
        x: usize,
        y: usize,
        text: &str,
        foreground: u8,
        background: u8,
    ) {
        if y >= HEIGHT {
            return;
        }
        for (index, byte) in text.bytes().enumerate() {
            let column = x + index;
            if column >= WIDTH {
                break;
            }
            self.put(column, y, byte, foreground, background);
        }
    }

    unsafe fn draw_memory(&mut self, x: usize, y: usize, bytes: u64) {
        let value = bytes / (1024 * 1024);
        let mut digits = [0u8; 20];
        let mut value = value;
        let mut len = 0;
        loop {
            digits[len] = b'0' + (value % 10) as u8;
            len += 1;
            value /= 10;
            if value == 0 {
                break;
            }
        }
        for (index, digit) in digits[..len].iter().rev().enumerate() {
            self.put(x + index, y, *digit, TEXT, PANEL);
        }
        self.draw_text(x + len, y, " MiB", TEXT, PANEL);
    }

    fn put(&mut self, x: usize, y: usize, character: u8, foreground: u8, background: u8) {
        if x >= WIDTH || y >= HEIGHT {
            return;
        }
        let color = foreground | (background << 4);
        let cell = u16::from(character) | (u16::from(color) << 8);
        unsafe {
            self.buffer
                .add(y * WIDTH + x)
                .write_volatile(cell);
        }
    }
}

impl Write for Terminal {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for byte in text.bytes() {
            match byte {
                b'\n' => self.new_line(),
                byte => self.write_character(byte),
            }
        }
        Ok(())
    }
}
