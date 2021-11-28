use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::gdt;
use crate::print;
use crate::println;

pub const PIC_1_OFFSET: u8 = 32; // primary 控制器的开始中断号
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8; // secondary 控制器的开始中断号

/// PIC line 的下标，架构如下
///                      ____________                          ____________
/// Real Time Clock --> |            |   Timer -------------> |            |
/// ACPI -------------> |            |   Keyboard-----------> |            |      _____
/// Available --------> | Secondary  |----------------------> | Primary    |     |     |
/// Available --------> | Interrupt  |   Serial Port 2 -----> | Interrupt  |---> | CPU |
/// Mouse ------------> | Controller |   Serial Port 1 -----> | Controller |     |_____|
/// Co-Processor -----> |            |   Parallel Port 2/3 -> |            |
/// Primary ATA ------> |            |   Floppy disk -------> |            |
/// Secondary ATA ----> |____________|   Parallel Port 1----> |____________|
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    KeyBoard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

/// 8259 可编程中断控制器
pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

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
        // 中断控制器使用下标的方式访问到 line
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::KeyBoard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
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

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!(".");

    // 需要明确的告诉 pic 该中断处理完了，这样 pic 才会触发下一个中断
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use x86_64::instructions::port::Port;

    lazy_static! {
        // 定义键盘
        static ref KEYBOARD: spin::Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = spin::Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore));
    }

    let mut keyboard = KEYBOARD.lock();
    // ps/2 键盘的数据端口，按下和抬起都是不同的 code
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::KeyBoard.as_u8());
    }
}

// 测试断点异常处理，主动触发断点异常，并且正常返回
#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}