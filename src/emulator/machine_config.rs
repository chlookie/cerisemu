use std::{
	collections::HashMap,
	fs,
	io::{self, Read},
};

use serde::{Deserialize, Serialize};

use crate::compiler;

use super::{
	machine::Interrupt,
	permission::Permission,
	program::{AddrInt, Address, Capability, Program, Register, Word, WordChar, WordInt},
	signed::Signed,
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct MachineConfig {
	pub size: usize,

	#[serde(default)]
	pub programs: HashMap<AddrInt, ProgramConfig>,

	#[serde(default)]
	pub registers: HashMap<Register, ParsingWord>,

	#[serde(default)]
	pub interrupt_table: HashMap<Interrupt, AddrInt>,
}

impl MachineConfig {
	pub fn from_program_config(program_config: ProgramConfig) -> Self {
		// We can't just put the direct ProgramConfig into programs, since we don't know what size the machine should be.
		// So by default we go ahead and pre-compile the program
		Self::from_program(program_config.compiled())
	}

	pub fn from_program(program: Program) -> Self {
		let size = program.rows.len();
		let program_config = ProgramConfig::CompiledProgram(program);
		let programs = HashMap::from([(0, program_config)]);

		Self {
			size,
			programs,
			..Default::default()
		}
	}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ProgramConfig {
	SourceFile(String),
	Source(String),
	CompiledFile(String),
	CompiledRon(String),
	CompiledProgram(Program),
}

impl ProgramConfig {
	pub fn from_path_as_source(path: &str) -> Self {
		Self::SourceFile(path.to_owned())
	}

	pub fn from_reader_as_source(reader: impl Read) -> Self {
		Self::Source(io::read_to_string(reader).expect("Couldn't read input reader"))
	}

	pub fn from_path_as_compiled(path: &str) -> Self {
		Self::CompiledFile(path.to_owned())
	}

	pub fn from_reader_as_compiled(reader: impl Read) -> Self {
		Self::CompiledRon(io::read_to_string(reader).expect("Couldn't read input reader"))
	}

	pub fn compiled(self) -> Program {
		match self {
			ProgramConfig::SourceFile(path) => {
				ProgramConfig::Source(fs::read_to_string(path).expect("Couldn't read source file")).compiled()
			}

			ProgramConfig::CompiledFile(path) => {
				ProgramConfig::CompiledRon(fs::read_to_string(path).expect("Couldn't read source file")).compiled()
			}

			ProgramConfig::CompiledRon(source) => {
				ron::de::from_str(&source).expect("Couldn't deserialize program config")
			}

			ProgramConfig::Source(source) => compiler::compile_unwrapped(&source),
			ProgramConfig::CompiledProgram(program) => program,
		}
	}
}

/// This type exists just to make it less annoying to write MachineConfigs in ron.
/// Instead of
///    R0: Capability(Signed(Capability(perm: O, base: Address(0), end: Address(0), address: Address(0))))
/// we can write
///    R0: Capability(O, 0, 0, 0)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ParsingWord {
	Integer(WordInt),
	Char(WordChar),
	Capability(Permission, AddrInt, AddrInt, AddrInt),
	Permission(Permission),
}

impl ParsingWord {
	pub fn parse(self) -> Word {
		match self {
			ParsingWord::Integer(i) => Word::Integer(i),
			ParsingWord::Char(c) => Word::Char(c),
			ParsingWord::Capability(p, b, e, a) => Word::Capability(Signed::new_unsigned(Capability {
				perm: p,
				base: Address(b),
				end: Address(e),
				address: Address(a),
			})),
			ParsingWord::Permission(p) => Word::Permission(p),
		}
	}
}
