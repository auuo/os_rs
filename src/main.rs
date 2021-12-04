#![no_std]
#![no_main]
#![feature(custom_test_frameworks)] // 使用自定义的测试框架
#![test_runner(os_rs::test_runner)] // 指定运行测试的函数
#![reexport_test_harness_main = "test_main"] // 将生成的测试入口函数名从 main 改为 test_main

use core::panic::PanicInfo;
use bootloader::BootInfo;
use bootloader::entry_point;
use x86_64::structures::paging::PageTable;

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

// 使用 bootloader 的宏定义程序入口点，可进行类型检查，也不再需要 extern "C" 和 no_mangle 等配置
entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use os_rs::memory::active_level_4_table;
    use x86_64::VirtAddr;

    println!("this is my printer");

    os_rs::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // let l4_table = unsafe { active_level_4_table(phys_mem_offset) };
    //
    // for (i, entry) in l4_table.iter().enumerate() {
    //     if !entry.is_unused() {
    //         println!("L4 Entry {}: {:?}", i, entry);
    //
    //         // 页表项指向的物理地址，3 级页表
    //         let phys = entry.frame().unwrap().start_address();
    //         let virt = phys.as_u64() + boot_info.physical_memory_offset;
    //         let ptr = VirtAddr::new(virt).as_ptr();
    //         let l3_table: &PageTable = unsafe { &*ptr };
    //
    //         for (i, entry) in l3_table.iter().enumerate() {
    //             if !entry.is_unused() {
    //                 println!("L3 Entry {}: {:?}", i, entry);
    //             }
    //         }
    //     }
    // }

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = unsafe { os_rs::memory::translate_addr(virt, phys_mem_offset) };
        println!("{:?} -> {:?}", virt, phys);
    }

    #[cfg(test)]
    test_main();

    os_rs::hlt_loop();
}