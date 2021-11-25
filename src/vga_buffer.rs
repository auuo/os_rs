use volatile::Volatile;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    // 使用 spin 自旋锁提供可变形
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Red, Color::White),
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

const BUFFER_HEIGHT: usize = 25;
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
                0x20...0x7e | b'\n' => self.write_byte(b),
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
