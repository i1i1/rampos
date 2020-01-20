#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(crate::test_runner)]

mod com;
mod pic;
mod vga;
use core::panic::PanicInfo;

#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    let mut exit_port = unsafe { cpuio::Port::new(0xf4) };
    exit_port.write(exit_code as u32);
}

pub fn exit_qemu_success() {
    exit_qemu(QemuExitCode::Success)
}

#[cfg(test)]
fn test_runner(tests: &[&test_types::UnitTest]) {
    com_println!("Running {} tests", tests.len());
    com_println!("{:->58}", "----");

    for test in tests {
        com_print!("\t{: <50}", test.name);
        (test.test_func)();
        com_println!("[ok]");
    }

    exit_qemu_success()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    com_println!("{}", info);
    loop {}
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    #[cfg(test)]
    test_main();
    loop {}
}
