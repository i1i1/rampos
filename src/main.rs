#![feature(abi_x86_interrupt, custom_test_frameworks)]
#![no_main]
#![no_std]
#![reexport_test_harness_main = "test_main"]
#![test_runner(crate::test_runner)]

mod com;
mod pic;
mod vga;
use bootloader::BootInfo;
use core::panic::PanicInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use cpuio::Port;
    let mut port = unsafe { Port::new(0xf4) };
    port.write(exit_code as u32);
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    com_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
    com_print!("trivial assertion... ");
    assert_eq!(1, 1);
    com_println!("[ok]");
}

#[test_case]
fn test_vga_simple() {
    com_print!("test_println... ");
    vga_println!("test_vga_simple output");
    com_println!("[ok]");
}

#[test_case]
fn test_println_many() {
    com_print!("test_println_many... ");
    for _ in 0..200 {
        vga_println!("test_println_many output");
    }
    com_println!("[ok]");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    vga_println!("{}", info);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start(_boot_info: &'static BootInfo) -> ! {
    #[cfg(test)]
    test_main();

    pic::init_interrupts(pic::IRQMask::empty());
    vga_println!("Pic inited!");
    loop {}
}
