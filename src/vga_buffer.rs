use volatile::Volatile; // Required to avoid the compiler optimising stuff away
use core::fmt; // Required as we'll be using the write macros.

/// Allowed colors that VGA can handle
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


/// Transparent here lets us basically use
/// the struct as a type checking thing
/// and it would still be stored as a u8.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

/// Color code creation from a set of background and foreground colors.
impl ColorCode {

    /// If each part of the foreground and background are u4 but we can't
    /// use the u4 as there is no rust primitive for that. We have to use
    /// u8 and shift it to the left and use a bitwise or.
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// This represents a character on screen.
/// The memory layout for this is as follows:
///
/// Bit(s) Value
/// 0-7    ASCII code point
/// 8-11   Foreground color
/// 12-14  Background color
/// 15     Blink
///
/// Since we use repr(C), I would expect that it means
/// the Struct in the ABI is represented in the order
/// that fields appear. Meaning we get first 8 bits as
/// ASCII (ish) character. And the next 8 bits as the
/// foreground and background colors.
///
///
/// Interesting that the background color has less space to
/// wiggle - only 3 bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;


/// This is basically just saying - we have a bunch of transparent
/// structs (newtypes) and we need to store them in a buffer.
///
/// This Buffer struct is still something thats transparent. So
/// the actual chars field is just a representation of the structs
/// (transparent structs) we made before.
///
///
/// We use volatile here because the optimisation the compiler could do
/// to our code COULD decide that this write is pointless as we're never
/// reading the buffer and it doesn't understand that there is a visible
/// side effect happening from the writes we'll be doing. In that case
/// it could just remove the writes - ignoring our instructions. We want
/// to force it to keep this in and thats what the volatile struct does.
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// This is the actual writer that keeps the state of what we're doing
/// to the VGA buffer.
///
/// TODO: Clear pre next-write
pub struct Writer {
    /// Where we are in the row
    column_position: usize,
    /// One color code for the whole buffer
    color_code: ColorCode,
    /// A reference (static) to a mutable buffer.
    /// This is static as we'd expect as the VGA
    /// Buffer is valid for the whole duration of
    /// our program. The way we make this buffer
    /// in practice is through casting a pointer
    /// and dereferencing it (Unsafe)
    buffer: &'static mut Buffer,
}

impl Writer {

    /// Turns strings into bytes and then writes them
    /// one by one.
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // There is a range of hex values
                // that represent the possible
                // characters that VGA can display.
                //
                // We check here for those.
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // We place a character as a placeholder if
                // we find a byte outside of the range.
                _ => self.write_byte(0xfe),
            }

        }
    }

    /// Writes the bytes to the buffer in memory.
    ///
    /// When it matches a `\n` character, it should
    /// know how to handle that - aka go to next row.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                // In order to write to the volatile wrapper
                // we need to use the write method.
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }


    fn new_line(&mut self) {/* TODO */}
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub fn print_something() {
    use core::fmt::Write;
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    write!(writer, "The numbers are {}, {}", 10 * 100, 1/100).unwrap();
}
