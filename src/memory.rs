use core::mem::MaybeUninit;
use core::ptr::addr_of_mut;

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use bootloader::BootInfo;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::frame::PhysFrame;
use x86_64::structures::paging::mapper::OffsetPageTable;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, Page, PageTable, PageTableFlags, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

const PAGE_SIZE: u64 = 4096;

static mut MAPPER: MaybeUninit<OffsetPageTable<'static>> = MaybeUninit::uninit();
static mut FRAME_ALLOCATOR: MaybeUninit<BootInfoFrameAllocator> = MaybeUninit::uninit();
static mut INITIALIZED: bool = false;

pub fn init(boot_info: &'static BootInfo) -> Result<u64, &'static str> {
    let physical_memory_offset = VirtAddr::try_new(boot_info.physical_memory_offset)
        .map_err(|_| "invalid physical memory offset")?;

    let level_4_frame = Cr3::read().0;
    let level_4_table_address = physical_memory_offset + level_4_frame.start_address().as_u64();
    let level_4_table = level_4_table_address.as_mut_ptr::<PageTable>();

    unsafe {
        let level_4_table: &'static mut PageTable = &mut *level_4_table;
        addr_of_mut!(MAPPER).write(MaybeUninit::new(OffsetPageTable::new(
            level_4_table,
            physical_memory_offset,
        )));
        addr_of_mut!(FRAME_ALLOCATOR).write(MaybeUninit::new(BootInfoFrameAllocator::new(
            &boot_info.memory_map,
        )));
        addr_of_mut!(INITIALIZED).write(true);
    }

    let usable_pages = boot_info
        .memory_map
        .iter()
        .filter(|region| region.region_type == MemoryRegionType::Usable)
        .fold(0u64, |total, region| {
            total.saturating_add(
                region
                    .range
                    .end_frame_number
                    .saturating_sub(region.range.start_frame_number),
            )
        });

    Ok(usable_pages.saturating_mul(PAGE_SIZE))
}

pub fn map_page(
    page: Page<Size4KiB>,
    frame: PhysFrame<Size4KiB>,
    flags: PageTableFlags,
) -> Result<(), &'static str> {
    unsafe {
        if !addr_of_mut!(INITIALIZED).read() {
            return Err("memory subsystem is not initialized");
        }
        let mapper = (*addr_of_mut!(MAPPER)).assume_init_mut();
        let allocator = (*addr_of_mut!(FRAME_ALLOCATOR)).assume_init_mut();
        mapper
            .map_to(page, frame, flags, allocator)
            .map_err(|_| "failed to map page")?
            .flush();
    }
    Ok(())
}

pub fn allocate_frame() -> Result<PhysFrame<Size4KiB>, &'static str> {
    unsafe {
        if !addr_of_mut!(INITIALIZED).read() {
            return Err("memory subsystem is not initialized");
        }
        (*addr_of_mut!(FRAME_ALLOCATOR))
            .assume_init_mut()
            .allocate_frame()
            .ok_or("no usable physical frame remains")
    }
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    region_index: usize,
    next_frame_number: u64,
}

impl BootInfoFrameAllocator {
    fn new(memory_map: &'static MemoryMap) -> Self {
        Self {
            memory_map,
            region_index: 0,
            next_frame_number: 0,
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        loop {
            let region = self.memory_map.get(self.region_index)?;
            if region.region_type != MemoryRegionType::Usable {
                self.region_index += 1;
                continue;
            }

            let frame_number = region
                .range
                .start_frame_number
                .max(self.next_frame_number);
            if frame_number < region.range.end_frame_number {
                self.next_frame_number = frame_number.checked_add(1)?;
                let address = frame_number.checked_mul(PAGE_SIZE)?;
                return PhysFrame::from_start_address(PhysAddr::new(address)).ok();
            }
            self.region_index += 1;
        }
    }
}
