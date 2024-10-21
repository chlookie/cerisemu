use cerisemu::emulator::{
	self, machine::State, machine_config::MachineConfig, permission::Permission::*, program::Register,
};

use crate::assert_register_capability;

#[test]
fn restrict_rwx_to_rw() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
                registers: {
					R(0): Capability(RWX, 0x000, 0x001, 0x000), // Random Capability
				},
				programs: {
					0x00: Source("restrict R0 RW, halt")
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Halted);
	assert_register_capability!(machine, Register::R(0), (RW, _, _, _));
}

#[test]
fn restrict_rwx_to_e() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
                registers: {
					R(0): Capability(RWX, 0x000, 0x001, 0x000), // Random Capability
				},
				programs: {
					0x00: Source("restrict R0 E, halt")
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Halted);
	assert_register_capability!(machine, Register::R(0), (E, _, _, _));
}

#[test]
fn restrict_fail_ro_to_e() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
                registers: {
					R(0): Capability(RO, 0x000, 0x001, 0x000), // Random Capability
				},
				programs: {
					0x00: Source("restrict R0 E, halt")
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
fn restrict_fail_rw_to_e() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
                registers: {
					R(0): Capability(RW, 0x000, 0x001, 0x000), // Random Capability
				},
				programs: {
					0x00: Source("restrict R0 E, halt")
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
fn restrict_fail_ro_to_rx() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
                registers: {
					R(0): Capability(RO, 0x000, 0x001, 0x000), // Random Capability
				},
				programs: {
					0x00: Source("restrict R0 RX, halt")
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Failed);
}
