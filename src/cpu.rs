use core::mem::MaybeUninit;
use core::ptr::addr_of_mut;
use x86_64::instructions::segmentation::{DS, ES, Segment, SS, CS};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};
use x86_64::structures::idt::{
    InterruptDescriptorTable, InterruptStackFrame, InterruptStackIndex,
};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

use crate::{halt_loop, serial};

const DOUBLE_FAULT_IST_INDEX: u16 = 0;
const NMI_IST_INDEX: u16 = 1;
const MACHINE_CHECK_IST_INDEX: u16 = 2;
const STACK_SIZE: usize = 4096;

#[repr(align(16))]
struct Stack([u8; STACK_SIZE]);

static mut PRIVILEGE_STACK: Stack = Stack([0; STACK_SIZE]);
static mut DOUBLE_FAULT_STACK: Stack = Stack([0; STACK_SIZE]);
static mut NMI_STACK: Stack = Stack([0; STACK_SIZE]);
static mut MACHINE_CHECK_STACK: Stack = Stack([0; STACK_SIZE]);

static mut TSS: MaybeUninit<TaskStateSegment> = MaybeUninit::uninit();
static mut GDT: MaybeUninit<GlobalDescriptorTable> = MaybeUninit::uninit();
static mut IDT: MaybeUninit<InterruptDescriptorTable> = MaybeUninit::uninit();

pub fn init() {
    unsafe {
        initialize_tss();
        initialize_gdt();
        initialize_idt();
    }
}

unsafe fn initialize_tss() {
    let tss_ptr = addr_of_mut!(TSS).cast::<TaskStateSegment>();
    tss_ptr.write(TaskStateSegment::new());
    let tss = &mut *tss_ptr;

    tss.privilege_stack_table[0] = stack_top(addr_of_mut!(PRIVILEGE_STACK));
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] =
        stack_top(addr_of_mut!(DOUBLE_FAULT_STACK));
    tss.interrupt_stack_table[NMI_IST_INDEX as usize] = stack_top(addr_of_mut!(NMI_STACK));
    tss.interrupt_stack_table[MACHINE_CHECK_IST_INDEX as usize] =
        stack_top(addr_of_mut!(MACHINE_CHECK_STACK));
}

unsafe fn initialize_gdt() {
    let gdt_ptr = addr_of_mut!(GDT).cast::<GlobalDescriptorTable>();
    gdt_ptr.write(GlobalDescriptorTable::new());
    let gdt = &mut *gdt_ptr;

    let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    let data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
    let tss = &*addr_of_mut!(TSS).cast::<TaskStateSegment>();
    let tss_selector = gdt.add_entry(Descriptor::tss_segment(tss));

    gdt.load();
    unsafe {
        CS::set_reg(code_selector);
        DS::set_reg(data_selector);
        ES::set_reg(data_selector);
        SS::set_reg(data_selector);
        load_tss(tss_selector);
    }
}

