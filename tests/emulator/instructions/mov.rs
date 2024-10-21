use cerisemu::emulator::{
	self,
	machine::State,
	machine_config::MachineConfig,
	program::{Register, Word},
};

#[test]
fn mov_integer() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
				programs: {
					0x00: Source("mov R2 42, halt")
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Halted);
	assert_eq!(machine.read_register(Register::R(2)), Word::Integer(42));
}

#[test]
fn mov_register() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
				registers: {
					R(0): Integer(42), // Random value
				},
				programs: {
					0x00: Source("mov R1 R0, halt")
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Halted);
	assert_eq!(
		machine.read_register(Register::R(1)),
		machine.read_register(Register::R(0))
	);
}
