#![no_std]
#![no_main]
#![feature(custom_test_frameworks)] // 使用自定义的测试框架
#![test_runner(os_rs::test_runner)] // 指定运行测试的函数
#![reexport_test_harness_main = "test_main"] // 将生成的测试入口函数名从 main 改为 test_main

extern crate alloc; // 对内置 crate 依赖

use alloc::boxed::Box;
use core::panic::PanicInfo;
use bootloader::BootInfo;
use bootloader::entry_point;
use x86_64::structures::paging::Page;

use os_rs::println;
use os_rs::task::executor::Executor;
use os_rs::task::simple_executor::SimpleExecutor;
use os_rs::task::Task;
use os_rs::task::keyboard;

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

// 使用 bootloader 的宏定义程序入口点，可进行类型检查，也不再需要 extern "C" 和 no_mangle 等配置
entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use x86_64::VirtAddr;
    use x86_64::structures::paging::Translate;
    use os_rs::allocator;

    println!("this is my printer");

    os_rs::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { os_rs::memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { os_rs::memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let x = Box::new(123);

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    #[cfg(test)]
    test_main();

    os_rs::hlt_loop();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}