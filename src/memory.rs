use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB};

// 使用 x86_64 crate 提供的页表抽象
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// 返回 4 级页表
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    // 读取 4 级页表物理地址
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

/// 使用 bootloader 传递的 memory map 查询可使用的内存 frame
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    // 获取所有可用的 frame
    pub fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // 从 bootInfo memory map 里获取所有区域
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        // 获取每一块的地址范围
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // 将每一块区域划分为 4096 的大小
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // 转换为 PhysFrame 类型
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        // 获取可用的 frame，每次申请使用 next 位置的 frame
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

/// 创建一个虚拟内存到物理内存的映射
/// frame_allocator 是用来如果需要新的 frame 存储页表时使用的
#[deprecated]
pub fn create_example_mapping(page: Page,
                              mapper: &mut OffsetPageTable,
                              frame_allocator: &mut impl FrameAllocator<Size4KiB>) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush(); // 刷新 tlb
}

/// 永远返回 None，测试使用
#[deprecated]
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}

/// 通过页表查找虚拟地址对应的物理地址
/// 没有直接使用 virtAddr - physical_memory_offset 的原因是它只在虚拟地址是 complete-mapping 的一部分时才生效。
/// 比如 vga 同时映射到 0xb8000 和 0xb8000 + physical_memory_offset。
/// 使用 OffsetPageTable 代替
#[deprecated]
unsafe fn _translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    _translate_addr_inner(addr, physical_memory_offset)
}

#[deprecated]
fn _translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    use x86_64::structures::paging::page_table::FrameError;
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
    ];
    let mut frame = level_4_table_frame;
    // 遍历每一级页表
    for &index in &table_indexes {
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr = virt.as_ptr();
        let table: &PageTable = unsafe { &*table_ptr };

        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        };
    }
    // 页表 frame 指向的地址加上页内偏移量
    Some(frame.start_address() + u64::from(addr.page_offset()))
}