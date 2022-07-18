use core::fmt::{Result, Write};

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
    White = 15
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }

    pub fn set_fg_color(&mut self, color: Color) {
        self.0 = self.0 & 0xf0 | color as u8;
    }

    pub fn set_bg_color(&mut self, color: Color) {
        self.0 = (color as u8) << 4 | self.0 & 0x0f;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

#[derive(Debug)]
pub struct Writer {
    buffer: &'static mut [ScreenChar],
    num_columns: usize,
    pos: usize,
    color_code: ColorCode,
}

impl Writer {
    pub fn new(buffer: &'static mut [ScreenChar], num_lines: usize, num_columns: usize) -> Self {
        debug_assert_eq!(buffer.len(), num_lines * num_columns);

        Self {
            num_columns,
            pos: 0,
            buffer,
            color_code: ColorCode::new(Color::White, Color::Black)
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        debug_assert!(self.pos <= self.buffer.len());

        match byte {
            b'\n' => self.new_line(),
            byte => {
                self.buffer[self.pos] = ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                };
                self.pos += 1;
            }
        }

        if self.pos == self.buffer.len() {
            self.scroll();
        }
    }

    fn new_line(&mut self) {
        debug_assert!(self.pos <= self.buffer.len());

        if self.pos >= self.buffer.len() - self.num_columns {
            self.scroll();
            return;
        }

        self.pos = (self.pos - self.pos % self.num_columns) + self.num_columns;
    }

    fn scroll(&mut self) {
        debug_assert!(self.pos <= self.buffer.len());

        self.buffer.copy_within(self.num_columns..self.buffer.len(), 0);

        let new_cursor_pos = self.buffer.len() - self.num_columns;

        self.buffer[new_cursor_pos..].fill(ScreenChar {
            ascii_character: b' ',
            color_code: ColorCode::new(Color::White, Color::Black)
        });

        self.pos = new_cursor_pos;
    }

    pub fn clear(&mut self) {
        debug_assert!(self.pos <= self.buffer.len());

        self.buffer.fill(ScreenChar {
            ascii_character: b' ',
            color_code: ColorCode::new(Color::White, Color::Black)
        });

        self.pos = 0;
    }

    pub fn color_code(&self) -> ColorCode {
        self.color_code
    }

    pub fn set_color_code(&mut self, color: ColorCode) {
        self.color_code = color;
    }

    pub fn set_fg_color(&mut self, color: Color) {
        self.color_code.set_fg_color(color)
    }

    pub fn set_bg_color(&mut self, color: Color) {
        self.color_code.set_bg_color(color)
    }
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}
