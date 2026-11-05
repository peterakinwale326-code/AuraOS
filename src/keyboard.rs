use core::arch::asm;

const STATUS_PORT: u16 = 0x64;
const DATA_PORT: u16 = 0x60;

#[derive(Clone, Copy)]
pub enum Key {
    Character(u8),
    Enter,
    Backspace,
}

pub struct Keyboard {
    left_shift_pressed: bool,
    right_shift_pressed: bool,
    caps_lock: bool,
    caps_lock_pressed: bool,
    extended: bool,
}

impl Keyboard {
    pub const fn new() -> Self {
        Self {
            left_shift_pressed: false,
            right_shift_pressed: false,
            caps_lock: false,
            caps_lock_pressed: false,
            extended: false,
        }
    }

    pub fn poll_key(&mut self) -> Option<Key> {
        if unsafe { read_port(STATUS_PORT) } & 1 == 0 {
            return None;
        }

        let scan_code = unsafe { read_port(DATA_PORT) };
        if scan_code == 0xE0 {
            self.extended = true;
            return None;
        }
        if self.extended {
            self.extended = false;
            return None;
        }

        match scan_code {
            0x2A => {
                self.left_shift_pressed = true;
                return None;
            }
            0x36 => {
                self.right_shift_pressed = true;
                return None;
            }
            0xAA => {
                self.left_shift_pressed = false;
                return None;
            }
            0xB6 => {
                self.right_shift_pressed = false;
                return None;
            }
            0x3A if !self.caps_lock_pressed => {
                self.caps_lock = !self.caps_lock;
                self.caps_lock_pressed = true;
                return None;
            }
            0xBA => {
                self.caps_lock_pressed = false;
                return None;
            }
            0x3A => {
                return None;
            }
            code if code & 0x80 != 0 => return None,
            _ => {}
        }

        match scan_code {
            0x1C => Some(Key::Enter),
            0x0E => Some(Key::Backspace),
            code => self.translate(code).map(Key::Character),
        }
    }

    fn translate(&self, scan_code: u8) -> Option<u8> {
        let shift_pressed = self.left_shift_pressed || self.right_shift_pressed;
        let (plain, shifted) = match scan_code {
            0x02 => (b'1', b'!'),
            0x03 => (b'2', b'@'),
            0x04 => (b'3', b'#'),
            0x05 => (b'4', b'$'),
            0x06 => (b'5', b'%'),
            0x07 => (b'6', b'^'),
            0x08 => (b'7', b'&'),
            0x09 => (b'8', b'*'),
            0x0A => (b'9', b'('),
            0x0B => (b'0', b')'),
            0x0C => (b'-', b'_'),
            0x0D => (b'=', b'+'),
            0x10 => (b'q', b'Q'),
            0x11 => (b'w', b'W'),
            0x12 => (b'e', b'E'),
            0x13 => (b'r', b'R'),
            0x14 => (b't', b'T'),
            0x15 => (b'y', b'Y'),
            0x16 => (b'u', b'U'),
            0x17 => (b'i', b'I'),
            0x18 => (b'o', b'O'),
            0x19 => (b'p', b'P'),
            0x1A => (b'[', b'{'),
            0x1B => (b']', b'}'),
            0x1E => (b'a', b'A'),
            0x1F => (b's', b'S'),
            0x20 => (b'd', b'D'),
            0x21 => (b'f', b'F'),
            0x22 => (b'g', b'G'),
            0x23 => (b'h', b'H'),
            0x24 => (b'j', b'J'),
            0x25 => (b'k', b'K'),
            0x26 => (b'l', b'L'),
            0x27 => (b';', b':'),
            0x28 => (b'\'', b'"'),
            0x29 => (b'`', b'~'),
            0x2B => (b'\\', b'|'),
            0x2C => (b'z', b'Z'),
            0x2D => (b'x', b'X'),
            0x2E => (b'c', b'C'),
            0x2F => (b'v', b'V'),
            0x30 => (b'b', b'B'),
            0x31 => (b'n', b'N'),
            0x32 => (b'm', b'M'),
            0x33 => (b',', b'<'),
            0x34 => (b'.', b'>'),
            0x35 => (b'/', b'?'),
            0x39 => (b' ', b' '),
            _ => return None,
        };

        if plain.is_ascii_lowercase() {
            Some(if shift_pressed ^ self.caps_lock {
                shifted
            } else {
                plain
            })
        } else {
            Some(if shift_pressed { shifted } else { plain })
        }
    }
}

unsafe fn read_port(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }
    value
}
