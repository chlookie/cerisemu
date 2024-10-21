use std::{
	cmp::Ordering,
	collections::HashMap,
	fmt::{Display, Formatter, Result},
	ops::{
		Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Deref, DerefMut, Div,
		DivAssign, Mul, MulAssign, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
	},
};

use crate::{
	compiler::ast::{Ast, AstRow, AstWord},
	emulator::{
		instruction::{Instruction, RegisterOrWord},
		machine::{Interrupt, State},
		permission::Permission,
		program::{AddrInt, Address, Capability, Program, Register, Row, Word, WordInt},
	},
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub trait Lattice: PartialOrd {
	fn top() -> Self;
	fn bot() -> Self;

	fn join(&self, other: Self) -> Self;
	fn meet(&self, other: Self) -> Self;
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub fn pretty_hashmap<K, V>(hashmap: &HashMap<K, V>) -> String
where
	K: Display + Ord,
	V: Display,
{
	let mut out = String::new();

	let mut entries = hashmap.iter().collect::<Vec<(&K, &V)>>();
	entries.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));

	for (k, v) in entries {
		out.push_str(&format!("\n{}: {}", k, v));
	}

	out
}

pub fn indent_string(string: &str, indent: &str) -> String {
	string
		.lines()
		.map(|s| indent.to_owned() + s)
		.collect::<Vec<_>>()
		.join("\n")
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

impl Display for Ast {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		let rows = self.rows.iter();

		let instructions = rows
			.clone()
			.filter(|l| matches!(l, (AstRow::Instruction(_), _)))
			.count();

		let data_words = rows.clone().filter(|l| matches!(l, (AstRow::Word(_), _))).count();

		let label_rows = rows.filter(|l| matches!(l, (AstRow::Label(_), _))).count();

		let labels = self
			.labels
			.iter()
			.map(|(k, v)| format!("{}:{}", k, v.0))
			.collect::<Vec<String>>();

		f.pad(&format!(
			"AST ({instructions} instructions, {data_words} data words, {label_rows} unprocessed labels; processed labels: {labels:?})",
		))
	}
}

impl Display for Program {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		f.pad(&format!("{:?}", self))
	}
}

impl Display for Row {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Row::Word(word) => f.pad(&format!("Word({})", word)),
			Row::Instruction(instruction) => f.pad(&format!("Instruction({})", instruction)),
		}
	}
}

impl Display for Instruction {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Instruction::Fail => f.pad("fail"),
			Instruction::Halt => f.pad("halt"),
			Instruction::Mov(r, p) => f.pad(&format!("mov {} {}", r, p)),
			Instruction::Load(r1, r2) => f.pad(&format!("load {} {}", r1, r2)),
			Instruction::Store(r, p) => f.pad(&format!("store {} {}", r, p)),
			Instruction::Jmp(r) => f.pad(&format!("jmp {}", r)),
			Instruction::Jnz(r1, r2) => f.pad(&format!("jnz {} {}", r1, r2)),
			Instruction::Restrict(r1, perm) => f.pad(&format!("restrict {} {}", r1, perm)),
			Instruction::Subseg(r, p1, p2) => f.pad(&format!("subseg {} {} {}", r, p1, p2)),
			Instruction::Lea(r, p) => f.pad(&format!("lea {} {}", r, p)),
			Instruction::Add(r, p1, p2) => f.pad(&format!("add {} {} {}", r, p1, p2)),
			Instruction::Sub(r, p1, p2) => f.pad(&format!("sub {} {} {}", r, p1, p2)),
			Instruction::Lt(r, p1, p2) => f.pad(&format!("lt {} {} {}", r, p1, p2)),
			Instruction::Getp(r1, r2) => f.pad(&format!("getp {} {}", r1, r2)),
			Instruction::Getb(r1, r2) => f.pad(&format!("getb {} {}", r1, r2)),
			Instruction::Gete(r1, r2) => f.pad(&format!("gete {} {}", r1, r2)),
			Instruction::Geta(r1, r2) => f.pad(&format!("geta {} {}", r1, r2)),
			Instruction::Isptr(r1, r2) => f.pad(&format!("isptr {} {}", r1, r2)),
		}
	}
}

impl Display for RegisterOrWord {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		match self {
			RegisterOrWord::Register(r) => f.pad(&format!("{}", r)),
			RegisterOrWord::Word(w) => f.pad(&format!("{}", w)),
		}
	}
}

impl Display for Register {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		match self {
			Register::PC => f.pad("PC"),
			Register::R(n) => f.pad(&format!("R{}", n)),
		}
	}
}

