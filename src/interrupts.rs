use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::gdt;
use crate::println;

lazy_static! {
    /// 初始化中断向量表，必须是 static 的
    static ref IDT: InterruptDescriptorTable = {
        // 使用 x86_64 crate 提供的数据结构
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            // 为 double fault 设置单独的栈，避免 hit guard page 导致 triple fault
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

pub fn init_idt() {
    IDT.load(); // 使用 lidt 指令注册该表的地址
}

/// debug 断点的中断处理
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

// double fault 的 error_code 永远是 0
extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

// 测试断点异常处理，主动触发断点异常，并且正常返回
#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}