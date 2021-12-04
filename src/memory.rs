use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::PageTable;

/// 返回 4 级页表
pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    // 读取 4 级页表物理地址
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

/// 通过页表查找虚拟地址对应的物理地址
/// 没有直接使用 virtAddr - physical_memory_offset 的原因是它只在虚拟地址是 complete-mapping 的一部分时才生效。
/// 比如 vga 同时映射到 0xb8000 和 0xb8000 + physical_memory_offset。
pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    translate_addr_inner(addr, physical_memory_offset)
}

fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
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