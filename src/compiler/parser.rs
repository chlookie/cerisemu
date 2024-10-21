use logos::{Lexer, Span};

use super::ast::{Ast, AstInstruction, AstRegisterOrWord, AstRow, AstWord};
use super::expressions::parse_expression;
use super::tokens::{InstructionToken, Token};
use super::CompilationError;
use crate::emulator::permission::Permission;
use crate::emulator::program::{LabelIdentifier, Register};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

/// Parses an entire program from a logos lexer into an AST.
/// Handles parsing labels and separating rows, which are then parsed by parse_row()
pub fn parse_program(l: &mut Lexer<'_, Token>) -> Result<Ast, CompilationError> {
	let mut rows = Vec::new();

	let mut expect_instruction = true;
	let mut start_of_line = true;

	while let Some(token) = l.next() {
		if let Ok(token) = token {
			match token {
				Token::Identifier(label) if start_of_line => {
					if let Some(Ok(Token::Colon)) = l.next() {
						rows.push((AstRow::Label(label), l.span()));
					} else {
						return Err(CompilationError::new(
							"parsing progam",
							"unexpected token, expected COLON",
							l.span(),
						));
					}

					start_of_line = false;
				}

				Token::Comma if !expect_instruction => {
					expect_instruction = true;
				}

				Token::LineBreak => {
					expect_instruction = true;
					start_of_line = true;
				}

				_ if expect_instruction => {
					rows.push(parse_row(token, l)?);
					expect_instruction = false;
					start_of_line = false;
				}

				_ => {
					return Err(CompilationError::new(
						"parsing progam",
						"unexpected token here",
						l.span(),
					))
				}
			}
		} else {
			return Err(CompilationError::new(
				"parsing program",
				"unexpected syntax error here",
				l.span(),
			));
		}
	}

	Ok(Ast {
		rows,
		..Default::default()
	})
}

/// Parses a "row", which is either a single instruction and its arguments, or direct data (integer, char or string).
/// A row can be on its own line, or multiple rows on a line can be separated by a comma.
/// Assumes the first token of the row has already been parsed.
#[rustfmt::skip]
fn parse_row(first_token: Token, l: &mut Lexer<'_, Token>) -> Result<(AstRow, Span), CompilationError> {
	let row_start = l.span().start;

	let row = match first_token {
		Token::Empty          => AstRow::default(),
		Token::Goto           => AstRow::Goto(parse_goto_label_ident(l)?),
		Token::Instruction(_) => AstRow::Instruction(parse_instruction(first_token, l)?),
		Token::Character(c)   => AstRow::Word(AstWord::Char(c)),
		Token::Integer(i)     => AstRow::Word(AstWord::Integer(i)),
		Token::LeftBracket    => AstRow::Word(AstWord::Expression(parse_expression(l)?)),
		Token::String(s)      => AstRow::String(s),
		_ => return Err(CompilationError::new("parsing row", "unexpected token, expected instruction or data", l.span())),
	};

	let row_end = l.span().end;
	Ok((row, (row_start..row_end)))
}

/// Parses a single goto row label identifier
fn parse_goto_label_ident(l: &mut Lexer<'_, Token>) -> Result<LabelIdentifier, CompilationError> {
	if let Some(Ok(token)) = l.next() {
		match token {
			Token::Identifier(label) => Ok(label),
			_ => Err(CompilationError::new(
				"parsing row",
				"unexpected token, expected label identifier",
				l.span(),
			)),
		}
	} else {
		Err(CompilationError::new(
			"parsing register",
			"unexpected syntax error here",
			l.span(),
		))
	}
}

