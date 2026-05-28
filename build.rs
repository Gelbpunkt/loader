use std::env;

fn main() {
	built::write_built_file().expect("Failed to acquire build-time information");

	set_linker_script();
}

fn set_linker_script() {
	let cfg_target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
	let cfg_feature = env::var("CARGO_CFG_FEATURE").unwrap();

	let linker_scripts: &[&str] = match cfg_target_arch.as_str() {
		"aarch64" if cfg_feature.contains("elf") => &["image.ld", "src/arch/aarch64/memory.ld"],
		"riscv64" if cfg_feature.contains("sbi") => &["src/arch/riscv64/link.ld"],
		"x86_64" if cfg_feature.contains("multiboot") => {
			&["src/arch/x86_64/platform/multiboot/link.ld"]
		}
		_ => return,
	};

	for linker_script in linker_scripts {
		println!("cargo:rerun-if-changed={linker_script}");
		println!("cargo:rustc-link-arg=-T{linker_script}");
	}
}
