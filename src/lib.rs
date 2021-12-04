#![no_std] // lib 是独立的单元，也需要指定
#![cfg_attr(test, no_main)] // 不知道这个的含义
#![feature(custom_test_frameworks)] // 使用自定义框架
#![test_runner(test_runner)] // 收集可测试函数后会调用这个函数
#![reexport_test_harness_main = "test_main"] // 将生成的测试入口函数名从 main 改为 test_main
#![feature(abi_x86_interrupt)] // 设置中断向量表需要遵循 x86 的调用规范

use core::panic::PanicInfo;
#[cfg(test)]
use bootloader::{entry_point, BootInfo};

pub mod serial;
pub mod vga_buffer;
pub mod interrupts;
pub mod gdt;
pub mod memory;

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable(); // 开启中断
}

pub trait Testable {
    fn run(&self);
}

// 为每个测试打印函数名
impl<T> Testable for T where T: Fn() {
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

/// rust 会收集测试调用这个函数
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

/// test 模式使用这个函数处理 panic，输出到串口
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

// 使用 bootloader 的宏定义程序入口点，可进行类型检查，也不再需要 extern "C" 和 no_mangle 等配置
#[cfg(test)]
entry_point!(test_kernel_main);

/// test 模式程序入口点.
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

/// test 模式 panic 会调用这个方法
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

/// qemu isa-debug-exit 设备退出码，退出码计算方式：(value << 1) | 1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

/// 使用 qemu 提供的端口映射的硬件进行退出
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4); // cargo.toml 中提供的 isa-debug-exit 设备端口号
        port.write(exit_code as u32);
    }
}

/// 死循环，使用 hlt 指令休眠
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt(); // hlt 指令休眠直到下一个中断到来
    }
}