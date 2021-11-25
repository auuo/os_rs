#![no_std]
#![no_main]

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
    panic!("i panic");
}