use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::{Page, PageTableFlags, Size4KiB};
use x86_64::VirtAddr;

use crate::memory;

const HEAP_START: u64 = 0x4444_4444_0000;
const HEAP_SIZE: usize = 100 * 1024;
const PAGE_SIZE: u64 = 4096;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init() -> Result<(), &'static str> {
    let heap_start = VirtAddr::new(HEAP_START);
    let heap_end = HEAP_START
        .checked_add(HEAP_SIZE as u64)
        .ok_or("kernel heap address overflow")?;
    let mut page_address = HEAP_START;
    while page_address < heap_end {
        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(page_address));
        let frame = memory::allocate_frame()?;
        memory::map_page(
            page,
            frame,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        )?;
        page_address += PAGE_SIZE;
    }

    unsafe {
        ALLOCATOR.lock().init(heap_start.as_mut_ptr(), HEAP_SIZE);
    }
    Ok(())
}

pub const fn size_kib() -> usize {
    HEAP_SIZE / 1024
}
