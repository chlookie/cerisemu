use std::collections::{hash_map::Entry, HashMap};

use super::{
	ast::{Ast, AstInstruction, AstRegisterOrWord, AstRow, AstWord},
	expressions::evaluate,
	CompilationError,
};
use crate::{
	compiler::ast::AstExpression,
	emulator::program::{AddrInt, Address, LabelIdentifier, Register},
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

/// Processes all strings in the AST and converts them to individual characters.
pub fn process_strings_to_chars(mut ast: Ast) -> Result<Ast, CompilationError> {
	ast.rows = ast
		.rows
		.iter()
		.flat_map(|l| match l {
			(AstRow::String(s), _span) => s
				.chars()
				.enumerate()
				.map(|(i, c)| (AstRow::Word(AstWord::Char(c)), (i..i + 1)))
				.collect::<Vec<_>>(),
			_ => vec![l.clone()],
		})
		.collect();

	Ok(ast)
}

/// Desugars all GOTOs into a proper label and the necessary instructions
pub fn desugar_gotos(mut ast: Ast) -> Result<Ast, CompilationError> {
	let mut goto_counter = 1;

	ast.rows = ast
		.rows
		.iter()
		.flat_map(|l| match l {
			(AstRow::Goto(dest_label), span) => {
				let temp_label = format!(":goto{}:", goto_counter);
				goto_counter += 1;

				vec![
					(AstRow::Label(temp_label.clone()), span.clone()),
					(
						AstRow::Instruction(AstInstruction::Lea(
							Register::PC,
							AstRegisterOrWord::Word(AstWord::Expression(AstExpression::OpSub(
								Box::new(AstExpression::OpSub(
									Box::new(AstExpression::Label(dest_label.clone())),
									Box::new(AstExpression::Label(temp_label)),
								)),
								Box::new(AstExpression::Integer(1)),
							))),
						)),
						span.clone(),
					),
				]
			}
			_ => vec![l.clone()],
		})
		.collect();

	Ok(ast)
}

/// Removes all label lines from the AST, computes their position and adds them to the label map at the top of the AST.
pub fn extract_labels(mut ast: Ast) -> Result<Ast, CompilationError> {
	let mut i = 0;
	while i < ast.rows.len() {
		// Check that the line is a label line
		if let (AstRow::Label(label), span) = &ast.rows[i] {
			// If so, add it to the label map and remove it from the AST
			if let Entry::Vacant(e) = ast.labels.entry(label.to_owned()) {
				e.insert(Address(i as AddrInt));
				ast.rows.remove(i);
			} else {
				return Err(CompilationError::new(
					"extracting labels",
					"repeated label",
					span.to_owned(),
				));
			}
		} else {
			i += 1;
		}
	}

	Ok(ast)
}

/// Evaluates all expressions in the AST, converting them to an Integer and filling label identifiers where needed.
pub fn evaluate_expressions(mut ast: Ast) -> Result<Ast, CompilationError> {
	ast.rows = ast
		.rows
		.iter()
		.cloned()
		.map(|(row, span)| {
			Ok((
				match row {
					AstRow::Instruction(inst) => AstRow::Instruction(
						evaluate_instruction(inst, &ast.labels)
							.map_err(|s| CompilationError::new("evaluating expressions", &s, span.clone()))?,
					),
					AstRow::Word(AstWord::Expression(e)) => {
						AstRow::Word(AstWord::Integer(evaluate(e, &ast.labels).map_err(|s| {
							CompilationError::new("evaluating expressions", &s, span.clone())
						})?))
					}
					_ => row,
				},
				span,
			))
		})
		.collect::<Result<Vec<_>, CompilationError>>()?;

	Ok(ast)
}

fn evaluate_instruction(
	inst: AstInstruction,
	env: &HashMap<LabelIdentifier, Address>,
) -> Result<AstInstruction, String> {
	Ok(match inst {
		// Single expression instructions
		AstInstruction::Mov(r, p) => AstInstruction::Mov(r, evaluate_register_or_word(p, env)?),
		AstInstruction::Store(r, p) => AstInstruction::Store(r, evaluate_register_or_word(p, env)?),
		AstInstruction::Lea(r, p) => AstInstruction::Lea(r, evaluate_register_or_word(p, env)?),

		// Double expression instructions
		AstInstruction::Subseg(r, p1, p2) => AstInstruction::Subseg(
			r,
			evaluate_register_or_word(p1, env)?,
			evaluate_register_or_word(p2, env)?,
		),

		AstInstruction::Add(r, p1, p2) => AstInstruction::Add(
			r,
			evaluate_register_or_word(p1, env)?,
			evaluate_register_or_word(p2, env)?,
		),

		AstInstruction::Sub(r, p1, p2) => AstInstruction::Sub(
			r,
			evaluate_register_or_word(p1, env)?,
			evaluate_register_or_word(p2, env)?,
		),

		AstInstruction::Lt(r, p1, p2) => AstInstruction::Lt(
			r,
			evaluate_register_or_word(p1, env)?,
			evaluate_register_or_word(p2, env)?,
		),

		// Anything else
		_ => inst,
	})
}

fn evaluate_register_or_word(
	reg_or_word: AstRegisterOrWord,
	env: &HashMap<LabelIdentifier, Address>,
) -> Result<AstRegisterOrWord, String> {
	Ok(match reg_or_word {
		AstRegisterOrWord::Word(AstWord::Expression(e)) => AstRegisterOrWord::Word(AstWord::Integer(evaluate(e, env)?)),
		_ => reg_or_word,
	})
}
