use serde::{Deserialize, Serialize};

use super::{
	permission::Permission,
	program::{Register, Word},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[rustfmt::skip]
pub enum Instruction {
	/// fail
	Fail,
	/// halt
	Halt,
	/// mov r ρ
	Mov     (Register, RegisterOrWord),
	/// load r1 r2
	Load    (Register, Register),
	/// store r ρ
	Store   (Register, RegisterOrWord),
	/// jmp r
	Jmp     (Register),
	/// jnz r1 r2
	Jnz     (Register, Register),
	/// restrict r ρ
	Restrict(Register, Permission),
	/// subseg r ρ1 ρ2
	Subseg  (Register, RegisterOrWord, RegisterOrWord),
	/// lea r ρ
	Lea     (Register, RegisterOrWord),
	/// add r ρ1 ρ2
	Add     (Register, RegisterOrWord, RegisterOrWord),
	/// sub r ρ1 ρ2
	Sub     (Register, RegisterOrWord, RegisterOrWord),
	/// lt r ρ1 ρ2
	Lt      (Register, RegisterOrWord, RegisterOrWord),
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum RegisterOrWord {
	Register(Register),
	Word(Word),
}
