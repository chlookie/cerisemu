use std::{
	error::Error,
	fmt::{self, Debug, Display, Formatter},
};

use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use logos::{Logos, Span};

use crate::emulator::program::Program;

use self::tokens::Token;

pub mod ast;
pub mod expressions;
pub mod generator;
pub mod parser;
pub mod processor;
pub mod tokens;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub fn compile_unwrapped(source: &str) -> Program {
	compile(source)
		.map_err(|CompilationError { ctx, msg, span }| {
			let mut colors = ColorGenerator::new();
			let a = colors.next();

			// Generate the ariadne fancy-print using the source code and the span returned from the compilation result
			Report::build(ReportKind::Error, (), 12)
				.with_message(format!("Could not compile program, error while {}.", ctx))
				.with_label(Label::new(span.clone()).with_message(msg.clone()).with_color(a))
				.finish()
				.eprint(Source::from(source))
				.unwrap();
		})
		.expect("Compile error.")
}

/// Compile the given source code string
pub fn compile(source: &str) -> Result<Program, CompilationError> {
	// Create the logos lexer.
	// The lexer is responsible for converting the source code to individual tokens that are easier to parse
	let mut lexer = Token::lexer(source);

	// Parse the tokenized source code to an AST
	let mut ast = parser::parse_program(&mut lexer)?;

	// Process and transform the AST in various ways
	let processors = vec![
		processor::process_strings_to_chars,
		processor::desugar_gotos,
		processor::extract_labels,
		processor::evaluate_expressions,
	];
	for proc in processors {
		ast = (proc)(ast)?;
	}

	// Generate the program in internal representation
	let program = generator::generate_program_from_ast(ast)?;

	Ok(program)
}

#[derive(Debug)]
pub struct CompilationError {
	ctx: String,
	msg: String,
	span: Span,
}

impl Error for CompilationError {}

impl Display for CompilationError {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl CompilationError {
	pub fn new(ctx: &str, msg: &str, span: Span) -> Self {
		CompilationError {
			ctx: ctx.to_owned(),
			msg: msg.to_owned(),
			span,
		}
	}
}
