use bitflags::*;
use cpuio::*;
use lazy_static::lazy_static;
use spin::Mutex;

bitflags! {
    pub struct IRQMask: u16 {
        const TIMER = 0b0000_0001;
        const PS_2  = 0b0000_0010;
        // ...
    }
}

fn io_wait() {
    unsafe { Port::new(0x80).write(0u8) }
}

struct Pic {
    master_cmd: Port<u8>,
    master_dt: Port<u8>,
    slave_cmd: Port<u8>,
    slave_dt: Port<u8>,
    imr: IRQMask,
}

impl Pic {
    fn set_imr(&mut self, imr: IRQMask) {
        let master_imr = imr.bits() as u8;
        let slave_imr = (imr.bits() >> 8) as u8;

        self.master_dt.write(!master_imr);
        self.slave_dt.write(!slave_imr);
        self.imr = imr;
    }

    fn init(&mut self, imr: IRQMask) {
        const ICW_INIT: u8 = 0x10;
        const ICW_IC4: u8 = 0x01;

        // ICW 1. Initialisation.
        self.master_cmd.write(ICW_INIT | ICW_IC4);
        self.slave_cmd.write(ICW_INIT | ICW_IC4);
        io_wait();

        // ICW 2. Map interrupts.
        self.master_dt.write(0x20);
        self.slave_dt.write(0x28);
        io_wait();

        // ICW 3. Connect slave with master.
        self.master_dt.write(0x4); // 0x4 -- IRQ2 pin
        self.slave_dt.write(0x4); // 0x4 -- IRQ2 pin address
        io_wait();

        // ICW4. final steps
        self.master_dt.write(0x1); // Enable PIC for 80x86 mode
        io_wait();

        self.set_imr(imr);
    }
}

lazy_static! {
    static ref PIC: Mutex<Pic> = Mutex::new(Pic {
        master_cmd: unsafe { Port::new(0x20) },
        master_dt: unsafe { Port::new(0x21) },
        slave_cmd: unsafe { Port::new(0xA0) },
        slave_dt: unsafe { Port::new(0xA1) },
        imr: IRQMask::empty(),
    });
}

pub fn init_interrupts(imr: IRQMask) {
    PIC.lock().init(imr)
}
