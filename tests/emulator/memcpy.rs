use cerisemu::emulator::{
	self,
	machine::State,
	machine_config::{MachineConfig, ProgramConfig},
	program::Address,
};

#[test]
fn works() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x100,
				registers: {
					// To call memcpy:
					// r0: return
					// r1: source memory (RO, a, b, a)
					// r2: destination memory (RW, c, d, c)
					
					R(0):  Capability(E , 0x00, 0x02, 0x01), // Return to the 'halt' instruction
					R(1):  Capability(RO, 0x50, 0x5B, 0x50), // Source, should copy 'Lorem ipsum'
					R(2):  Capability(RW, 0x60, 0x6B, 0x60), // Destination
					R(16): Capability(E , 0x10, 0x32, 0x10), // memcpy, 33 rows long
				},
				programs: {
					0x00: Source("jmp r16, halt"),
					0x10: SourceFile("asm/memcpy.asm"),
					0x50: SourceFile("asm/lorem.asm"),
				},
			)
		"#,
	)
	.unwrap();

	let result_machine = emulator::emulate(config);

	let dst_memory = &result_machine.memory[Address(0x60)..Address(0x6B)];
	let expected = ProgramConfig::Source(r#""Lorem ipsum""#.to_owned()).compiled().rows;

	assert_eq!(result_machine.exec_state, State::Halted);
	assert_eq!(dst_memory, expected.as_slice());
}

#[test]
fn works_bad_address1() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x100,
				registers: {
					// To call memcpy:
					// r0: return
					// r1: source memory (RO, a, b, a)
					// r2: destination memory (RW, c, d, c)
					
					R(0):  Capability(E , 0x00, 0x02, 0x01), // Return to the 'halt' instruction
					R(1):  Capability(RO, 0x50, 0x5B, 0x51), // BAD HERE, SHOULD AUTO-CORRECT
					R(2):  Capability(RW, 0x60, 0x6B, 0x60),
					R(16): Capability(E , 0x10, 0x32, 0x10), // memcpy, 33 rows long
				},
				programs: {
					0x00: Source("jmp r16, halt"),
					0x10: SourceFile("asm/memcpy.asm"),
					0x50: SourceFile("asm/lorem.asm"),
				},
			)
		"#,
	)
	.unwrap();

	let result_machine = emulator::emulate(config);

	let dst_memory = &result_machine.memory[Address(0x60)..Address(0x6B)];
	let expected = ProgramConfig::Source(r#""Lorem ipsum""#.to_owned()).compiled().rows;

	assert_eq!(result_machine.exec_state, State::Halted);
	assert_eq!(dst_memory, expected.as_slice());
}

#[test]
fn works_bad_address2() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x100,
				registers: {
					// To call memcpy:
					// r0: return
					// r1: source memory (RO, a, b, a)
					// r2: destination memory (RW, c, d, c)
					
					R(0):  Capability(E , 0x00, 0x02, 0x01), // Return to the 'halt' instruction
					R(1):  Capability(RO, 0x50, 0x5B, 0x50),
					R(2):  Capability(RW, 0x60, 0x6B, 0x61), // BAD HERE, SHOULD AUTO-CORRECT
					R(16): Capability(E , 0x10, 0x32, 0x10), // memcpy, 33 rows long
				},
				programs: {
					0x00: Source("jmp r16, halt"),
					0x10: SourceFile("asm/memcpy.asm"),
					0x50: SourceFile("asm/lorem.asm"),
				},
			)
		"#,
	)
	.unwrap();

	let result_machine = emulator::emulate(config);

	let dst_memory = &result_machine.memory[Address(0x60)..Address(0x6B)];
	let expected = ProgramConfig::Source(r#""Lorem ipsum""#.to_owned()).compiled().rows;

	assert_eq!(result_machine.exec_state, State::Halted);
	assert_eq!(dst_memory, expected.as_slice());
}

