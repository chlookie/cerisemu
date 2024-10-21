use std::io::{self, Read, Write};

use emulator::{
	machine_config::{MachineConfig, ProgramConfig},
	program::Program,
};
use ron::ser::PrettyConfig;
use serde::Serialize;

pub mod compiler;
pub mod emulator;
pub mod util;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

fn serialize_pretty<T>(value: &T) -> String
where
	T: ?Sized + Serialize,
{
	ron::ser::to_string_pretty(
		value,
		PrettyConfig::default().struct_names(true).indentor("\t".to_owned()),
	)
	.expect("Couldn't serialize output to RON. This should not happen.")
}

pub fn compile(input: impl Read, mut output: impl Write) {
	// To pass file to stdin (not required, just in case):
	// windows cmd: cargo run -- < test.asm
	// nushell: open test.asm | cargo run

	// Read input file and compile program
	let program = ProgramConfig::from_reader_as_source(input).compiled();

	println!("Finished compiling. Program is {} rows long.", program.rows.len());

	// Serialize the program to RON
	let out = serialize_pretty(&program);

	// Write program to output
	output
		.write_all(out.as_bytes())
		.expect("Could not write to output writer.");
}

pub fn emulate(input: impl Read, mut output: impl Write, compile: bool, dump: bool, backtrace: bool) {
	// Create machine config to emulate depending on input
	let machine_config = if compile {
		// If it needs compiling, compile the program from source
		MachineConfig::from_program_config(ProgramConfig::from_reader_as_source(input))
	} else {
		// Otherwise parse it from the given file as RON
		// We try to either parse the RON as a compiled program, a program config, or as a machine config
		let source = io::read_to_string(input).expect("Couldn't read input file.");

		if let Ok(program) = ron::de::from_str::<Program>(&source) {
			// Source is a compiled program, initialize machine
			MachineConfig::from_program(program)
		} else if let Ok(program_config) = ron::de::from_str::<ProgramConfig>(&source) {
			// Source is a program config, compile it and initialize machine
			MachineConfig::from_program_config(program_config)
		} else if let Ok(machine_config) = ron::de::from_str::<MachineConfig>(&source) {
			// Source is a machine config, initialize machine from it
			machine_config
		} else {
			panic!("Could not de-serialize input as a compiled program, a program config, or as a machine config; is it a correct RON file?")
		}
	};

	// Run the emulator
	let post_machine = emulator::emulate(machine_config);

	if backtrace {
		post_machine.print_backtrace();
	}

	println!("\n\n{}\n\n", post_machine);
	post_machine.print_status();

	if dump {
		println!("Dumping.");

		// Ouput the post-emulation machine to output for debugging
		let out = serialize_pretty(&post_machine);

		output
			.write_all(out.as_bytes())
			.expect("Could not write to output writer.");
	}
}
