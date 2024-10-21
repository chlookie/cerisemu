use std::{
	fs::OpenOptions,
	io::{self, Read, Write},
	path::PathBuf,
};

use clap::{command, Arg, ArgAction, Command};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

fn main() {
	let matches = cli().get_matches();
	match matches.subcommand() {
		Some(("compile", compile_matches)) => {
			let input = make_reader(compile_matches.get_one::<PathBuf>("in").cloned());
			let output = make_writer(compile_matches.get_one::<PathBuf>("out").cloned());
			cerisemu::compile(input, output)
		}

		Some(("emulate", compile_matches)) => {
			let input = make_reader(compile_matches.get_one::<PathBuf>("in").cloned());
			let output = make_writer(compile_matches.get_one::<PathBuf>("out").cloned());
			let compile = compile_matches.get_flag("compile");
			let dump = compile_matches.get_flag("dump");
			let backtrace = compile_matches.get_flag("backtrace");
			cerisemu::emulate(input, output, compile, dump, backtrace)
		}

		_ => unreachable!("A subcommand hasn't been properly programmed! This should not happen."),
	}
}

fn cli() -> Command {
	command!().propagate_version(true).subcommand_required(true).subcommand(
		Command::new("compile")
			.about("Compile an assembly text to internal program representation.")
			.arg(
				Arg::new("in")
					.long("in")
					.short('i')
					.help("Set the input file for the compiler to read from. Reads from stdin if not specified.")
					.value_parser(clap::value_parser!(PathBuf))
					.action(ArgAction::Set)
			)
			.arg(
				Arg::new("out")
					.long("out")
					.short('o')
					.help("Set the output file for the compiler to write the compiled code to. Writes to stdout if not specified.")
					.value_parser(clap::value_parser!(PathBuf))
					.action(ArgAction::Set)
			),
	).subcommand(Command::new("emulate")
			.about("Emulate a capability machine given a single program or a machine config.")
			.arg(
				Arg::new("in")
					.long("in")
					.short('i')
					.help("Set the input file for the emulator to read from. If the --compile flag is not set, will be interpreted either as a compiled program or a config file. Reads from stdin if not specified.")
					.value_parser(clap::value_parser!(PathBuf))
					.action(ArgAction::Set)
			)
			.arg(
				Arg::new("out")
					.long("out")
					.short('o')
					.help("Set the output file for the emulator to write the finished machine to. Writes to stdout if not specified.")
					.value_parser(clap::value_parser!(PathBuf))
					.action(ArgAction::Set)
			)
			.arg(
				Arg::new("compile")
					.long("compile")
					.short('c')
					.help("Indicates that the input file needs to first be compiled. If set, the input file must be an uncompiled ASM file. If not set, the input file must be a proper RON file.")
					.required(false)
					.action(ArgAction::SetTrue)
			)
			.arg(
				Arg::new("dump")
					.long("dump")
					.short('d')
					.help("Indicates that the machine should be dumped to the specified output after emulation.")
					.required(false)
					.action(ArgAction::SetTrue)
			)
			.arg(
				Arg::new("backtrace")
					.long("backtrace")
					.short('b')
					.help("Indicates that the machine backtrace should be printed after emulation.")
					.required(false)
					.action(ArgAction::SetTrue)
			)
	)
}

pub fn make_reader(in_path: Option<PathBuf>) -> Box<dyn Read> {
	in_path
		.map(|path| {
			Box::new(
				OpenOptions::new()
					.read(true)
					.open(path)
					.expect("Couldn't open input file."),
			) as Box<dyn Read>
		})
		.unwrap_or_else(|| {
			println!("No input path specified, reading from stdin.");
			Box::new(io::stdin()) as Box<dyn Read>
		})
}

pub fn make_writer(out_path: Option<PathBuf>) -> Box<dyn Write> {
	out_path
		.map(|path| {
			Box::new(
				OpenOptions::new()
					.write(true)
					.create(true)
					.truncate(true)
					.open(path)
					.expect("Couldn't open or create output file."),
			) as Box<dyn Write>
		})
		.unwrap_or_else(|| {
			println!("No output path specified, writing to stdout.");
			Box::new(io::stdout()) as Box<dyn Write>
		})
}
