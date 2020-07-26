#![no_std]
#![cfg_attr(test, no_main)] // Define that we have no main and test
#![feature(custom_test_frameworks)] // Allow custom testing framework interface
#![feature(abi_x86_interrupt)] // Required as the extern x86_interrupt convention is unstable
#![test_runner(crate::test_runner)] // Define what runs a test
#![reexport_test_harness_main = "test_main"] // Avoid name clashes with normal main for the runner

// Unsure why we import this
extern crate rlibc;

// Required for panic handling
use core::panic::PanicInfo;

pub mod gdt;
pub mod interrupts;
pub mod serial;
pub mod vga_buffer;

pub fn init() {
    gdt::init(); // initialise the global descriptor table
    interrupts::init_idt(); // interrupt descriptor table
}

// Define a more explicit type for testing
pub trait Testable {
    fn run(&self);
}

// Implement the testable for functions
// that we need to test.
impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

// This is what handles the tests being run.
pub fn test_runner(tests: &[&dyn Testable]) {
    // Prints to the original caller of the qemu test
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

// Panic handler for the case where tests fail.
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

// Exit codes for QEMU - required for smoother testing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

// We write out to the exit device specified in the `test-args`
// in Cargo.toml
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

/// Entry point for `cargo xtest`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
