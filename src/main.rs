#![no_std] // Don't link the Rust standard library
#![no_main] // Disable all Rust-level entry points (main as it needs a C runtime setup)

// ??
extern crate rlibc;

mod vga_buffer; // Allows us to write to the VGA buffer.

use core::panic::PanicInfo; // A type used for the panic handler

// Avoid renaming our function (no unique name) as it is required by the
// boot process.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    vga_buffer::print_something();
    loop {}
}

/// This function is called on panic.
///
/// We use the abort strategy - we don't do unwinding.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
