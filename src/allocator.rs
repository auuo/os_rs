use core::alloc::{GlobalAlloc, Layout};
use linked_list_allocator::LockedHeap;

use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};
use x86_64::structures::paging::mapper::MapToError;
use x86_64::VirtAddr;

pub const HEAP_START: usize = 0x_4444_4444_0000;
// 定义堆内存开始的虚拟内存
pub const HEAP_SIZE: usize = 100 * 1024; // 堆大小 100k

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// 初始化堆内存，将堆内存部分映射到页表
pub fn init_heap(mapper: &mut impl Mapper<Size4KiB>,
                 frame_allocator: &mut impl FrameAllocator<Size4KiB>, ) -> Result<(), MapToError<Size4KiB> > {
    // 计算页范围
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    // 将堆内存所在页映射到物理 frame
    for page in page_range {
        // 申请一个 frame，映射到该 frame 上
        let frame = frame_allocator.allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush()
        };
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

#[deprecated]
pub struct Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should be never called")
    }
}

/// 内存申请失败会调用该函数
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}