use cerisemu::emulator::{self, machine::State, machine_config::MachineConfig};

#[test]
fn jmp_halt() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
				registers: {
					R(0): Capability(E, 0x00, 0x0A, 0x09),
				},
				programs: {
					0x00: Source("jmp R0, fail"),
					0x09: Source("halt"),
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Halted);
}
