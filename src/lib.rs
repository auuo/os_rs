#![no_std] // lib 是独立的单元，也需要指定
#![cfg_attr(test, no_main)] // 不知道这个的含义
#![feature(custom_test_frameworks)] // 使用自定义框架
#![test_runner(test_runner)] // 收集可测试函数后会调用这个函数
#![reexport_test_harness_main = "test_main"] // 将生成的测试入口函数名从 main 改为 test_main

use core::panic::PanicInfo;

pub mod serial;
pub mod vga_buffer;

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

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

/// test 模式程序入口点. no_mangle 避免 _start 函数名被重写
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
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
