# AuraOS

AuraOS is a Rust x86-64 operating-system project targeting legacy and low-memory PCs. This repository is under active M1 kernel development; it is not yet a complete desktop operating system.

## Current checkpoint

- `no_std` x86-64 kernel entry point using the `bootloader` crate.
- COM1 serial output for early boot and fatal exception diagnostics.
- Kernel GDT, TSS, dedicated privilege/IST stacks, and IDT handlers for CPU exceptions.
- Boot memory-map frame allocator and active page-table mapper initialization.
- 100 KiB kernel heap backed by boot memory frames.
- Interactive AuraShell rendered in VGA text mode with polled PS/2 keyboard input.
- `help`, `about`, `memory`, `clear`, and `echo` shell commands.
- Pinned nightly toolchain and custom kernel target with SIMD disabled.
- BIOS/QEMU run path through `bootimage`.

Not implemented yet: UEFI boot, recoverable page faults, external interrupt controllers, scheduling, drivers, storage, user processes, graphical framebuffer UI, networking, and applications. The shell currently polls the legacy PS/2 controller and is intended for early bring-up.

## Prerequisites

- Rustup and the pinned nightly toolchain (the `rust-toolchain.toml` file selects it).
- On Windows, Visual Studio Build Tools with the **Desktop development with C++** workload, including the MSVC linker (`link.exe`).
- QEMU with `qemu-system-x86_64` available on `PATH`.
- The `bootimage` Cargo utility: `cargo install bootimage --locked`.

The toolchain file requests `rust-src` and `llvm-tools-preview`, needed by the custom target and boot image tooling.

## Run in QEMU

From PowerShell:

```powershell
.\scripts\run-qemu.ps1
```

The QEMU display shows the AuraShell status screen and command prompt. Type `help` to list commands. Serial output reports kernel, CPU-table, memory, heap, and shell initialization. This is VGA text mode, not the planned graphical desktop.

## Roadmap

1. M0: tooling and a serial-reporting BIOS boot in QEMU.
2. M1: BIOS/UEFI boot, CPU tables, exceptions, physical memory, paging, and heap. BIOS CPU tables, exception reporting, frame allocation, mapper initialization, a 100 KiB heap, and the first interactive text shell are in progress.
3. M2: platform timers, interrupts, and hardware discovery.
4. M3: storage, VFS, IPC, and system-call foundations.
5. M4: framebuffer graphics and the first usable shell.

The specification describes a multi-year operating-system effort. Each milestone should be independently bootable and tested in QEMU before expanding hardware support.
