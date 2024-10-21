use self::{machine::Machine, machine_config::MachineConfig};

pub mod exec;
pub mod instruction;
pub mod machine;
pub mod machine_config;
pub mod memory;
pub mod permission;
pub mod program;
pub mod signed;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub fn emulate(machine_config: MachineConfig) -> Machine {
	let mut machine = Machine::initialize_from_config(machine_config);
	machine.exec_machine();
	machine
}
