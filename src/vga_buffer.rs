use volatile::Volatile;
use core::fmt;
use spin::Mutex;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorCode(u8);

impl ColorCode {
    fn new(background: Color, foreground: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | foreground as u8)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_char: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}


pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8){
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line()
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_char: byte,
                    color_code: color_code,
                });
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let c = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(c);
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_char: b' ',
            color_code: self.color_code
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl Writer {
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20...0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }

        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::vga_buffer::kprint(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kprintln {
    () => (kprint!("\n"));
    ($fmt:expr) => (kprint!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (kprint!(concat!($fmt, "\n"), $($arg)*));
}

pub fn kprint(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[cfg(test)]
mod test {
    use super::*;

    fn construct_writer() -> Writer {
        use std::boxed::Box;

        let buffer = construct_buffer();
        Writer {
            column_position: 0,
            color_code: ColorCode::new(Color::Black, Color::White),
            buffer: Box::leak(Box::new(buffer)),
        }
    }

    fn construct_buffer() ->  Buffer {
        use array_init::array_init;

        Buffer {
            chars: array_init(|_| array_init(|_| Volatile::new(empty_char())))
        }
    }

    fn empty_char() -> ScreenChar {
        ScreenChar {
            ascii_char: b' ',
            color_code: ColorCode::new(Color::Black, Color::White),
        }
    }

    #[test]
    fn write_byte() {
        let mut writer = construct_writer();

        writer.write_byte(b'X');
        writer.write_byte(b'Y');

        for (i, row) in writer.buffer.chars.iter().enumerate() {
            for (j, screen_char) in row.iter().enumerate() {
                let screen_char = screen_char.read();

                if i == BUFFER_HEIGHT - 1 && j == 0 {
                    assert_eq!(screen_char.ascii_char, b'X');
                    assert_eq!(screen_char.color_code, writer.color_code);
                } else if i == BUFFER_HEIGHT - 1 && j == 1 {
                    assert_eq!(screen_char.ascii_char, b'Y');
                    assert_eq!(screen_char.color_code, writer.color_code);
                } else {
                    assert_eq!(screen_char, empty_char());
                }
            }
        }
    }

    #[test]
    fn write_formatted() {
        use std::fmt::Write;

        let mut writer = construct_writer();

        writeln!(&mut writer, "a").unwrap();
        writeln!(&mut writer, "b{}", "c").unwrap();

        for (i, row) in writer.buffer.chars.iter().enumerate() {
            for (j, screen_char) in row.iter().enumerate() {
                let screen_char = screen_char.read();

                if i == BUFFER_HEIGHT - 3 && j == 0 {
                    assert_eq!(screen_char.ascii_char, b'a');
                    assert_eq!(screen_char.color_code, writer.color_code);
                } else if i == BUFFER_HEIGHT - 2 && j == 0 {
                    assert_eq!(screen_char.ascii_char, b'b');
                    assert_eq!(screen_char.color_code, writer.color_code);
                } else if i == BUFFER_HEIGHT - 2 && j == 1 {
                    assert_eq!(screen_char.ascii_char, b'c');
                    assert_eq!(screen_char.color_code, writer.color_code);
                } else if i == BUFFER_HEIGHT - 1 && j == 0 {
                    assert_eq!(screen_char.ascii_char, b' ');
                    assert_eq!(screen_char.color_code, writer.color_code);
                } else {
                    assert_eq!(screen_char, empty_char());
                }
            }
        }
    }

    #[test]
    fn change_color() {
        use std::fmt::Write;

        let mut writer = construct_writer();
        let original_color_code = writer.color_code;
        let changed_color_code = ColorCode::new(Color::Black, Color::Black);

        writeln!(&mut writer, "a").unwrap();
        writer.color_code = changed_color_code;
        writeln!(&mut writer, "b").unwrap();

        for (i, row) in writer.buffer.chars.iter().enumerate() {
            for (j, screen_char) in row.iter().enumerate() {
                let screen_char = screen_char.read();

                if i == BUFFER_HEIGHT - 3 && j == 0 {
                    assert_eq!(screen_char.ascii_char, b'a');
                    assert_eq!(screen_char.color_code, original_color_code);
                } else if i == BUFFER_HEIGHT - 2 && j == 0 {
                    assert_eq!(screen_char.ascii_char, b'b');
                    assert_eq!(screen_char.color_code, changed_color_code);
                } else if i == BUFFER_HEIGHT - 1 && j == 0 {
                    assert_eq!(screen_char.ascii_char, b' ');
                    assert_eq!(screen_char.color_code, changed_color_code);
                }
            }
        }
    }
}











