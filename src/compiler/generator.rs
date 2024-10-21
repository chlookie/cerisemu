use crate::emulator::{
	instruction::{Instruction, RegisterOrWord},
	program::{Program, Row, Word},
	signed::Signed,
};

use super::{
	ast::{Ast, AstInstruction, AstRegisterOrWord, AstRow, AstWord},
	CompilationError,
};

pub fn generate_program_from_ast(ast: Ast) -> Result<Program, CompilationError> {
	Ok(Program {
		rows: ast
			.rows
			.into_iter()
			.map(|(row, _)| -> Row {
				match row {
					AstRow::Instruction(instruction) => Row::Instruction(generate_instruction(instruction)),
					AstRow::Word(word) => Row::Word(generate_word(word)),
					_ => panic!(
						"Unprocessed elements in AST! This should not happen, AST was not pre-processed properly."
					),
				}
			})
			.collect(),
	})
}

#[rustfmt::skip]
fn generate_word(word: AstWord) -> Word {
	match word {
		AstWord::Integer(i)    => Word::Integer(i),
		AstWord::Char(c)       => Word::Char(c),
		AstWord::Capability(c) => Word::Capability(Signed::new_unsigned(c)),
		_ => panic!("Unprocessed elements in AST! This should not happen, AST was not processed properly."),
	}
}

#[rustfmt::skip]
fn generate_instruction(instruction: AstInstruction) -> Instruction {
	match instruction {
		AstInstruction::Fail              => Instruction::Fail,
		AstInstruction::Halt              => Instruction::Halt,
		AstInstruction::Mov(r, p)         => Instruction::Mov(r, generate_reg_or_word(p)),
		AstInstruction::Load(r1, r2)      => Instruction::Load(r1, r2),
		AstInstruction::Store(r, p)       => Instruction::Store(r, generate_reg_or_word(p)),
		AstInstruction::Jmp(r)            => Instruction::Jmp(r),
		AstInstruction::Jnz(r1, r2)       => Instruction::Jnz(r1, r2),
		AstInstruction::Restrict(r, p)    => Instruction::Restrict(r, p),
		AstInstruction::Subseg(r, p1, p2) => Instruction::Subseg(r, generate_reg_or_word(p1), generate_reg_or_word(p2)),
		AstInstruction::Lea(r, p)         => Instruction::Lea(r, generate_reg_or_word(p)),
		AstInstruction::Add(r, p1, p2)    => Instruction::Add(r, generate_reg_or_word(p1), generate_reg_or_word(p2)),
		AstInstruction::Sub(r, p1, p2)    => Instruction::Sub(r, generate_reg_or_word(p1), generate_reg_or_word(p2)),
		AstInstruction::Lt(r, p1, p2)     => Instruction::Lt(r, generate_reg_or_word(p1), generate_reg_or_word(p2)),
		AstInstruction::Getp(r1, r2)      => Instruction::Getp(r1, r2),
		AstInstruction::Getb(r1, r2)      => Instruction::Getb(r1, r2),
		AstInstruction::Gete(r1, r2)      => Instruction::Gete(r1, r2),
		AstInstruction::Geta(r1, r2)      => Instruction::Geta(r1, r2),
		AstInstruction::Isptr(r1, r2)     => Instruction::Isptr(r1, r2),
	}
}

fn generate_reg_or_word(reg_or_word: AstRegisterOrWord) -> RegisterOrWord {
	match reg_or_word {
		AstRegisterOrWord::Register(r) => RegisterOrWord::Register(r),
		AstRegisterOrWord::Word(w) => RegisterOrWord::Word(generate_word(w)),
	}
}
