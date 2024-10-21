use core::panic;
use std::collections::HashMap;

use logos::Lexer;

use super::{ast::AstExpression, tokens::Token, CompilationError};
use crate::emulator::program::{Address, LabelIdentifier, WordInt};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

/// Parses a label math expression.
/// Assumes the first bracket [ was already read.
/// Implementation based on https://github.com/erikeidt/erikeidt.github.io/blob/master/The-Double-E-Method.md
pub fn parse_expression(l: &mut Lexer<'_, Token>) -> Result<AstExpression, CompilationError> {
	let mut state = State::Unary;
	let mut operator_stack = vec![Op::OpenParen];
	let mut operand_stack = vec![];

	// While there are tokens to be read
	while let Some(Ok(token)) = l.next() {
		if state == State::Unary {
			// In the unary state, we're expecting either a unary operator or an operand or grouping parenthesis (or others).
			match token {
				// If we see an operator token (e.g. -), then we know it is a unary operator (e.g. unary negation). Push the identified unary operator onto the operator stack. Stay in unary state.
				Token::Minus => operator_stack.push(Op::Neg),
				Token::Plus => operator_stack.push(Op::Pos),

				// If we see an operand (e.g. identifier, constant), so create AST for the operand and push it onto the operand stack. Switch to binary state.
				Token::Identifier(label) => {
					operand_stack.push(AstExpression::Label(label));
					state = State::Binary;
				}
				Token::Integer(i) => {
					operand_stack.push(AstExpression::Integer(i));
					state = State::Binary;
				}

				// If we see an open parenthesis, then push operator for grouping paren onto the operator stack. Stay in unary state.
				Token::LeftParenthesis => operator_stack.push(Op::OpenParen),

				// Anything else means a parsing error
				_ => {
					return Err(CompilationError::new(
						"parsing expression (unary step)",
						"unexpected token",
						l.span(),
					))
				}
			}
		} else {
			// In the binary state, we are expecting binary operators, or close parenthesis (or open paren).
			match token {
				// If we see an operator token (e.g. - or *) in the binary state, then we know we have a binary operator (e.g. subtraction or multiplication). Reduce, then push the identified operator. Switch back to unary state.
				Token::Plus => {
					reduce(Op::Add, &mut operator_stack, &mut operand_stack);
					operator_stack.push(Op::Add);
					state = State::Unary;
				}

				Token::Minus => {
					reduce(Op::Sub, &mut operator_stack, &mut operand_stack);
					operator_stack.push(Op::Sub);
					state = State::Unary;
				}

				Token::Star => {
					reduce(Op::Mul, &mut operator_stack, &mut operand_stack);
					operator_stack.push(Op::Mul);
					state = State::Unary;
				}

				Token::Slash => {
					reduce(Op::Div, &mut operator_stack, &mut operand_stack);
					operator_stack.push(Op::Div);
					state = State::Unary;
				}

				// If we see a close parenthesis, then reduce until matching open parenthesis. Discard grouping paren. Stay in binary state.
				Token::RightParenthesis => {
					reduce(Op::CloseParen, &mut operator_stack, &mut operand_stack);
					state = State::Binary;
				}

				// If we see a close parenthesis, then reduce until matching open parenthesis. Discard grouping paren. Stay in binary state.
				// If the operator stack is empty, we're done.
				Token::RightBracket => {
					reduce(Op::CloseParen, &mut operator_stack, &mut operand_stack);
					state = State::Binary;
					// We're done
					if operand_stack.len() == 1 && operator_stack.is_empty() {
						return Ok(operand_stack.pop().unwrap());
					}
				}

				// Anything else means a parsing error
				_ => {
					return Err(CompilationError::new(
						"parsing expression (unary step)",
						"unexpected token",
						l.span(),
					))
				}
			}
		}
	}

	Err(CompilationError::new(
		"parsing expression",
		"unfinished expression here",
		l.span(),
	))
}

/// Evaluates an expression, filling in any Label Identifiers where needed.
/// Is implemented recursively, because the expressions are supposed to be simple and not deeply nested anyway.
#[rustfmt::skip]
pub fn evaluate(expression: AstExpression, env: &HashMap<LabelIdentifier, Address>) -> Result<WordInt, String> {
Ok(
	match expression {
		AstExpression::Integer(i)    => i,
		AstExpression::Label(label)  => env.get(&label).ok_or(format!("label \"{}\" not found in program", label).to_owned())?.0 as WordInt,
		AstExpression::OpAdd(e1, e2) => evaluate(*e1, env)? + evaluate(*e2, env)?,
		AstExpression::OpSub(e1, e2) => evaluate(*e1, env)? - evaluate(*e2, env)?,
		AstExpression::OpMul(e1, e2) => evaluate(*e1, env)? * evaluate(*e2, env)?,
		AstExpression::OpDiv(e1, e2) => evaluate(*e1, env)? / evaluate(*e2, env)?,
		AstExpression::OpNeg(e1)     => - evaluate(*e1, env)?,
	})
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum State {
	Unary,
	Binary,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Op {
	/// Addition
	Add,
	/// Substraction
	Sub,
	/// Multiplication
	Mul,
	/// Integer division
	Div,
	/// Unary negation
	Neg,
	/// Unary positive
	Pos,
	/// Open/left parenthesis
	OpenParen,
	/// Close/right parenthesis
	CloseParen,
}

fn precedence(op: &Op) -> isize {
	match op {
		Op::Add => 1,
		Op::Sub => 1,
		Op::Mul => 2,
		Op::Div => 2,
		Op::Neg => 3,
		Op::Pos => 3,
		Op::OpenParen => -1,
		Op::CloseParen => -2,
	}
}

fn reduce(new_op: Op, operator_stack: &mut Vec<Op>, operand_stack: &mut Vec<AstExpression>) {
	while operator_stack
		.last()
		.is_some_and(|op| precedence(op) >= precedence(&new_op))
	{
		let op = operator_stack.pop().unwrap();
		match op {
			Op::Add | Op::Sub | Op::Mul | Op::Div => {
				let ast_op = match op {
					Op::Add => AstExpression::OpAdd,
					Op::Sub => AstExpression::OpSub,
					Op::Mul => AstExpression::OpMul,
					Op::Div => AstExpression::OpDiv,
					_ => panic!(),
				};
				let right = operand_stack.pop().unwrap();
				let left = operand_stack.pop().unwrap();
				operand_stack.push((ast_op)(Box::new(left), Box::new(right)));
			}

			Op::Neg => {
				let operand = operand_stack.pop().unwrap();
				operand_stack.push(AstExpression::OpNeg(Box::new(operand)));
			}

			Op::Pos => {
				let operand = operand_stack.pop().unwrap();
				operand_stack.push(operand);
			}

			Op::OpenParen if new_op == Op::CloseParen => break,

			_ => panic!(),
		}
	}
}
