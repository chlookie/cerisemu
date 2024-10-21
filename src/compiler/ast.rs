use std::collections::HashMap;

use logos::Span;

use crate::emulator::{
	permission::Permission,
	program::{Address, Capability, LabelIdentifier, Register, WordChar, WordInt},
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub type WordString = String;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ast {
	pub rows: Vec<(AstRow, Span)>,
	pub labels: HashMap<String, Address>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AstRow {
	Word(AstWord),
	Instruction(AstInstruction),
	String(WordString),
	Label(LabelIdentifier),
	Goto(LabelIdentifier),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AstWord {
	Integer(WordInt),
	Expression(AstExpression),
	Char(WordChar),
	Capability(Capability),
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[rustfmt::skip]
pub enum AstInstruction {
	/// fail
	Fail,
	/// halt
	Halt,
	/// mov r ρ
	Mov     (Register, AstRegisterOrWord),
	/// load r1 r2
	Load    (Register, Register),
	/// store r ρ
	Store   (Register, AstRegisterOrWord),
	/// jmp r
	Jmp     (Register),
	/// jnz r1 r2
	Jnz     (Register, Register),
	/// restrict r ρ
	Restrict(Register, Permission),
	/// subseg r ρ1 ρ2
	Subseg  (Register, AstRegisterOrWord, AstRegisterOrWord),
	/// lea r ρ
	Lea     (Register, AstRegisterOrWord),
	/// add r ρ1 ρ2
	Add     (Register, AstRegisterOrWord, AstRegisterOrWord),
	/// sub r ρ1 ρ2
	Sub     (Register, AstRegisterOrWord, AstRegisterOrWord),
	/// lt r ρ1 ρ2
	Lt      (Register, AstRegisterOrWord, AstRegisterOrWord),
	/// getp r1 r2
	Getp    (Register, Register),
	/// getb r1 r2
	Getb    (Register, Register),
	/// gete r1 r2
	Gete    (Register, Register),
	/// geta r1 r2
	Geta    (Register, Register),
	/// isptr r1 r2
	Isptr   (Register, Register),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AstExpression {
	Label(LabelIdentifier),
	Integer(WordInt),
	OpAdd(Box<AstExpression>, Box<AstExpression>),
	OpSub(Box<AstExpression>, Box<AstExpression>),
	OpMul(Box<AstExpression>, Box<AstExpression>),
	OpDiv(Box<AstExpression>, Box<AstExpression>),
	OpNeg(Box<AstExpression>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AstRegisterOrWord {
	Register(Register),
	Word(AstWord),
}
