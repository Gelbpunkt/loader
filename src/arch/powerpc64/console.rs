/// QEMU's pseries uses a spapr-vty device mapped to hvc0 in Linux
/// It uses hypervisor calls as opposed to RTAS or 16550 UART, which are
/// used by other machine types like e500.
///
/// Currently, we only implement the hardware spapr-vty.
///
/// See https://files.openpower.foundation/s/ZmtZyCGiJ2oJHim for more
/// information on sPAPR and RTAS. The virtual terminal implemented here
/// is described in section 17.5.
///
/// QEMU documentation is found at the following link:
/// https://www.qemu.org/docs/master/system/ppc/pseries.html
///
/// https://www.linux-kvm.org/page/PowerPC_Hypercall_ABI
use core::arch::asm;

// PowerPC HCALL codes

const H_PUT_TERM_CHAR: u64 = 0x58;

// Return codes

const H_SUCCESS: u64 = 0;
const H_BUSY: u64 = 1;

fn pack_bytes_to_u64(buf: &[u8]) -> (u64, u64, usize) {
	let len = buf.len().min(16);
	let mut first_word = 0u64;
	let mut second_word = 0u64;

	for i in 0..len.min(8) {
		first_word |= (buf[i] as u64) << (8 * (7 - i));
	}
	for i in 8..len {
		second_word |= (buf[i] as u64) << (8 * (15 - i));
	}
	(first_word, second_word, len)
}

pub struct Console;

impl Console {
	pub fn write_bytes(&self, buf: &[u8]) {
		let mut offset = 0;
		let mut status: u64;

		while offset < buf.len() {
			let chunk = &buf[offset..buf.len().min(offset + 16)];
			let (first_word, second_word, count) = pack_bytes_to_u64(chunk);

			loop {
				unsafe {
					asm!(
						".long 0x44000022", // hvsc
						in("r3") H_PUT_TERM_CHAR,
						in("r4") 0, // terminal 0
						in("r5") count as u64,
						in("r6") first_word,
						in("r7") second_word,
						lateout("r3") status,
						options(nostack, preserves_flags, nomem),
					);
				}

				if status == H_SUCCESS {
					break;
				} else if status == H_BUSY {
					// Try again immediately
				} else {
					// Panic here makes so much sense, I know
					panic!("Hypercall failed with status {:#x}", status);
				}
			}

			offset += count;
		}
	}
}

impl Default for Console {
	fn default() -> Self {
		Self
	}
}
