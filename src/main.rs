#![no_std]
#![no_main]
#![feature(custom_test_frameworks)] // 使用自定义的测试框架
#![test_runner(os_rs::test_runner)] // 指定运行测试的函数
#![reexport_test_harness_main = "test_main"] // 将生成的测试入口函数名从 main 改为 test_main

use core::panic::PanicInfo;

use os_rs::println;

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
    os_rs::test_panic_handler(info)
}

/// 程序入口点. no_mangle 避免 _start 函数名被重写
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("this is my printer");

    os_rs::init();

    // 触发 breakpoint 中断用于测试
    x86_64::instructions::interrupts::int3();

    #[cfg(test)]
    test_main();

    loop {}
}