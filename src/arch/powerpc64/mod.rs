mod console;
mod start;

use core::slice;

use fdt::node::FdtNode;
use hermit_entry::boot_info::{DeviceTreeAddress, HardwareInfo, PlatformInfo};
use hermit_entry::elf::LoadedKernel;
use log::info;
use sptr::Strict;

pub use self::console::Console;
use super::address_range::AddressRange;
use crate::{BootInfo, BootInfoExt};

fn find_kernel_linux(chosen: &FdtNode<'_, '_>) -> Option<&'static [u8]> {
	let initrd_start = chosen.property("linux,initrd-start")?.as_usize()?;
	let initrd_start = sptr::from_exposed_addr_mut::<u8>(initrd_start);
	let initrd_end = chosen.property("linux,initrd-end")?.as_usize()?;
	let initrd_end = sptr::from_exposed_addr_mut::<u8>(initrd_end);
	// SAFETY: We trust the raw pointer from the firmware
	let initrd_len = unsafe { initrd_end.offset_from(initrd_start).try_into().unwrap() };

	// SAFETY: We trust the raw pointer from the firmware
	Some(unsafe { slice::from_raw_parts(initrd_start, initrd_len) })
}

pub fn find_kernel() -> &'static [u8] {
	let fdt = start::get_fdt();
	let chosen = fdt.find_node("/chosen").unwrap();
	find_kernel_linux(&chosen).expect("could not find kernel")
}

pub unsafe fn get_memory(memory_size: u64) -> u64 {
	let memory_size = usize::try_from(memory_size).unwrap();

	let initrd = AddressRange::try_from(find_kernel().as_ptr_range()).unwrap();
	let fdt = {
		let start = start::get_fdt_ptr();
		let end = unsafe { start.add(start::get_fdt().total_size()) };
		AddressRange::try_from(start..end).unwrap()
	};

	info!("initrd = {initrd}");
	info!("fdt    = {fdt}");

	// OpenPOWER specification, 4.1 "Program Loading"
	const RECOMMENDED_CONGRUENCY: usize = 64 * 1024;
	let initrd = initrd.align_to(RECOMMENDED_CONGRUENCY);
	let fdt = fdt.align_to(RECOMMENDED_CONGRUENCY);

	let [first, second] = if initrd < fdt {
		[initrd, fdt]
	} else {
		[fdt, initrd]
	};

	let start_address = if first.next(memory_size).overlaps(second) {
		second.end()
	} else {
		first.end()
	};

	u64::try_from(start_address).unwrap()
}

pub unsafe fn boot_kernel(kernel_info: LoadedKernel) -> ! {
	info!("We are booting!");

	let LoadedKernel {
		load_info,
		entry_point,
	} = kernel_info;

	let fdt = start::get_fdt();

	let phys_addr_range = {
		let memory = fdt.memory();
		let mut regions = memory.regions();

		let mem_region = regions.next().unwrap();
		assert!(
			regions.next().is_none(),
			"hermit-loader can only handle one memory region yet"
		);

		let mem_base = u64::try_from(mem_region.starting_address.addr()).unwrap();
		let mem_size = u64::try_from(mem_region.size.unwrap()).unwrap();
		mem_base..mem_base + mem_size
	};

	let device_tree = {
		let fdt_addr = start::get_fdt_ptr().expose_addr();
		DeviceTreeAddress::new(fdt_addr.try_into().unwrap())
	};

	let boot_info = BootInfo {
		hardware_info: HardwareInfo {
			phys_addr_range,
			serial_port_base: None,
			device_tree,
		},
		load_info,
		platform_info: PlatformInfo::LinuxBoot,
	};

	let stack = start::get_stack_ptr();
	let entry: *const u8 = sptr::from_exposed_addr(entry_point.try_into().unwrap());
	let raw_boot_info = boot_info.write();

	info!("Entering kernel");

	loop {
		core::hint::spin_loop();
	}
}
