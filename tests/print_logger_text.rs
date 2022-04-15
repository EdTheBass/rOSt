#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os_core::test_framework::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use bootloader::{BootInfo, entry_point};
use os_core::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os_core::test_panic_handler(info)
}

entry_point!(kernel_start);
#[no_mangle]
pub fn kernel_start(_boot_info: &'static mut BootInfo) -> ! {
    os_core::init(_boot_info.framebuffer.as_mut().take().unwrap());
    test_main();
    loop {}
}

#[test_case]
pub fn simple_print_test() {
    println!("Hello, World!");
}

#[test_case]
pub fn bulk_print_test() {
    for _ in 0..250 {
        println!("Hello, World!");
    }
}
