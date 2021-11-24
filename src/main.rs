#![no_std]
#![no_main]

use core::panic::PanicInfo;

/// panic 时会调用这个方法
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // 要求返回 !
    loop {}
}

static HELLO: &[u8] = b"Hello World";

/// 程序入口点. no_mangle 避免 _start 函数名被重写
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }
    loop {}
}