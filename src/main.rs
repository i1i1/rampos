#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

mod pic;
mod vga;
use bootloader::BootInfo;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start(_boot_info: &'static BootInfo) -> ! {
    pic::init_interrupts(pic::IRQMask::empty());
    println!("Pic inited!");
    loop {}
}
