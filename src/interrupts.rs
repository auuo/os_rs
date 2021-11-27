use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::println;

lazy_static! {
    /// 初始化中断向量表，必须是 static 的
    static ref IDT: InterruptDescriptorTable = {
        // 使用 x86_64 crate 提供的数据结构
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
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