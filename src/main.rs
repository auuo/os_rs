#![no_std]
#![no_main]

#![feature(custom_test_frameworks)] // 使用自定义的测试框架
#![test_runner(crate::test_runner)] // 指定运行测试的函数
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

mod vga_buffer;

/// panic 时会调用这个方法
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    // 要求返回 !
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
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion...");
    assert_eq!(1, 1);
    println!("[ok]");
}