use serde::{Deserialize, Serialize};

use super::{instruction::Instruction, permission::Permission, signed::Signed};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub type RegInt = u8;
pub type AddrInt = usize;

pub type WordInt = i64;
pub type WordChar = char;

pub type LabelIdentifier = String;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct Program {
	pub rows: Vec<Row>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Row {
	Word(Word),
	Instruction(Instruction),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Word {
	Integer(WordInt),
	Char(WordChar),
	Capability(Signed<Capability>),
	Permission(Permission),
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Capability {
	/// Permission
	pub perm: Permission,

	/// Base
	pub base: Address,

	/// End
	pub end: Address,

	/// Address
	pub address: Address,
}

#[derive(Serialize, Deserialize, Hash, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Register {
	PC,
	R(RegInt),
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address(pub AddrInt);
