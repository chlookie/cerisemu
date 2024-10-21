use ::std::boxed::Box;
use std::{
	fmt::{self, Display, Formatter},
	ops::{Index, IndexMut, Range},
};

use serde::Serialize;

use super::program::{Address, Program, Row};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub const DEFAULT_SIZE: usize = 256;

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Memory {
	rows: Box<[Row]>,
}

impl Memory {
	pub fn new(size: usize) -> Self {
		// We need to first create a vec and then get a slice from it to make sure we create everything directly on the heap
		Self {
			rows: vec![Row::default(); size].into_boxed_slice(),
		}
	}

	pub fn from_program(size: usize, program: Program) -> Self {
		let mut mem = Self::new(size);
		mem.load_program(program, Address(0x0));
		mem
	}

	pub fn load_program(&mut self, program: Program, address: Address) {
		self.rows.as_mut()[address.0..program.rows.len() + address.0].clone_from_slice(&program.rows);
	}

	pub fn mem_size(&self) -> usize {
		self.rows.len()
	}
}

impl Default for Memory {
	fn default() -> Self {
		Self::new(DEFAULT_SIZE)
	}
}

impl Index<Address> for Memory {
	type Output = Row;

	fn index(&self, index: Address) -> &Self::Output {
		&self.rows.as_ref()[index.0]
	}
}

impl IndexMut<Address> for Memory {
	fn index_mut(&mut self, index: Address) -> &mut Self::Output {
		&mut self.rows.as_mut()[index.0]
	}
}

impl Index<Range<Address>> for Memory {
	type Output = [Row];

	fn index(&self, range: Range<Address>) -> &Self::Output {
		&self.rows.as_ref()[range.start.0..range.end.0]
	}
}

impl IndexMut<Range<Address>> for Memory {
	fn index_mut(&mut self, range: Range<Address>) -> &mut Self::Output {
		&mut self.rows.as_mut()[range.start.0..range.end.0]
	}
}

impl Display for Memory {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		let mut out = Vec::<String>::new();
		let mut is_zero = true;

		let mem_size = format!("{}", Address(self.mem_size() - 1));
		let address_padding = mem_size.chars().count();

		for (address, row) in self.rows.iter().enumerate() {
			let address = Address(address);

			if row == &Row::default() {
				if !is_zero {
					let last = out.len() - 1;
					out[last].replace_range(..address_padding, &format!("{:<address_padding$}", address - 1));

					out.push(format!("{:<address_padding$} ...", ""));
					is_zero = true;
				}
			} else {
				let row = match row {
					Row::Word(w) => w as &dyn Display,
					Row::Instruction(i) => i as &dyn Display,
				};

				if is_zero {
					out.push(format!("{:<address_padding$} | {}", address, row));
				} else {
					out.push(format!("{:<address_padding$} | {}", "", row));
				}
				is_zero = false;
			}
		}

		let last = out.len() - 1;
		out[last].replace_range(
			..address_padding,
			&format!("{:<address_padding$}", Address(self.mem_size() - 1)),
		);

		f.pad(&("\n".to_owned() + &out.join("\n")))
	}
}
