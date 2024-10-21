use std::{
	cmp::max,
	collections::HashMap,
	fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::util::{indent_string, pretty_hashmap};

use super::{
	machine_config::MachineConfig,
	memory::Memory,
	program::{Address, Capability, Program, Register, Row, Word},
	signed::{self, Signable, Signed, SigningKey, VerifyingKey},
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(Serialize, Clone, Debug)]
pub struct Machine {
	pub exec_state: State,
	registers: HashMap<Register, Word>,
	interrupt_table: HashMap<Interrupt, Address>,
	pub memory: Memory,

	#[serde(skip)]
	signing_key: SigningKey,
	#[serde(skip)]
	verifying_key: VerifyingKey,

	#[serde(skip)]
	backtrace: Vec<Vec<String>>,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum State {
	Running,

	#[default]
	Halted,

	Failed,

	Interrupted(Interrupt),
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Interrupt {
	Halt,
	Fail,
}

impl Default for Machine {
	fn default() -> Self {
		let (signing_key, verifying_key) = signed::create_key_pair();

		Self {
			exec_state: Default::default(),
			registers: Default::default(),
			memory: Default::default(),
			interrupt_table: Default::default(),
			backtrace: Default::default(),
			signing_key,
			verifying_key,
		}
	}
}

impl Machine {
	pub fn new() -> Self {
		Self { ..Default::default() }
	}

	pub fn initialize_from_program(program: Program) -> Self {
		let mut machine = Self::new();
		machine.load_program(program, Address(0x0));
		machine
	}

	pub fn initialize_from_config(machine_config: MachineConfig) -> Self {
		let mut machine = Self {
			memory: Memory::new(machine_config.size),
			..Default::default()
		};

		// Load programs from the config
		for (address_int, program_config) in machine_config.programs {
			let program = program_config.compiled();
			machine.load_program(program, Address(address_int))
		}

		// Load registers from the config
		for (register, parsing_value) in machine_config.registers {
			let mut value = parsing_value.parse();

			if let Word::Capability(capability) = value {
				// Sign capability if needed before writing to the register
				value = Word::Capability(capability.re_signed(&machine.signing_key))
			}

			// Write the value to the corresponding register
			machine.write_register(register, value);
		}

		// Load interrupt table from the config
		for (interrupt, address_int) in machine_config.interrupt_table {
			machine.set_interrupt_address(interrupt, Address(address_int))
		}

		machine
	}

	pub fn load_program(&mut self, program: Program, address: Address) {
		self.memory.load_program(program, address)
	}

	pub fn read_register(&self, register: Register) -> Word {
		self.registers.get(&register).cloned().unwrap_or_default()
	}

	pub fn write_register(&mut self, register: Register, value: Word) {
		self.registers.insert(register, value.clone());

		if register != Register::PC {
			self.append_backtrace(format!("{} = {}", register, value));
		}
	}

	pub fn verify_capability(&self, signed_capability: Signed<Capability>) -> Option<Capability> {
		signed_capability.verify(&self.verifying_key)
	}

	pub fn sign_capability(&self, capability: Capability) -> Signed<Capability> {
		capability.signed(&self.signing_key)
	}

	pub fn get_register_capability(&self, register: Register) -> Option<Capability> {
		if let Word::Capability(capability) = self.read_register(register) {
			self.verify_capability(capability)
		} else {
			None
		}
	}

	pub fn set_interrupt_address(&mut self, interrupt: Interrupt, address: Address) {
		self.interrupt_table.insert(interrupt, address);
	}

	pub fn get_interrupt_memory(&mut self, interrupt: Interrupt) -> Row {
		self.memory[*self.interrupt_table.entry(interrupt).or_insert(Address(0x0))].clone()
	}

	pub fn new_backtrace(&mut self, message: String) {
		self.backtrace.push(vec![message]);
	}

	pub fn append_backtrace(&mut self, message: String) {
		if let Some(last) = self.backtrace.last_mut() {
			last.push(message);
		} else {
			self.new_backtrace(message);
		}
	}

	pub fn print_status(&self) {
		let state = self.exec_state;
		let pc = &self.read_register(Register::PC);

		if let Some(Capability { address, .. }) = self.get_register_capability(Register::PC) {
			println!("Machine status: {} at address {}", state, address)
		} else {
			println!("Machine status: {} at invalid address (PC = {})", state, pc)
		}
	}

	pub fn print_backtrace(&self) {
		println!("Machine backtrace:");

		if self.backtrace.is_empty() {
			println!("(Empty)");
			return;
		}

		let collumn_sizes = self.backtrace.iter().fold(Vec::<usize>::new(), |mut acc, trace| {
			while acc.len() < trace.len() {
				acc.push(0)
			}
			for (i, a) in acc.iter_mut().enumerate() {
				if i < trace.len() - 1 {
					*a = max(*a, trace.get(i).map(String::len).unwrap_or(0))
				}
			}
			acc
		});

		for trace in &self.backtrace {
			println!(
				"> {}",
				trace
					.iter()
					.enumerate()
					.map(|(i, s)| format!("{:<width$}", s, width = collumn_sizes[i]))
					.collect::<Vec<String>>()
					.join(" | ")
			);
		}
	}
}

impl Display for Machine {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		let indent = "   ";

		let exec_state = self.exec_state;
		let interrupt_table = indent_string(&pretty_hashmap(&self.interrupt_table), indent);
		let registers = indent_string(&pretty_hashmap(&self.registers), indent);
		let memory = indent_string(&format!("{}", self.memory), indent);

		let inner_machine = indent_string(
			&format!(
				"State: {}\nInterrupt Table: {}\nRegisters: {}\nMemory: {}",
				exec_state, interrupt_table, registers, memory,
			),
			indent,
		);

		f.pad(&format!("Machine (\n{}\n)", inner_machine))
	}
}
