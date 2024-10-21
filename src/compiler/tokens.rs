use logos::{Lexer, Logos};

use crate::emulator::program::{RegInt, WordInt};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

/// All the tokens that can be tokenized into from the source code.
/// Is defined from a specific string token or from a regex expression. If there's any conflicts, logos fails at compile time.
#[derive(Clone, Debug, Logos)]
#[logos(skip r"[ \t\f]+")]
pub enum Token {
	/// A comment, a semi-colon followed by any number of characters until the end of the line.
	/// Is automatically skipped by the lexer
	#[regex(r";[^\n\r]*", logos::skip)]
	Comment,

	/// A line break, so any permutation of \n and \r for one of more empty lines.
	#[regex(r"[\n|\r|\r\n]+")]
	LineBreak,

	/// A comma ,
	#[token(",")]
	Comma,

	/// A colon :
	#[token(":")]
	Colon,

	/// A plus +
	#[token("+")]
	Plus,
	/// A minus -
	#[token("-")]
	Minus,
	/// A star *
	#[token("*")]
	Star,
	/// A forward slash /
	#[token("/")]
	Slash,

	/// A left parentesis (
	#[token("(")]
	LeftParenthesis,
	/// A right parentesis )
	#[token(")")]
	RightParenthesis,

	/// A left bracket [
	#[token("[")]
	LeftBracket,
	/// A right bracket ]
	#[token("]")]
	RightBracket,

	/// An integer. Cab be parsed from
	// A decimal (base 10) integer.
	// Cannot start with 0. Can also contain _ as a delimited between digits.
	#[regex(r"([1-9][_0-9]*|0)", integer_callback)]
	// A hexadecimal (base 16) integer.
	// Must start with the prefix 0x or 0X.
	// Must be made up of any of the digits 0 1 2 3 4 5 6 7 8 9 a b c d e f.
	// Can also contain _ as a delimited between digits.
	#[regex(r"0[xX][0-9a-fA-F][_0-9a-fA-F]*", hex_callback)]
	// An octal (base 8) integer.
	// Must start with the prefix 0o or 0O (zero followed by the letter o).
	// Must be made up of any of the digits 0 1 2 3 4 5 6 7.
	// Can also contain _ as a delimited between digits.
	#[regex(r"0[oO][0-7][_0-7]*", oct_callback)]
	// A binary (base 2) integer.
	// Must start with the prefix 0b or 0B.
	// Must be made up of any of the digits 0 1.
	// Can also contain _ as a delimited between digits.
	#[regex(r"0[bB][0-1][_0-1]*", bin_callback)]
	Integer(WordInt),

	/// A character, between two quotes 'c'.
	#[regex(r"'(?:[^']|\')'", char_callback)]
	Character(char),

	/// A string, between two double-quotes "Hello World".
	#[regex(r#""([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#, string_callback)]
	String(String),

	/// An identifier.
	#[regex(r"[\p{XID_Start}_#][\p{XID_Continue}#]*", ident_callback)]
	Identifier(String),

	// The GOTO synctatic sugar token.
	#[token("goto", priority = 10, ignore(case))]
	Goto,

	// An empty row.
	#[token("empty", priority = 10, ignore(case))]
	Empty,

	/// The zero permission, represented by the token O.
	#[token("O", priority = 10)]
	PermissionO,
	/// The execute permission, represented by the token E.
	#[token("E", priority = 10)]
	PermissionE,
	/// The read-only permission, represented by the token RO.
	#[token("RO", priority = 10)]
	PermissionRO,
	/// The read/execute permission, represented by the token RX.
	#[token("RX", priority = 10)]
	PermissionRX,
	/// The read/write permission, represented by the token RW.
	#[token("RW", priority = 10)]
	PermissionRW,
	/// The read/write/execute permission, represented by the token RWX.
	#[token("RWX", priority = 10)]
	PermissionRWX,

	/// The PC register
	#[token("PC", priority = 10, ignore(case))]
	RegisterPC,
	/// Any general-purpose register, of the form R0, r1, R2, ...
	#[regex(r"[rR]\d+", register_callback)]
	RegisterGeneral(RegInt),

