#![no_std]
#![no_main]
#![warn(rust_2018_idioms)]
#![warn(unsafe_op_in_unsafe_fn)]
#![allow(unstable_name_collisions)]
#![allow(clippy::missing_safety_doc)]

use ::log::info;
#[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
use goblin::elf::header::header64::{EI_DATA, ELFDATA2LSB, ELFMAG, Header, SELFMAG};
use hermit_entry::boot_info::{BootInfo, RawBootInfo};

#[macro_use]
mod macros;

mod arch;
mod bump_allocator;
#[cfg(any(target_os = "uefi", target_arch = "x86_64"))]
mod fdt;
mod log;
mod os;

#[cfg(any(target_os = "uefi", all(target_arch = "x86_64", target_os = "none")))]
extern crate alloc;

trait BootInfoExt {
	fn write(self) -> &'static RawBootInfo;
}

impl BootInfoExt for BootInfo {
	fn write(self) -> &'static RawBootInfo {
		info!("boot_info = {self:#x?}");

		take_static::take_static! {
			static RAW_BOOT_INFO: Option<RawBootInfo> = None;
		}

		let raw_boot_info = RAW_BOOT_INFO.take().unwrap();

		raw_boot_info.insert(RawBootInfo::from(self))
	}
}

#[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
trait FdtExt {
	fn find_kernel(&self) -> Option<&'static [u8]>;
}

#[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
impl FdtExt for fdt::Fdt<'_> {
	fn find_kernel(&self) -> Option<&'static [u8]> {
		let chosen = self.find_node("/chosen").unwrap();

		// The reg size of the module nodes is always 0, so we cannot trust them and
		// instead need to parse the ELF header
		let find_module_start = || {
			let module = chosen
				.children()
				.find(|node| node.name.starts_with("module@"))?;
			let start_ptr = module.reg().unwrap().next().unwrap().starting_address;
			let header = unsafe { &*core::mem::transmute::<*const u8, *const Header>(start_ptr) };

			if header.e_ident[0..SELFMAG] != ELFMAG[..] {
				return None;
			}

			let len = if header.e_ident[EI_DATA] == ELFDATA2LSB {
				u64::from_le(header.e_shoff)
					+ (u16::from_le(header.e_shentsize) as u64
						* u16::from_le(header.e_shnum) as u64)
			} else {
				u64::from_be(header.e_shoff)
					+ (u16::from_be(header.e_shentsize) as u64
						* u16::from_be(header.e_shnum) as u64)
			};

			Some(unsafe { core::slice::from_raw_parts(start_ptr, len.try_into().unwrap()) })
		};

		let find_linux_initrd = || {
			let start = chosen.property("linux,initrd-start")?.as_usize()?;
			let end = chosen.property("linux,initrd-end")?.as_usize()?;
			let start_ptr = core::ptr::with_exposed_provenance::<u8>(start);
			let end_ptr = core::ptr::with_exposed_provenance::<u8>(end);
			let len = unsafe { end_ptr.offset_from(start_ptr).try_into().unwrap() };
			Some(unsafe { core::slice::from_raw_parts(start_ptr, len) })
		};

		find_module_start().or_else(find_linux_initrd)
	}
}

#[doc(hidden)]
fn _print(args: core::fmt::Arguments<'_>) {
	use core::fmt::Write;

	self::os::CONSOLE.lock().write_fmt(args).unwrap();
}
