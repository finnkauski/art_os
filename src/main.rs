#![no_std] // Don't link the Rust standard library
#![no_main] // Disable all Rust-level entry points (main as it needs a C runtime setup)
#![feature(custom_test_frameworks)] // Required for custom test frameworks
#![test_runner(blog_os::test_runner)] // Define the test running funtion
#![reexport_test_harness_main = "test_main"] // Avoid name clashes

use blog_os::println;
use core::panic::PanicInfo; // Required as we need to get deets on the panic.

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    blog_os::init();

    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();

    #[cfg(test)]
    test_main();

    loop {}
}

/// This function is called on panic.
///
/// We use the abort strategy - we don't do unwinding.
///
/// After making the println! macro we added the printout
/// of the panic info.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

// Called when we're testing as we want to close out our
// panics by closing the emulator etc. Basically different
// procedure from the cases where we deploy our kernel to
// hardware.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

// Some test case.
#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
