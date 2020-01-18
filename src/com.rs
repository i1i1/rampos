use core::fmt::{Arguments, Error, Write};
use cpuio::*;
use lazy_static::lazy_static;
use spin::Mutex;

pub struct COM {
    ports: [Port<u8>; 6],
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
    fn trans_empty(&mut self) -> bool {
        (self.ports[5].read() & 0x20) == 0
    }

    fn fifo_empty(&mut self) -> bool {
        (self.ports[5].read() & 0x01) == 0
    }

    fn write_byte(&mut self, byte: u8) {
        while self.trans_empty() {}

        self.ports[0].write(byte)
    }

    fn read_byte(&mut self) -> u8 {
        while self.fifo_empty() {}

        self.ports[0].read()
    }

    fn read(&mut self, buf: &mut [u8]) -> usize {
        for i in 0..buf.len() {
            buf[i] = self.read_byte();
        }
        buf.len()
    }

    unsafe fn new(base_port: u16) -> Self {
        let mut ports = [
            Port::new(base_port),
            Port::new(base_port + 1),
            Port::new(base_port + 2),
            Port::new(base_port + 3),
            Port::new(base_port + 4),
            Port::new(base_port + 5),
        ];

        // Disable DLAB
        ports[0].write(0x00);
        // Disable all interrupts
        ports[1].write(0x00);
        // Enable DLAB (set baud rate divisor)
        ports[3].write(0x80);
        // Set divisor to 1 (115200 baud)
        ports[0].write(0x01);
        ports[1].write(0x00);
        // Disable DLAB, 7 bits, no parity, one stop bit
        ports[3].write(0x02);
        // Enable FIFO, clear FIFO, with 14 byte threshold
        ports[2].write(0xc7);
        // IRQs enabled, RTS and DTR set
        ports[4].write(0x0B);

        COM { ports: ports }
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
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::com_print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
    COM1.lock().write_fmt(args).unwrap();
}
