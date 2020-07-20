use volatile::Volatile; // Required to avoid the compiler optimising stuff away
use core::fmt; // Required as we'll be using the write macros.
use lazy_static::lazy_static; // see Cargo.toml
use spin::Mutex; // see Cargo.toml;

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


    /// Basically this implements how we shuffle the
    /// buffer around in order to add a new line.
    ///
    /// The idea is that we first copy the characters in their respective
    /// rows one row up. Then we clear the last row of all
    /// the leftover characters.
    ///
    /// And finally reset the column position of the writer.
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    /// This method replaces all the characters int the last row
    /// with empty space characters.
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// Create a static writer when we need it for the first time.
// As we can't do it at compile time due to us dereferencing
// a raw pointer (???) and the `const evaluator` is not able to
// handle that.
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

// `Borrowed` from the definition of println
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
