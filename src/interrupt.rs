use core::mem;
use lazy_static::lazy_static;
use modular_bitfield::prelude::*;
use spin::Mutex;

#[repr(transparent)]
struct Pointer(u32);

/**
 *  31                23                15                7               0
 * ╔═════════════════╪═════════════════╪═════════════════╪═════════════════╗
 * ║                                 BASE                                  ║2
 * ╚═════════════════╪═════════════════╦═════════════════╪═════════════════╣
 *                                     ║               LIMIT               ║0
 *                                     ╚═════════════════╪═════════════════╝
 */
#[repr(packed)]
struct Idt {
    limit: u16,
    base: Pointer,
}

/**
 *                              80386 TASK GATE
 * 31                23                15                7                0
 * ╔═════════════════╪═════════════════╪═══╤═══╤═════════╪═════════════════╗
 * ║▒▒▒▒▒▒▒▒▒▒▒▒▒(NOT USED)▒▒▒▒▒▒▒▒▒▒▒▒│ P │DPL│0 0 1 0 1│▒▒▒(NOT USED)▒▒▒▒║4
 * ╟───────────────────────────────────┼───┴───┴─────────┴─────────────────╢
 * ║             SELECTOR              │▒▒▒▒▒▒▒▒▒▒▒▒▒(NOT USED)▒▒▒▒▒▒▒▒▒▒▒▒║0
 * ╚═════════════════╪═════════════════╪═════════════════╪═════════════════╝
 *
 *                               80386 INTERRUPT GATE
 * 31                23                15                7                0
 * ╔═════════════════╪═════════════════╪═══╤═══╤═════════╪═════╪═══════════╗
 * ║           OFFSET 31..16           │ P │DPL│0 1 1 1 0│0 0 0│(NOT USED) ║4
 * ╟───────────────────────────────────┼───┴───┴─────────┴─────┴───────────╢
 * ║             SELECTOR              │           OFFSET 15..0            ║0
 * ╚═════════════════╪═════════════════╪═════════════════╪═════════════════╝
 *
 *                               80386 TRAP GATE
 * 31                23                15                7                0
 * ╔═════════════════╪═════════════════╪═══╤═══╤═════════╪═════╪═══════════╗
 * ║          OFFSET 31..16            │ P │DPL│0 1 1 1 1│0 0 0│(NOT USED) ║4
 * ╟───────────────────────────────────┼───┴───┴─────────┴─────┴───────────╢
 * ║             SELECTOR              │           OFFSET 15..0            ║0
 * ╚═════════════════╪═════════════════╪═════════════════╪═════════════════╝
 */
#[bitfield]
#[derive(Clone, Copy)]
struct Gate {
    offset_low: B16,
    selector: B16,
    _res: B5,
    gtype: B8,
    /// Descriptor Privilege Level
    dpl: B2,
    present: bool,
    offset_high: B16,
}

enum GateType {
    Task = 0b00101_000,
    Trap = 0b01110_000,
    Interrupt = 0b01111_000,
}

impl Gate {
    fn create(gtype: GateType, segm: u16, dpl: u8, handler: &fn()) -> Gate {
        let mut ret = Gate::new();
        let handler = (handler as *const fn()) as u32;

        ret.set_selector(segm);
        ret.set_gtype(gtype as u8);
        ret.set_dpl(dpl);
        ret.set_offset_low(handler as u16);
        ret.set_offset_high(((handler as u32) >> 16) as u16);

        ret
    }
}

unsafe fn load_idt(idt: &'static Idt) {
    asm!(
        "movl 0x4(%esp), %eax
         lidt (%eax)
         sti"
         :: "{eax}"(idt)
    )
}

lazy_static! {
    static ref INTERRUPTS: Mutex<[Gate; 256]> = Mutex::new([Gate::new(); 256]);
    static ref IDT: Idt = Idt {
        limit: 256 * 8 - 1,
        base: Pointer(&INTERRUPTS as *const INTERRUPTS as u32),
    };
}

fn add_handler(num: u8, gtype: GateType, handler: &fn()) {
    // Assuming that segment 1 is executable
    let (segm, dpl) = (1, 0);

    INTERRUPTS.lock()[num as usize] = Gate::create(gtype, segm, dpl, handler);
}

#[inline(never)]
fn def_handler() {
    panic!("Something went wrong... Triggered interrupt")
}

pub fn init() {
    for i in 0..=255 {
        add_handler(i, GateType::Interrupt, &(def_handler as fn()))
    }

    unsafe { load_idt(&IDT) }
}
