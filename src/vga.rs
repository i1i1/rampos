use core::fmt::{Arguments, Error, Write};
use lazy_static::lazy_static;
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
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(fg: Color, bg: Color) -> ColorCode {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
}

type Ascii = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar(Ascii, ColorCode);

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

type BufferRow = [ScreenChar; BUFFER_WIDTH];
type Buffer = [BufferRow; BUFFER_HEIGHT];

pub struct VGAWriter {
    row: usize,
    column: usize,
    color: ColorCode,
    buffer: &'static mut Buffer,
}

impl Write for VGAWriter {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
        Ok(())
    }
}

impl VGAWriter {
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column >= BUFFER_WIDTH {
                    self.new_line();
                }

                // Same code but volatile:
                // ```
                // let mut ref = self.buffer[self.row][self.column];
                // *ref = ScreenChar(byte, self.color);
                // ```
                unsafe {
                    let refer = &mut self.buffer[self.row][self.column];
                    let ptr = refer as *mut ScreenChar;
                    ptr.write_volatile(ScreenChar(byte, self.color));
                }
                self.column += 1;
            }
        }
    }

    fn new_line(&mut self) {
        if self.row < BUFFER_HEIGHT - 1 {
            self.row += 1;
            self.column = 0;
            return;
        }

        for i in 0..BUFFER_HEIGHT - 1 {
            // `self.buffer[i] = self.buffer[i + 1]`
            unsafe {
                let ptr_w = &mut self.buffer[i] as *mut BufferRow;
                let ptr_r = &mut self.buffer[i + 1] as *mut BufferRow;
                ptr_w.write_volatile(ptr_r.read_volatile());
            }
        }
        // ```
        // let chr = ScreenChar(b' ', self.color);
        // self.buffer[BUFFER_HEIGHT - 1] = [chr; BUFFER_WIDTH];
        // ```
        unsafe {
            let ptr = &mut self.buffer[BUFFER_HEIGHT - 1] as *mut BufferRow;
            ptr.write_volatile([ScreenChar(b' ', self.color); BUFFER_WIDTH]);
        }
        self.column = 0;
    }
}

lazy_static! {
    pub static ref VGAWRITER: Mutex<VGAWriter> = Mutex::new(VGAWriter {
        row: 0,
        column: 0,
        color: ColorCode::new(Color::Green, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
    VGAWRITER.lock().write_fmt(args).unwrap();
}