	/// Any instruction, represented by any of the below tokens.
	#[rustfmt::skip]
	#[token("fail",     |_| InstructionToken::Fail,     ignore(case))]
	#[token("halt",     |_| InstructionToken::Halt,     ignore(case))]
	#[token("mov",      |_| InstructionToken::Mov,      ignore(case))]
	#[token("load",     |_| InstructionToken::Load,     ignore(case))]
	#[token("store",    |_| InstructionToken::Store,    ignore(case))]
	#[token("jmp",      |_| InstructionToken::Jmp,      ignore(case))]
	#[token("jnz",      |_| InstructionToken::Jnz,      ignore(case))]
	#[token("restrict", |_| InstructionToken::Restrict, ignore(case))]
	#[token("subseg",   |_| InstructionToken::Subseg,   ignore(case))]
	#[token("lea",      |_| InstructionToken::Lea,      ignore(case))]
	#[token("add",      |_| InstructionToken::Add,      ignore(case))]
	#[token("sub",      |_| InstructionToken::Sub,      ignore(case))]
	#[token("lt",       |_| InstructionToken::Lt,       ignore(case))]
	#[token("getp",     |_| InstructionToken::Getp,     ignore(case))]
	#[token("getb",     |_| InstructionToken::Getb,     ignore(case))]
	#[token("gete",     |_| InstructionToken::Gete,     ignore(case))]
	#[token("geta",     |_| InstructionToken::Geta,     ignore(case))]
	#[token("isptr",    |_| InstructionToken::Isptr,    ignore(case))]
	Instruction(InstructionToken),
}

/// The instructions that Token::Instruction can take
#[derive(Clone, Debug)]
pub enum InstructionToken {
	Fail,
	Halt,
	Mov,
	Load,
	Store,
	Jmp,
	Jnz,
	Restrict,
	Subseg,
	Lea,
	Add,
	Sub,
	Lt,
	Getp,
	Getb,
	Gete,
	Geta,
	Isptr,
}

/// The callback to convert a decimal integer string to int.
/// Simply parses the string to int.
fn integer_callback(lexer: &mut Lexer<Token>) -> WordInt {
	lexer.slice().replace('_', "").parse().unwrap()
}

/// The callback to convert a heximal integer string to int.
/// Removes the "0x" prefix and parses the integer with radix 16
fn hex_callback(lexer: &mut Lexer<Token>) -> WordInt {
	WordInt::from_str_radix(&lexer.slice()[2..].replace('_', ""), 16).unwrap()
}

/// The callback to convert an octal integer string to int.
/// Removes the "0o" prefix and parses the integer with radix 8
fn oct_callback(lexer: &mut Lexer<Token>) -> WordInt {
	WordInt::from_str_radix(&lexer.slice()[2..].replace('_', ""), 8).unwrap()
}

/// The callback to convert a binary integer string to int.
/// Removes the "0b" prefix and parses the integer with radix 2
fn bin_callback(lexer: &mut Lexer<Token>) -> WordInt {
	WordInt::from_str_radix(&lexer.slice()[2..].replace('_', ""), 2).unwrap()
}

/// The callback to extract a char.
/// Removes the quotes ' surrounding the char and converts to a char.
/// If there isn't exactly one character between the quotes, return None to make sure logos errors out.
fn char_callback(lexer: &mut Lexer<Token>) -> Option<char> {
	let slice = lexer.slice();
	let mut chars = slice[1..slice.len() - 1].chars();
	let count = chars.clone().count();

	if count == 0 || count > 1 {
		None
	} else {
		Some(chars.next().unwrap())
	}
}

/// The callback to extract a string.
/// Simply removes the quotes " surrounding the string.
fn string_callback(lexer: &mut Lexer<Token>) -> String {
	let slice = lexer.slice();
	slice[1..slice.len() - 1].to_owned()
}

/// The callback to extract an identifier.
/// Simply gets the string
fn ident_callback(lexer: &mut Lexer<Token>) -> String {
	lexer.slice().to_owned()
}

/// The callback to extract the number x of a register Rx.
fn register_callback(lexer: &mut Lexer<Token>) -> RegInt {
	lexer.slice()[1..].parse::<RegInt>().unwrap()
}
