use cerisemu::emulator::{
	self,
	machine::State,
	machine_config::MachineConfig,
	program::{Address, Row, Word},
};

#[test]
fn store_integer() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
                registers: {
					R(0): Capability(RW, 0x000, 0x004, 0x003), // Random Capability
				},
				programs: {
					0x00: Source("store R0 42, halt")
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Halted);
	assert_eq!(machine.memory[Address(0x003)], Row::Word(Word::Integer(42)))
}

#[test]
fn store_fails_missing_permission() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
                registers: {
					R(0): Capability(RO, 0x000, 0x004, 0x003), // Random Capability
				},
				programs: {
					0x00: Source("store R0 42, halt")
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Failed);
}
