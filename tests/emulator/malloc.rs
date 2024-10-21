use cerisemu::emulator::{
	self, machine::State, machine_config::MachineConfig, permission::Permission::*, program::Register,
};

use crate::assert_register_capability;

#[test]
fn init_works() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
				registers: {
					R(0): Integer(-0x10),                         // Return to the 'halt' instruction
					R(1): Capability(RWX, 0x100, 0x1FF, 0x100), // Malloc heap range
				},
				programs: {
					0x00: Source("lea pc [0x10 - 1], halt"),
					0x10: SourceFile("asm/malloc.asm"),
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Halted);

	assert_register_capability!(machine, Register::R(1), (RWX, _, _, _));
}

#[test]
fn malloc_works_twice() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
				registers: {
					R(0): Integer(-0x20),                         // Return to return0
					R(1): Capability(RWX, 0x100, 0x1FF, 0x100), // Malloc heap range
				},
				programs: {
					0x00: Source("
						start:
							lea pc [0x20 - 1]                  ; call init
						return0:
							
							mov r19 r1                         ; save malloc()
							
							mov r1 16                          ; assign number of words to alloc
							mov r0 pc, lea r0 4, restrict r0 E ; create return address
							jmp r19                            ; call malloc
						return1:
							mov r20 r1
						
							mov r1 32 		                    ; assign number of words to alloc
							mov r0 pc, lea r0 4, restrict r0 E ; create return address
							jmp r19                            ; call malloc
						return2:
							mov r21 r1
						
						halt
					"),
					0x20: SourceFile("asm/malloc.asm"),
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Halted);

	assert_register_capability!(machine, Register::R(20), (RWX, 0x100, 0x110, 0x100));
	assert_register_capability!(machine, Register::R(21), (RWX, 0x110, 0x130, 0x110));
}

#[test]
fn malloc_fails_negative() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
				registers: {
					R(0): Integer(-0x20),                         // Return to after the first instruction
					R(1): Capability(RWX, 0x100, 0x1FF, 0x100), // Malloc heap range
				},
				programs: {
					0x00: Source("
						lea pc [0x20 - 1]                  ; call init
						
						mov r19 r1                         ; save malloc()
						
						mov r1 [-1]                        ; assign number of words to alloc
						mov r0 pc, lea r0 4, restrict r0 E ; create return address
						jmp r19                            ; call malloc
						halt
					"),
					0x20: SourceFile("asm/malloc.asm"),
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
fn malloc_fails_overflow() {
	let config = ron::de::from_str::<MachineConfig>(
		r#"
			MachineConfig(
				size: 0x200,
				registers: {
					R(0): Integer(-0x20),                         // Return to after the first instruction
					R(1): Capability(RWX, 0x100, 0x1FF, 0x100), // Malloc heap range
				},
				programs: {
					0x00: Source("
						lea pc [0x20 - 1]                  ; call init
						
						mov r19 r1                         ; save malloc()
						
						mov r1 0x100                       ; assign number of words to alloc
						mov r0 pc, lea r0 4, restrict r0 E ; create return address
						jmp r19                            ; call malloc
						halt
					"),
					0x20: SourceFile("asm/malloc.asm"),
				},
			)
		"#,
	)
	.unwrap();

	let machine = emulator::emulate(config);
	machine.print_backtrace();

	assert_eq!(machine.exec_state, State::Failed);
}
