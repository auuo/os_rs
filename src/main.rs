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

/// rust 会收集测试调用这个函数
#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion...");
    assert_eq!(1, 1);
    println!("[ok]");
}