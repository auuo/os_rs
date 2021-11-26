#![no_std]
#![no_main]

#![feature(custom_test_frameworks)] // 使用自定义的测试框架
#![test_runner(crate::test_runner)] // 指定运行测试的函数
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

mod vga_buffer;
mod serial;

/// panic 时会调用这个方法
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    // 要求返回 !
    loop {}
}

/// test 模式 panic 会调用这个方法
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

/// 程序入口点. no_mangle 避免 _start 函数名被重写
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("this is my printer");

    #[cfg(test)]
    test_main();

    loop {}
}

/// qemu isa-debug-exit 设备退出码，退出码计算方式：(value << 1) | 1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4); // cargo.toml 中提供的 isa-debug-exit 设备端口号
        port.write(exit_code as u32);
    }
}

/// rust 会收集测试调用这个函数
#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T where T: Fn() {
    fn run(&self) -> () {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}