impl Display for Word {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Word::Integer(i) => f.pad(&format!("{}", i)),
			Word::Char(c) => f.pad(&format!("'{}'", c)),
			Word::Capability(c) => f.pad(&format!("{}", c)),
			Word::Permission(p) => f.pad(&format!("{}", p)),
		}
	}
}

impl Display for Capability {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		f.pad(&format!(
			"({}, {}, {}, {})",
			self.perm, self.base, self.end, self.address
		))
	}
}

impl Display for Permission {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		f.pad(&format!("{:?}", self))
	}
}

impl Display for Address {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		f.pad(&format!("{:#x}", self.0))
	}
}

impl Display for State {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.pad(&format!("{:?}", self))
	}
}

impl Display for Interrupt {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		match self {
			Interrupt::Halt => f.pad("HALT"),
			Interrupt::Fail => f.pad("FAIL"),
		}
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

impl Default for AstWord {
	fn default() -> Self {
		AstWord::Integer(WordInt::default())
	}
}

impl Default for AstRow {
	fn default() -> Self {
		AstRow::Word(AstWord::default())
	}
}

impl Default for Row {
	fn default() -> Self {
		Row::Word(Word::default())
	}
}

impl Default for Word {
	fn default() -> Self {
		Word::Integer(WordInt::default())
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[macro_export]
macro_rules! hashmap {
    // map-like
    ($($k:expr => $v:expr),* $(,)?) => {{
        core::convert::From::from([$(($k, $v),)*])
    }};
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

// Warning: High brain damage ahead

#[rustfmt::skip] impl Deref for Address {type Target = AddrInt; fn deref(&self) -> &Self::Target {&self.0}}
#[rustfmt::skip] impl DerefMut for Address {fn deref_mut(&mut self) -> &mut Self::Target {&mut self.0}}

#[rustfmt::skip] impl From<AddrInt> for Address {fn from(value: AddrInt) -> Self {Address(value)}}
#[rustfmt::skip] impl From<Address> for AddrInt {fn from(value: Address) -> Self {value.0}}

#[rustfmt::skip] impl PartialEq<AddrInt> for Address {fn eq(&self, other: &AddrInt) -> bool {self.0 == *other}}
#[rustfmt::skip] impl PartialEq<Address> for AddrInt {fn eq(&self, other: &Address) -> bool {*self == other.0}}
#[rustfmt::skip] impl PartialOrd<AddrInt> for Address {fn partial_cmp(&self, other: &AddrInt) -> Option<Ordering> {self.0.partial_cmp(other)}}
#[rustfmt::skip] impl PartialOrd<Address> for AddrInt {fn partial_cmp(&self, other: &Address) -> Option<Ordering> {self.partial_cmp(&other.0)}}

#[rustfmt::skip] impl Add    for Address {type Output = Self; fn add   (self, rhs: Self) -> Self::Output {Address(self.0 + rhs.0)}}
#[rustfmt::skip] impl Sub    for Address {type Output = Self; fn sub   (self, rhs: Self) -> Self::Output {Address(self.0 - rhs.0)}}
#[rustfmt::skip] impl BitAnd for Address {type Output = Self; fn bitand(self, rhs: Self) -> Self::Output {Address(self.0 & rhs.0)}}
#[rustfmt::skip] impl BitOr  for Address {type Output = Self; fn bitor (self, rhs: Self) -> Self::Output {Address(self.0 | rhs.0)}}
#[rustfmt::skip] impl BitXor for Address {type Output = Self; fn bitxor(self, rhs: Self) -> Self::Output {Address(self.0 ^ rhs.0)}}
#[rustfmt::skip] impl Mul    for Address {type Output = Self; fn mul   (self, rhs: Self) -> Self::Output {Address(self.0 * rhs.0)}}
#[rustfmt::skip] impl Div    for Address {type Output = Self; fn div   (self, rhs: Self) -> Self::Output {Address(self.0 / rhs.0)}}
#[rustfmt::skip] impl Rem    for Address {type Output = Self; fn rem   (self, rhs: Self) -> Self::Output {Address(self.0 % rhs.0)}}
#[rustfmt::skip] impl Shl    for Address {type Output = Self; fn shl   (self, rhs: Self) -> Self::Output {Address(self.0 << rhs.0)}}
#[rustfmt::skip] impl Shr    for Address {type Output = Self; fn shr   (self, rhs: Self) -> Self::Output {Address(self.0 >> rhs.0)}}

#[rustfmt::skip] impl Add<AddrInt>    for Address {type Output = Self; fn add   (self, rhs: AddrInt) -> Self::Output {Address(self.0 + rhs)}}
#[rustfmt::skip] impl Sub<AddrInt>    for Address {type Output = Self; fn sub   (self, rhs: AddrInt) -> Self::Output {Address(self.0 - rhs)}}
#[rustfmt::skip] impl BitAnd<AddrInt> for Address {type Output = Self; fn bitand(self, rhs: AddrInt) -> Self::Output {Address(self.0 & rhs)}}
#[rustfmt::skip] impl BitOr<AddrInt>  for Address {type Output = Self; fn bitor (self, rhs: AddrInt) -> Self::Output {Address(self.0 | rhs)}}
#[rustfmt::skip] impl BitXor<AddrInt> for Address {type Output = Self; fn bitxor(self, rhs: AddrInt) -> Self::Output {Address(self.0 ^ rhs)}}
#[rustfmt::skip] impl Mul<AddrInt>    for Address {type Output = Self; fn mul   (self, rhs: AddrInt) -> Self::Output {Address(self.0 * rhs)}}
#[rustfmt::skip] impl Div<AddrInt>    for Address {type Output = Self; fn div   (self, rhs: AddrInt) -> Self::Output {Address(self.0 / rhs)}}
#[rustfmt::skip] impl Rem<AddrInt>    for Address {type Output = Self; fn rem   (self, rhs: AddrInt) -> Self::Output {Address(self.0 % rhs)}}
#[rustfmt::skip] impl Shl<AddrInt>    for Address {type Output = Self; fn shl   (self, rhs: AddrInt) -> Self::Output {Address(self.0 << rhs)}}
#[rustfmt::skip] impl Shr<AddrInt>    for Address {type Output = Self; fn shr   (self, rhs: AddrInt) -> Self::Output {Address(self.0 >> rhs)}}

#[rustfmt::skip] impl AddAssign    for Address {fn add_assign   (&mut self, rhs: Self) {self.0 += rhs.0}}
#[rustfmt::skip] impl SubAssign    for Address {fn sub_assign   (&mut self, rhs: Self) {self.0 -= rhs.0}}
#[rustfmt::skip] impl BitAndAssign for Address {fn bitand_assign(&mut self, rhs: Self) {self.0 &= rhs.0}}
#[rustfmt::skip] impl BitOrAssign  for Address {fn bitor_assign (&mut self, rhs: Self) {self.0 |= rhs.0}}
#[rustfmt::skip] impl BitXorAssign for Address {fn bitxor_assign(&mut self, rhs: Self) {self.0 ^= rhs.0}}
#[rustfmt::skip] impl MulAssign    for Address {fn mul_assign   (&mut self, rhs: Self) {self.0 *= rhs.0}}
#[rustfmt::skip] impl DivAssign    for Address {fn div_assign   (&mut self, rhs: Self) {self.0 /= rhs.0}}
#[rustfmt::skip] impl RemAssign    for Address {fn rem_assign   (&mut self, rhs: Self) {self.0 %= rhs.0}}
#[rustfmt::skip] impl ShlAssign    for Address {fn shl_assign   (&mut self, rhs: Self) {self.0 <<= rhs.0}}
#[rustfmt::skip] impl ShrAssign    for Address {fn shr_assign   (&mut self, rhs: Self) {self.0 >>= rhs.0}}

#[rustfmt::skip] impl AddAssign<AddrInt>    for Address {fn add_assign   (&mut self, rhs: AddrInt) {self.0 += rhs}}
#[rustfmt::skip] impl SubAssign<AddrInt>    for Address {fn sub_assign   (&mut self, rhs: AddrInt) {self.0 -= rhs}}
#[rustfmt::skip] impl BitAndAssign<AddrInt> for Address {fn bitand_assign(&mut self, rhs: AddrInt) {self.0 &= rhs}}
#[rustfmt::skip] impl BitOrAssign<AddrInt>  for Address {fn bitor_assign (&mut self, rhs: AddrInt) {self.0 |= rhs}}
#[rustfmt::skip] impl BitXorAssign<AddrInt> for Address {fn bitxor_assign(&mut self, rhs: AddrInt) {self.0 ^= rhs}}
#[rustfmt::skip] impl MulAssign<AddrInt>    for Address {fn mul_assign   (&mut self, rhs: AddrInt) {self.0 *= rhs}}
#[rustfmt::skip] impl DivAssign<AddrInt>    for Address {fn div_assign   (&mut self, rhs: AddrInt) {self.0 /= rhs}}
#[rustfmt::skip] impl RemAssign<AddrInt>    for Address {fn rem_assign   (&mut self, rhs: AddrInt) {self.0 %= rhs}}
#[rustfmt::skip] impl ShlAssign<AddrInt>    for Address {fn shl_assign   (&mut self, rhs: AddrInt) {self.0 <<= rhs}}
#[rustfmt::skip] impl ShrAssign<AddrInt>    for Address {fn shr_assign   (&mut self, rhs: AddrInt) {self.0 >>= rhs}}
