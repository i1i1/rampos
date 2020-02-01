#![feature(abi_x86_interrupt, asm)]
#![no_main]
#![no_std]

pub mod com;
pub mod interrupt;
pub mod pic;
pub mod vga;
use {
    bootloader::{bootinfo::*, BootInfo},
    core::panic::PanicInfo,
};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    vga_println!("{}", info);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    vga_println!("Memory map:");

    for MemoryRegion { range, region_type } in boot_info.memory_map.iter() {
        let (start, end) = (range.start_frame_number, range.end_frame_number);
        vga_println!("\t{:08x?} {:?}", (start, end), region_type);
    }

    pic::init_interrupts(pic::IRQMask::empty());
    interrupt::init();
    vga_println!("Interrupts and pic inited!");
    loop {}
}
