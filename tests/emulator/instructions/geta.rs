use cerisemu::emulator::{
	self,
	machine::State,
	machine_config::MachineConfig,
	program::{Register, Word},
};

#[test]
fn geta_test1() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
                registers: {
					R(0): Capability(RX, 0x000, 0x001, 0x000), // Random Capability
				},
				programs: {
					0x00: Source("geta R1 R0, halt")
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Halted);
	assert_eq!(machine.read_register(Register::R(1)), Word::Integer(0x000));
}

#[test]
fn geta_test2() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
                registers: {
					R(0): Capability(RX, 0x000, 0x003, 0x003), // Random Capability
				},
				programs: {
					0x00: Source("geta R1 R0, halt")
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Halted);
	assert_eq!(machine.read_register(Register::R(1)), Word::Integer(0x003));
}