#[test]
fn fails_no_args() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x100,
				programs: {
					0x00: SourceFile("asm/memcpy.asm"),
				},
			)
		"#,
	)
	.unwrap();

	let result_machine = emulator::emulate(config);

	assert_eq!(result_machine.exec_state, State::Failed);
}

#[test]
fn fails_different_size1() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x100,
				registers: {
					// To call memcpy:
					// r0: return
					// r1: source memory (RO, a, b, a)
					// r2: destination memory (RW, c, d, c)
					
					R(0):  Capability(E , 0x00, 0x02, 0x01), // Return to the 'halt' instruction
					R(1):  Capability(RO, 0x50, 0x5B, 0x50),
					R(2):  Capability(RW, 0x60, 0x6C, 0x60), // BAD HERE
					R(16): Capability(E , 0x10, 0x32, 0x10), // memcpy, 33 rows long
				},
				programs: {
					0x00: Source("jmp r16, halt"),
					0x10: SourceFile("asm/memcpy.asm"),
					0x50: SourceFile("asm/lorem.asm"),
				},
			)
		"#,
	)
	.unwrap();

	let result_machine = emulator::emulate(config);

	assert_eq!(result_machine.exec_state, State::Failed);
}

#[test]
fn fails_different_size2() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x100,
				registers: {
					// To call memcpy:
					// r0: return
					// r1: source memory (RO, a, b, a)
					// r2: destination memory (RW, c, d, c)
					
					R(0):  Capability(E , 0x00, 0x02, 0x01), // Return to the 'halt' instruction
					R(1):  Capability(RO, 0x50, 0x5A, 0x50), // BAD HERE
					R(2):  Capability(RW, 0x60, 0x6B, 0x60),
					R(16): Capability(E , 0x10, 0x32, 0x10), // memcpy, 33 rows long
				},
				programs: {
					0x00: Source("jmp r16, halt"),
					0x10: SourceFile("asm/memcpy.asm"),
					0x50: SourceFile("asm/lorem.asm"),
				},
			)
		"#,
	)
	.unwrap();

	let result_machine = emulator::emulate(config);

	assert_eq!(result_machine.exec_state, State::Failed);
}

#[test]
fn fails_different_size3() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x100,
				registers: {
					// To call memcpy:
					// r0: return
					// r1: source memory (RO, a, b, a)
					// r2: destination memory (RW, c, d, c)
					
					R(0):  Capability(E , 0x00, 0x02, 0x01), // Return to the 'halt' instruction
					R(1):  Capability(RO, 0x51, 0x5B, 0x50), // BAD HERE
					R(2):  Capability(RW, 0x60, 0x6B, 0x60),
					R(16): Capability(E , 0x10, 0x32, 0x10), // memcpy, 33 rows long
				},
				programs: {
					0x00: Source("jmp r16, halt"),
					0x10: SourceFile("asm/memcpy.asm"),
					0x50: SourceFile("asm/lorem.asm"),
				},
			)
		"#,
	)
	.unwrap();

	let result_machine = emulator::emulate(config);

	assert_eq!(result_machine.exec_state, State::Failed);
}

#[test]
fn fails_different_size4() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x100,
				registers: {
					// To call memcpy:
					// r0: return
					// r1: source memory (RO, a, b, a)
					// r2: destination memory (RW, c, d, c)
					
					R(0):  Capability(E , 0x00, 0x02, 0x01), // Return to the 'halt' instruction
					R(1):  Capability(RO, 0x50, 0x5B, 0x50), // BAD HERE
					R(2):  Capability(RW, 0x5F, 0x6B, 0x60),
					R(16): Capability(E , 0x10, 0x32, 0x10), // memcpy, 33 rows long
				},
				programs: {
					0x00: Source("jmp r16, halt"),
					0x10: SourceFile("asm/memcpy.asm"),
					0x50: SourceFile("asm/lorem.asm"),
				},
			)
		"#,
	)
	.unwrap();

	let result_machine = emulator::emulate(config);

	assert_eq!(result_machine.exec_state, State::Failed);
}