/// Parses an instruction.
/// Assumes the first token of the instruction has already been parsed.
#[rustfmt::skip]
fn parse_instruction(instruction_token: Token, l: &mut Lexer<'_, Token>) -> Result<AstInstruction, CompilationError> {
	match instruction_token {
		Token::Instruction(InstructionToken::Fail)     => Ok(AstInstruction::Fail),
		Token::Instruction(InstructionToken::Halt)     => Ok(AstInstruction::Halt),
		Token::Instruction(InstructionToken::Mov)      => Ok(AstInstruction::Mov     (parse_reg(l)?, parse_reg_or_word(l)?)),
		Token::Instruction(InstructionToken::Load)     => Ok(AstInstruction::Load    (parse_reg(l)?, parse_reg(l)?)),
		Token::Instruction(InstructionToken::Store)    => Ok(AstInstruction::Store   (parse_reg(l)?, parse_reg_or_word(l)?)),
		Token::Instruction(InstructionToken::Jmp)      => Ok(AstInstruction::Jmp     (parse_reg(l)?)),
		Token::Instruction(InstructionToken::Jnz)      => Ok(AstInstruction::Jnz     (parse_reg(l)?, parse_reg(l)?)),
		Token::Instruction(InstructionToken::Restrict) => Ok(AstInstruction::Restrict(parse_reg(l)?, parse_permission(l)?)),
		Token::Instruction(InstructionToken::Subseg)   => Ok(AstInstruction::Subseg  (parse_reg(l)?, parse_reg_or_word(l)?, parse_reg_or_word(l)?)),
		Token::Instruction(InstructionToken::Lea)      => Ok(AstInstruction::Lea     (parse_reg(l)?, parse_reg_or_word(l)?)),
		Token::Instruction(InstructionToken::Add)      => Ok(AstInstruction::Add     (parse_reg(l)?, parse_reg_or_word(l)?, parse_reg_or_word(l)?)),
		Token::Instruction(InstructionToken::Sub)      => Ok(AstInstruction::Sub     (parse_reg(l)?, parse_reg_or_word(l)?, parse_reg_or_word(l)?)),
		Token::Instruction(InstructionToken::Lt)       => Ok(AstInstruction::Lt      (parse_reg(l)?, parse_reg_or_word(l)?, parse_reg_or_word(l)?)),
		Token::Instruction(InstructionToken::Getp)     => Ok(AstInstruction::Getp    (parse_reg(l)?, parse_reg(l)?)),
		Token::Instruction(InstructionToken::Getb)     => Ok(AstInstruction::Getb    (parse_reg(l)?, parse_reg(l)?)),
		Token::Instruction(InstructionToken::Gete)     => Ok(AstInstruction::Gete    (parse_reg(l)?, parse_reg(l)?)),
		Token::Instruction(InstructionToken::Geta)     => Ok(AstInstruction::Geta    (parse_reg(l)?, parse_reg(l)?)),
		Token::Instruction(InstructionToken::Isptr)    => Ok(AstInstruction::Isptr   (parse_reg(l)?, parse_reg(l)?)),
		_ => Err(CompilationError::new("parsing instruction", "unexpected token, expected instruction", l.span())),
	}
}

/// Parses either a register or a data word.
#[rustfmt::skip]
fn parse_reg_or_word(l: &mut Lexer<'_, Token>) -> Result<AstRegisterOrWord, CompilationError> {
	if let Some(Ok(token)) = l.next() {
		match token {
			Token::Integer(i)         => Ok(AstRegisterOrWord::Word(AstWord::Integer(i))),
			Token::Character(c)       => Ok(AstRegisterOrWord::Word(AstWord::Char(c))),
			
			Token::RegisterPC         => Ok(AstRegisterOrWord::Register(Register::PC)),
			Token::RegisterGeneral(r) => Ok(AstRegisterOrWord::Register(Register::R(r))),
			
			Token::LeftBracket        => Ok(AstRegisterOrWord::Word(AstWord::Expression(parse_expression(l)?))),
			
			_ => Err(CompilationError::new("parsing register or word", "unexpected token, expected value", l.span())),
		}
	} else {
		Err(CompilationError::new("parsing register or word", "unexpected syntax error here", l.span()))
	}
}

/// Parses a register.
#[rustfmt::skip]
fn parse_reg(l: &mut Lexer<'_, Token>) -> Result<Register, CompilationError> {
	if let Some(Ok(token)) = l.next() {
		match token {
			Token::RegisterPC         => Ok(Register::PC),
			Token::RegisterGeneral(r) => Ok(Register::R(r)),
			_ => Err(CompilationError::new("parsing register", "unexpected token, expected register", l.span())),
		}
	} else {
		Err(CompilationError::new("parsing register", "unexpected syntax error here", l.span()))
	}
}

/// Parses a permission
#[rustfmt::skip]
fn parse_permission(l: &mut Lexer<'_, Token>) -> Result<Permission, CompilationError> {
	if let Some(Ok(token)) = l.next() {
		match token {
			Token::PermissionO   => Ok(Permission::O),
			Token::PermissionE   => Ok(Permission::E),
			Token::PermissionRO  => Ok(Permission::RO),
			Token::PermissionRX  => Ok(Permission::RX),
			Token::PermissionRW  => Ok(Permission::RW),
			Token::PermissionRWX => Ok(Permission::RWX),
			_ => Err(CompilationError::new("parsing permission", "unexpected token, expected permission", l.span())),
		}
	} else {
		Err(CompilationError::new("parsing permission", "unexpected syntax error here", l.span()))
	}
}
