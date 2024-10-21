use cerisemu::emulator::{
	self, machine::State, machine_config::MachineConfig, permission::Permission::*, program::Register,
};

use crate::assert_register_capability;

#[test]
fn subseg_limit_end() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
                registers: {
					R(0): Capability(RX, 0x000, 0x004, 0x000), // Random Capability
				},
				programs: {
					0x00: Source("subseg R0 0x000 0x002, halt")
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Halted);
	assert_register_capability!(machine, Register::R(0), (RX, 0x000, 0x002, 0x000));
}

#[test]
fn subseg_limit_base() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
                registers: {
					R(0): Capability(RX, 0x000, 0x004, 0x000), // Random Capability
				},
				programs: {
					0x00: Source("subseg R0 0x002 0x004, halt")
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Halted);
	assert_register_capability!(machine, Register::R(0), (RX, 0x002, 0x004, 0x000));
}

#[test]
fn subseg_fail_limit_base_and_test_address() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
                registers: {
					R(0): Capability(RX, 0x000, 0x004, 0x000), // Random Capability
				},
				programs: {
					0x00: Source("subseg R0 0x002 0x004, store R0 R0, halt")
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Failed);
	assert_register_capability!(machine, Register::R(0), (RX, 0x002, 0x004, 0x000));
}

#[test]
fn subseg_limit_both() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
                registers: {
					R(0): Capability(RX, 0x000, 0x004, 0x000), // Random Capability
				},
				programs: {
					0x00: Source("subseg R0 0x001 0x003, halt")
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Halted);
	assert_register_capability!(machine, Register::R(0), (RX, 0x001, 0x003, 0x000));
}

#[test]
fn subseg_fail_extend_end() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
                registers: {
					R(0): Capability(RX, 0x000, 0x004, 0x000), // Random Capability
				},
				programs: {
					0x00: Source("subseg R0 0x001 0x005, halt")
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
fn subseg_fail_extend_base() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
                registers: {
					R(0): Capability(RX, 0x002, 0x004, 0x000), // Random Capability
				},
				programs: {
					0x00: Source("subseg R0 0x001 0x004, halt")
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
fn subseg_fail_extend_both() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
                registers: {
					R(0): Capability(RX, 0x002, 0x004, 0x000), // Random Capability
				},
				programs: {
					0x00: Source("subseg R0 0x001 0x005, halt")
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Failed);
}
