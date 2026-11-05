use core::fmt::Write;

use crate::keyboard::{Key, Keyboard};
use crate::ui::Terminal;

const INPUT_CAPACITY: usize = 64;

pub fn run(mut terminal: Terminal, usable_memory_bytes: u64) -> ! {
    let mut keyboard = Keyboard::new();
    let mut input = [0u8; INPUT_CAPACITY];
    let mut input_len = 0;
    terminal.prompt();

    loop {
        let Some(key) = keyboard.poll_key() else {
            core::hint::spin_loop();
            continue;
        };

        match key {
            Key::Character(character) if input_len < input.len() => {
                input[input_len] = character;
                input_len += 1;
                terminal.write_character(character);
            }
            Key::Backspace if input_len > 0 => {
                input_len -= 1;
                terminal.erase_character();
            }
            Key::Enter => {
                terminal.new_line();
                execute_command(&mut terminal, &input[..input_len], usable_memory_bytes);
                input_len = 0;
                terminal.prompt();
            }
            _ => {}
        }
    }
}

fn execute_command(terminal: &mut Terminal, input: &[u8], usable_memory_bytes: u64) {
    let input = trim(input);
    if input.is_empty() {
        return;
    }

    if input == b"help" {
        terminal.write_line("Available commands:");
        terminal.write_line("  help    Show this command list");
        terminal.write_line("  about   About AuraOS");
        terminal.write_line("  memory  Show usable physical memory");
        terminal.write_line("  clear   Clear the shell screen");
        terminal.write_line("  echo    Print text after the command");
    } else if input == b"about" {
        terminal.write_line("AuraOS: a Rust x86-64 OS for older, low-memory PCs.");
        terminal.write_line("This is an early development shell, not a finished desktop.");
    } else if input == b"memory" {
        let _ = writeln!(
            terminal,
            "Usable memory map: {} MiB",
            usable_memory_bytes / (1024 * 1024)
        );
    } else if input == b"clear" {
        terminal.redraw();
    } else if input == b"echo" {
        terminal.write_line("");
    } else if input.starts_with(b"echo ") {
        terminal.write_line_bytes(&input[5..]);
    } else {
        let _ = writeln!(terminal, "Unknown command: {}", Ascii(input));
        terminal.write_line("Type 'help' to see available commands.");
    }
}

fn trim(mut input: &[u8]) -> &[u8] {
    while input.first() == Some(&b' ') {
        input = &input[1..];
    }
    while input.last() == Some(&b' ') {
        input = &input[..input.len() - 1];
    }
    input
}

struct Ascii<'a>(&'a [u8]);

impl core::fmt::Display for Ascii<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for &byte in self.0 {
            formatter.write_char(char::from(byte))?;
        }
        Ok(())
    }
}
