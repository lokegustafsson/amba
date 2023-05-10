use capstone::prelude::*;

pub fn x64_to_assembly(x64_code: &[u8], start_addr: u64) -> Vec<(usize, String)> {
	let cs = Capstone::new()
		.x86()
		.mode(arch::x86::ArchMode::Mode64)
		.syntax(arch::x86::ArchSyntax::Intel)
		.detail(true)
		.build()
		.expect("Failed to create Capstone object");

	let insns = cs
		.disasm_all(x64_code, start_addr)
		.expect("Failed to disassemble");
	insns
		.iter()
		.map(|ins| (ins.len(), ins.to_string()))
		.collect()
}

#[cfg(test)]
mod tests {
	use crate::*;

	#[test]
	fn example() {
		const MACHINE_CODE: &'static [u8] =
			b"\x55\x48\x8b\x05\xb8\x13\x00\x00\xe9\x14\x9e\x08\x00\x45\x31\xe4";
		const ASSEMBLY: &str = "\n0x1000: push rbp\n0x1001: mov rax, qword ptr [rip + 0x13b8]\n0x1008: jmp 0x8ae21\n0x100d: xor r12d, r12d";

		assert_eq!(
			x64_to_assembly(MACHINE_CODE, 0x1000)
				.into_iter()
				.map(|(_, ins)| format!("\n{ins}"))
				.collect::<String>(),
			ASSEMBLY
		)
	}
}
