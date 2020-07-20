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
    println!("Hello World{}", "!");
    panic!("Some info");
}

/// This function is called on panic.
///
/// We use the abort strategy - we don't do unwinding.
///
/// After making the println! macro we added the printout
/// of the panic info.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
