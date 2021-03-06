use {
    core::fmt::{Arguments, Error, Write},
    cpuio::*,
    lazy_static::lazy_static,
    spin::Mutex,
};

pub struct COM {
    base: u16,
}

impl Write for COM {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for byte in s.bytes() {
            self.write_byte(byte)
        }
        Ok(())
    }
}

impl COM {
    fn port(&mut self, port: u16) -> Port<u8> {
        assert!(port < 6);
        unsafe { Port::new(self.base + port) }
    }

    fn trans_empty(&mut self) -> bool {
        self.port(5).read() & 0x20 == 0
    }

    fn fifo_empty(&mut self) -> bool {
        self.port(5).read() & 0x01 == 0
    }

    fn write_byte(&mut self, byte: u8) {
        while self.trans_empty() {}

        self.port(0).write(byte)
    }

    fn read_byte(&mut self) -> u8 {
        while self.fifo_empty() {}

        self.port(0).read()
    }

    // TODO: probably where is some nice `std::` trait representing this
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        for i in 0..buf.len() {
            buf[i] = self.read_byte();
        }
        buf.len()
    }

    pub unsafe fn new(base: u16) -> Self {
        let mut ret = COM { base };

        // Disable DLAB
        ret.port(0).write(0x00);
        // Disable all interrupts
        ret.port(1).write(0x00);
        // Enable DLAB (set baud rate divisor)
        ret.port(3).write(0x80);
        // Set divisor to 1 (115200 baud)
        ret.port(0).write(0x01);
        ret.port(1).write(0x00);
        // Disable DLAB, 7 bits, no parity, one stop bit
        ret.port(3).write(0x02);
        // Enable FIFO, clear FIFO, with 14 byte threshold
        ret.port(2).write(0xc7);
        // IRQs enabled, RTS and DTR set
        ret.port(4).write(0x0B);

        ret
    }
}

lazy_static! {
    pub static ref COM1: Mutex<COM> = Mutex::new(unsafe { COM::new(0x3f8) });
}

#[macro_export]
macro_rules! com_print {
    ($($arg:tt)*) => ($crate::com::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! com_println {
    () => ($crate::com_print!("\n"));
    ($($arg:tt)*) => ($crate::com_print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
    COM1.lock().write_fmt(args).unwrap();
}
