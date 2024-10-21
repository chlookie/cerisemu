use cerisemu::emulator::{
	self,
	machine::State,
	machine_config::MachineConfig,
	program::{Register, Word},
};

#[test]
fn load_works() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x100,
				registers: {
					R(0): Capability(RX, 0x00, 0x100, 0xFF),
				},
				programs: {
					0x00: Source("load R1 R0, halt"),
					0xFF: Source("42"),
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Halted);
	assert_eq!(machine.read_register(Register::R(1)), Word::Integer(42));
}

#[test]
fn load_fails_bounds1() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x100,
				registers: {
					R(0): Capability(RX, 0x00, 0xFF, 0xFF),
				},
				programs: {
					0x00: Source("load R1 R0, halt"),
					0xFF: Source("42"),
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Failed);
}

#[test]
fn load_fails_bounds2() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x100,
				registers: {
					R(0): Capability(RX, 0x100, 0x101, 0xFF),
				},
				programs: {
					0x00: Source("load R1 R0, halt"),
					0xFF: Source("42"),
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Failed);
}

#[test]
fn load_fails_permission() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x100,
				registers: {
					R(0): Capability(O, 0x00, 0x100, 0xFF),
				},
				programs: {
					0x00: Source("load R1 R0, halt"),
					0xFF: Source("42"),
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Failed);
}
