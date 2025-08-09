use core::arch::naked_asm;
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};

use fdt::Fdt;
use log::info;

static mut STACK: Stack = Stack::new();
static FDT: AtomicPtr<u8> = AtomicPtr::new(ptr::null_mut());

pub fn get_fdt_ptr() -> *const u8 {
	FDT.load(Ordering::Relaxed).cast_const()
}

pub fn get_fdt() -> Fdt<'static> {
	// SAFETY: We trust the FDT pointer provided by the firmware
	unsafe { Fdt::from_ptr(get_fdt_ptr()).unwrap() }
}

pub fn get_stack_ptr() -> *mut u8 {
	// SAFETY: We only create a pointer here
	let stack_top = ptr::addr_of_mut!(STACK);
	// SAFETY: Pointing directly past the object is allowed
	let stack_bottom = unsafe { stack_top.add(1) };
	stack_bottom.cast::<u8>()
}

// TODO: Migrate to Constrained Naked Functions
// https://github.com/rust-lang/rust/issues/90957
// TODO: Migrate to asm_const for Stack::SIZE
// https://github.com/rust-lang/rust/issues/93332

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn _start() -> ! {
	naked_asm!(
		// Build 64-bit address into r1 from symbol BOOT_STACK
		//"lis     1, {stack}@highest",     // r1 = (stack >> 48) << 48
		//"ori     1, 1, {stack}@higher",   // r1 |= ((stack >> 32) & 0xFFFF) << 32
		//"rldicr  1, 1, 32, 31",           // r1 <<= 32 (clear lower 32 bits)
		//"oris    1, 1, {stack}@h",        // r1 |= ((stack >> 16) & 0xFFFF) << 16
		//"ori     1, 1, {stack}@l",        // r1 |= (stack & 0xFFFF)

		// Adjust stack pointer to top of stack (stack grows down)
		//"addi    1, 1, {stack_size}",

		// Initialize r2 to TOC_PTR
		//"lis     2, TOC_PTR@highest",
		//"ori     2, 2, TOC_PTR@higher",
		//"rldicr  2, 2, 32, 31",
		//"oris    2, 2, TOC_PTR@h",
		//"ori     2, 2, TOC_PTR@l",

		// Branch to start() function
		"bl      {start}",
		"b       .",

		//stack = sym STACK,
		//stack_size = const 16 * 1024, // 32K stack
		start = sym start,
	);
}

extern "C" fn start(fdt: *const u8) {
	let bytes = b"Hello world!\n";
	let bytes2 = b"does this work?";
	unsafe { core::arch::asm!("trap") };

	super::Console::default().write_bytes(bytes);
	super::Console::default().write_bytes(bytes2);

	//FDT.store(fdt.cast_mut(), Ordering::SeqCst);

	//super::Console::default().write_bytes(get_fdt().root().model().as_bytes());

	loop {
		core::hint::spin_loop();
	}

	//unsafe { crate::os::loader_main() }
}

// Align to page size
#[repr(C, align(0x1000))]
pub struct Stack([u8; Self::SIZE]);

impl Stack {
	const SIZE: usize = 0x4000;

	pub const fn new() -> Self {
		Self([0; Self::SIZE])
	}
}
