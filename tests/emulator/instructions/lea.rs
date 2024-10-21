use cerisemu::emulator::{
	self, machine::State, machine_config::MachineConfig, permission::Permission::*, program::Register,
};

use crate::assert_register_capability;

#[test]
fn lea_positive_nr() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
                registers: {
					R(0): Capability(RX, 0x000, 0x001, 0x000), // Random Capability
				},
				programs: {
					0x00: Source("lea R0 1, halt")
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Halted);
	assert_register_capability!(machine, Register::R(0), (RX, _, _, 0x001));
}

#[test]
fn lea_negative_nr() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
				registers: {
					R(0): Capability(RX, 0x000, 0x001, 0x001), // Random Capability
				},
				programs: {
					0x00: Source("lea R0 [-1], halt")
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Halted);
	assert_register_capability!(machine, Register::R(0), (RX, _, _, 0x000));
}

#[test]
fn lea_fail_outside_of_allowance() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
				registers: {
					R(0): Capability(E, 0x000, 0x001, 0x001), // Random Capability
				},
				programs: {
					0x00: Source("lea R0 1, halt")
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Failed);
}
