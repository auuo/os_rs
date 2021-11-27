use lazy_static::lazy_static;
use x86_64::instructions::segmentation::Segment;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

/// double fault 处理函数使用的栈序号（interrupt stack table 中的序号）
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5; // 栈大小 20k
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE]; // 还没有内存管理，所以使用编译期分配的内存
            // 没有 guard page，所以不应该在里面强烈的使用导致爆栈
            // 因为栈从高到低生长，所以要返回高位地址
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        // 加载 tss
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, tss_selector })
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::registers::segmentation::CS;
    use x86_64::instructions::tables::load_tss;
    // 使用指令 lgdt 向 cpu 注册
    GDT.0.load();
    unsafe {
        // 注册 gdt 后还需要刷新 cs 和 tss
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}