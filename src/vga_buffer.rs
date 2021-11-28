use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

lazy_static! {
    // 使用 spin 自旋锁提供安全的内部可变性
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] // 使用 u8 的格式存储
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// vga 颜色码，一个字节大小，前 4 位为背景色，后 4 位为前景色
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // 使用和内容(u8)一样的内存布局
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }
}

/// vga 字符单元，一个字节的 ascii 码，一个字节的颜色
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // 要求与 c 语言相同保证字段顺序和布局
struct ScreenChar {
    ascii_char: u8,
    color_code: ColorCode,
}

/// vga 缓冲区的长度
const BUFFER_HEIGHT: usize = 25;
/// vga 缓冲区的宽度
const BUFFER_WIDTH: usize = 80;

/// vga 字符缓冲区数据结构，会将 vga 地址直接转到此结构
#[repr(transparent)] // 要求和它唯一的变量内存布局相同
struct Buffer {
    // 使用 Volatile 包装，保证 read 和 write 不被编译器优化掉。因为识别到只读或只写可能会被认为无效操作
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    // 光标所在列的位置，不用记录行位置是因为我们总写到最后一行，若写满则所有数据向上移动一行
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_string(&mut self, s: &str) {
        for b in s.bytes() {
            match b {
                // 可打印的 ascii 字符
                0x20..=0x7e | b'\n' => self.write_byte(b),
                // 不可打印的字符，显示一个方块的符号
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1; // 永远写到屏幕最下一行
                let col = self.column_position;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_char: byte,
                    color_code: self.color_code,
                });
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        // 整体向上移动一行
        for i in 0..BUFFER_HEIGHT - 1 {
            for j in 0..BUFFER_WIDTH {
                self.buffer.chars[i][j].write(self.buffer.chars[i + 1][j].read())
            }
        }
        // 最后一行清空
        for i in 0..self.buffer.chars[BUFFER_HEIGHT - 1].len() {
            self.buffer.chars[BUFFER_HEIGHT - 1][i].write(ScreenChar {
                ascii_char: b' ',
                color_code: self.color_code,
            });
        }
        self.column_position = 0
    }
}

// 实现后可使用 write! 宏打印
impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*))); // 使用 _print 方法输出
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n")); // 无参数时打印换行
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*))); // 使用 print! 宏输出
}

#[doc(hidden)] // 需要 pub 但不需要生成文档
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // 避免硬件中断死锁
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap(); // 不会 panic
    });
}

// 测试没有 panic
#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

// 测试打印超过屏幕的行数
#[test_case]
fn test_println_many() {
    for i in 0..200 {
        println!("test_println_many output {}", i);
    }
}

// 测试字符是否输出到缓冲区
#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;;

    let s = "Some test string that fits on a single line";
    // 避免被硬件中断进入死锁
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_char), c);
        }
    });
}