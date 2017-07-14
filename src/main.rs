#![cfg_attr(target_arch = "arm", feature(core_intrinsics))]
#![feature(lang_items)]
#![feature(asm)]
#![feature(unique)]
#![feature(const_fn)]
#![feature(linkage)]
#![feature(compiler_builtins_lib)]

#![no_std]

#[macro_use]
extern crate bitflags;
extern crate spin;
extern crate compiler_builtins;

use core::ptr::Unique;

#[macro_use]
mod serial;
mod lang_items;

use core::str;

mod m0;
use m0::{PerilpM0, M0};

extern crate rk3399_tools;

mod clock_init;
use clock_init::setup_clocks;

const M0_START_ADDRESS:u32 = 0x250000;

fn main() {
	let pmugrf = unsafe { &*rk3399_tools::PMUGRF.get() };
	let pmucru = unsafe { &*rk3399_tools::PMUCRU.get() };
	let pmusgrf = unsafe { &*rk3399_tools::PMUSGRF.get() };

	println!("Hello from feo!");

	// setup iomux to select PMU JTAG
	pmugrf.pmugrf_gpio1b_iomux.modify(|_, w| unsafe {
		w.
		write_enable().bits(
			3 << 4 |
			3 << 2
		).
		gpio1b1_sel().bits(1). 	// pmum0jtag_tck
		gpio1b2_sel().bits(1)	// pmum0jtag_tms
	});

	// and enable SWD for the core
	pmusgrf.pmu_con0.modify(|_, w| unsafe { w.
		sgrf_mcu_dbgen().set_bit().
		write_mask().bits(1 << 5)
	});
	
	// memory fence
	unsafe { asm!("dsb sy"); }

	// TODO: before we do this, may need to configure
	// to enable everything into unsecure mode

	// start the M0
	// println!("Booting M0 at 0x{:x}...", M0_START_ADDRESS);
	let mut littleguy = PerilpM0 { };

	unsafe {
		littleguy.setup (pmusgrf, pmucru, M0_START_ADDRESS);
		littleguy.on (pmucru);
	}

	unsafe { asm!("wfi"); };
}
