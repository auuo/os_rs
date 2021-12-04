#![no_std]
#![no_main]
#![feature(custom_test_frameworks)] // 使用自定义的测试框架
#![test_runner(os_rs::test_runner)] // 指定运行测试的函数
#![reexport_test_harness_main = "test_main"] // 将生成的测试入口函数名从 main 改为 test_main

use core::panic::PanicInfo;
use bootloader::BootInfo;
use bootloader::entry_point;

use os_rs::println;

/// panic 时会调用这个方法
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    // 要求返回 !
    os_rs::hlt_loop();
}

/// test 模式 panic 会调用这个方法
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os_rs::test_panic_handler(info)
}

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    println!("this is my printer");

    os_rs::init();

    #[cfg(test)]
    test_main();

    os_rs::hlt_loop();
}