unsafe fn initialize_idt() {
    let idt_ptr = addr_of_mut!(IDT).cast::<InterruptDescriptorTable>();
    idt_ptr.write(InterruptDescriptorTable::new());
    let idt = &mut *idt_ptr;

    idt.divide_error.set_handler_fn(divide_error_handler);
    idt.debug.set_handler_fn(debug_handler);
    idt.non_maskable_interrupt
        .set_handler_fn(non_maskable_interrupt_handler)
        .set_stack_index(InterruptStackIndex::new(NMI_IST_INDEX).unwrap());
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.overflow.set_handler_fn(overflow_handler);
    idt.bound_range_exceeded
        .set_handler_fn(bound_range_exceeded_handler);
    idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
    idt.device_not_available
        .set_handler_fn(device_not_available_handler);
    idt.double_fault
        .set_handler_fn(double_fault_handler)
        .set_stack_index(InterruptStackIndex::new(DOUBLE_FAULT_IST_INDEX).unwrap());
    idt.invalid_tss.set_handler_fn(invalid_tss_handler);
    idt.segment_not_present
        .set_handler_fn(segment_not_present_handler);
    idt.stack_segment_fault
        .set_handler_fn(stack_segment_fault_handler);
    idt.general_protection_fault
        .set_handler_fn(general_protection_fault_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt.x87_floating_point
        .set_handler_fn(x87_floating_point_handler);
    idt.alignment_check
        .set_handler_fn(alignment_check_handler);
    idt.machine_check
        .set_handler_fn(machine_check_handler)
        .set_stack_index(InterruptStackIndex::new(MACHINE_CHECK_IST_INDEX).unwrap());
    idt.simd_floating_point
        .set_handler_fn(simd_floating_point_handler);

    idt.load();
}

unsafe fn stack_top(stack: *mut Stack) -> VirtAddr {
    let top = addr_of_mut!((*stack).0).cast::<u8>().add(STACK_SIZE);
    VirtAddr::new(top as u64)
}

fn fatal_exception(name: &str, frame: InterruptStackFrame) -> ! {
    let _ = serial::write_fmt(format_args!("\r\nEXCEPTION {name}: {frame:?}\r\n"));
    halt_loop()
}

fn fatal_exception_with_code(name: &str, frame: InterruptStackFrame, code: u64) -> ! {
    let _ = serial::write_fmt(format_args!(
        "\r\nEXCEPTION {name} (error code {code:#x}): {frame:?}\r\n"
    ));
    halt_loop()
}

extern "x86-interrupt" fn divide_error_handler(frame: InterruptStackFrame) {
    fatal_exception("divide error", frame)
}

extern "x86-interrupt" fn debug_handler(frame: InterruptStackFrame) {
    fatal_exception("debug", frame)
}

extern "x86-interrupt" fn non_maskable_interrupt_handler(frame: InterruptStackFrame) {
    fatal_exception("non-maskable interrupt", frame)
}

extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    fatal_exception("breakpoint", frame)
}

extern "x86-interrupt" fn overflow_handler(frame: InterruptStackFrame) {
    fatal_exception("overflow", frame)
}

extern "x86-interrupt" fn bound_range_exceeded_handler(frame: InterruptStackFrame) {
    fatal_exception("bound-range exceeded", frame)
}

extern "x86-interrupt" fn invalid_opcode_handler(frame: InterruptStackFrame) {
    fatal_exception("invalid opcode", frame)
}

extern "x86-interrupt" fn device_not_available_handler(frame: InterruptStackFrame) {
    fatal_exception("device not available", frame)
}

extern "x86-interrupt" fn double_fault_handler(
    frame: InterruptStackFrame,
    code: u64,
) -> ! {
    fatal_exception_with_code("double fault", frame, code)
}

extern "x86-interrupt" fn invalid_tss_handler(frame: InterruptStackFrame, code: u64) {
    fatal_exception_with_code("invalid TSS", frame, code)
}

extern "x86-interrupt" fn segment_not_present_handler(
    frame: InterruptStackFrame,
    code: u64,
) {
    fatal_exception_with_code("segment not present", frame, code)
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    frame: InterruptStackFrame,
    code: u64,
) {
    fatal_exception_with_code("stack-segment fault", frame, code)
}

extern "x86-interrupt" fn general_protection_fault_handler(
    frame: InterruptStackFrame,
    code: u64,
) {
    fatal_exception_with_code("general protection fault", frame, code)
}

extern "x86-interrupt" fn page_fault_handler(
    frame: InterruptStackFrame,
    code: u64,
) {
    let address = x86_64::registers::control::Cr2::read();
    let _ = serial::write_fmt(format_args!(
        "\r\nEXCEPTION page fault at {address:?} (error code {code:#x}): {frame:?}\r\n"
    ));
    halt_loop()
}

extern "x86-interrupt" fn x87_floating_point_handler(frame: InterruptStackFrame) {
    fatal_exception("x87 floating point", frame)
}

extern "x86-interrupt" fn alignment_check_handler(frame: InterruptStackFrame, code: u64) {
    fatal_exception_with_code("alignment check", frame, code)
}

extern "x86-interrupt" fn machine_check_handler(frame: InterruptStackFrame) -> ! {
    fatal_exception("machine check", frame)
}

extern "x86-interrupt" fn simd_floating_point_handler(frame: InterruptStackFrame) {
    fatal_exception("SIMD floating point", frame)
}
