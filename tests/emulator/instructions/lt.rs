use cerisemu::emulator::{
	self,
	machine::State,
	machine_config::MachineConfig,
	program::{Register, Word},
};

#[test]
fn lt_test1() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
				programs: {
					0x00: Source("lt R0 0x001 0x002, halt")
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Halted);
	assert_eq!(machine.read_register(Register::R(0)), Word::Integer(1));
}

#[test]
fn lt_test2() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
				programs: {
					0x00: Source("lt R0 0x001 0x000, halt")
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Halted);
	assert_eq!(machine.read_register(Register::R(0)), Word::Integer(0));
}

#[test]
fn lt_test3() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
				programs: {
					0x00: Source("lt R0 0x001 0x001, halt")
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Halted);
	assert_eq!(machine.read_register(Register::R(0)), Word::Integer(0));
}
