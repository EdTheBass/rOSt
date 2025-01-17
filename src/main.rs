#![no_std] // no standard library
#![no_main]
#![allow(incomplete_features)]
#![feature(
    custom_test_frameworks,
    abi_x86_interrupt,
    generic_const_exprs,
    core_intrinsics,
    alloc_error_handler
)]
#![test_runner(test_framework::test_runner::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::{arch::asm, panic::PanicInfo};
use kernel::structures::kernel_information::KernelInformation;
use tinytga::RawTga;
use vga::vga_core::{Clearable, ImageDrawable};

use core::alloc::Layout;

entry_point!(kernel);
pub fn kernel(boot_info: &'static mut BootInfo) -> ! {
    let mut kernel_info = kernel::init(boot_info);
    bootup_sequence(kernel_info);

    #[cfg(test)]
    kernel_test(kernel_info);
    #[cfg(not(test))]
    kernel_main(&mut kernel_info);

    kernel::hlt_loop();
}

fn bootup_sequence(kernel_info: KernelInformation) {
    kernel::register_driver(vga::driver_init);
    kernel::register_driver(ata::driver_init);
    kernel::reload_drivers(kernel_info);
    let data = include_bytes!("./assets/rost-logo.tga");
    let logo = RawTga::from_slice(data).unwrap();
    let logo_header = logo.header();
    let mut vga_device = vga::vga_device::VGADeviceFactory::from_kernel_info(kernel_info);
    vga_device.clear(vga::vga_color::BLACK);
    vga_device.draw_image(
        (vga_device.width as u16 - logo_header.width) / 2,
        (vga_device.height as u16 - logo_header.height) / 2,
        &logo,
    );
}

#[no_mangle]
extern "C" fn user_mode_check() {
    unsafe {
        asm!("mov rdi, 0", "syscall", "mov rdi, 1", "syscall");
    }
    loop {}
}

pub fn kernel_main(kernel_info: &mut KernelInformation) {
    unsafe {
        kernel::run_in_user_mode(user_mode_check, kernel_info);
    }
    /*
        let test = Box::new(4);
        log_println!("New boxed value: {:#?}", test);
        log_println!("im not dying :)");
    */
    /*
        log_println!("Getting all disks...");
        let disks = ata::get_all_disks();
        log_println!("Got {} disks, taking the non-bootable one...", disks.len());
        let mut disk = disks
            .into_iter()
            .map(|mut disk| (disk.has_bootloader(), disk))
            .find(|(boot, _)| !boot.unwrap_or(true))
            .expect("No non-bootable disk found")
            .1;
        log_println!("Got a disk, looking for partitions...");
        let mut partitions = disk.get_partitions().expect("Error getting partitions");
        if partitions.len() == 0 {
            log_println!("No partitions found, creating a new one...");
            let partition_size = disk.descriptor.lba_48_addressable_sectors as u32 / 2;
            disk.create_partition(partition_size, 0xED)
                .expect("Error creating partition");
            log_println!("Partition created, double-checking...");
            partitions = disk.get_partitions().expect("Error getting partitions");
            if partitions.len() == 0 {
                log_println!("No partitions found, giving up.");
                return;
            }
        }
        log_println!("Found {} partitions:", partitions.len());
        for partition in partitions {
            log_println!(
                "{:8} - starting at {:8X}",
                format_size(partition.descriptor.sectors * 512),
                partition.descriptor.start_lba
            )
        }
    */
}

/// Panic handler for the OS.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::panic_handler(info);
}

/// This is the main function for tests.
#[cfg(test)]
pub fn kernel_test(_kernel_info: KernelInformation) {
    test_main();
}

/// Panic handler for the OS in test mode.
#[cfg(test)]
#[panic_handler]
// this function is called if a panic occurs and it is a test, all output is redirected to the serial port
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info);
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
