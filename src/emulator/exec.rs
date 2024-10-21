use crate::util::Lattice;

use super::{
	instruction::{Instruction, RegisterOrWord},
	machine::{Interrupt, Machine, State},
	permission::Permission,
	program::{AddrInt, Address, Capability, Register, Row, Word, WordInt},
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

impl Machine {
	/// Executes an entire emulation loop.
	/// The PC register is first initialized with a (RWX, 0, MAX_ADDRESS, 0) capability.
	/// This loop is stopped when the machine reaches a HALTED or FAILED state.
	pub fn exec_machine(&mut self) {
		self.exec_state = State::Running;

		// Create the master capability that the OS will own. Copy it to PC.
		let master_capa = self.sign_capability(Capability {
			perm: Permission::top(),
			base: Address(0x0),
			end: Address(self.memory.mem_size()),
			address: Address(0x0),
		});

		self.write_register(Register::PC, Word::Capability(master_capa));

		loop {
			let new_state = self.exec_single();

			match new_state {
				// Machine is running normally; continue
				State::Running => {
					self.exec_state = new_state;
					self.append_backtrace(format!("New State: {}", new_state));
					continue;
				}

				// Machine failed or halted while trying to recover from an interrupt; stop the machine for good
				State::Halted | State::Failed if matches!(self.exec_state, State::Interrupted(_)) => {
					// Return the machine to state it triggered the interrupt with
					let (new_state, interrupt) = match self.exec_state {
						State::Interrupted(Interrupt::Halt) => (State::Halted, Interrupt::Halt),
						State::Interrupted(Interrupt::Fail) => (State::Failed, Interrupt::Fail),
						_ => unreachable!(),
					};

					self.exec_state = new_state;
					self.new_backtrace(format!("{} Interrupt not recoverable", interrupt));

					break;
				}

				// Machine halted or failed; attempt to recover with an interrupt and continue
				State::Halted | State::Failed => {
					let interrupt = match new_state {
						State::Halted => Interrupt::Halt,
						State::Failed => Interrupt::Fail,
						_ => unreachable!(),
					};

					let Row::Word(destination) = self.get_interrupt_memory(interrupt) else {
						// If recovery is impossible because the destination row isn't a Word,
						// then terminate with the appropriate state
						self.exec_state = new_state;
						self.new_backtrace(format!("{} Interrupt not recoverable", interrupt));
						break;
					};

					self.append_backtrace(format!("New State: {}", new_state));

					// Mark the state as interrupted so we can terminate if the machine fails to recover
					self.exec_state = State::Interrupted(interrupt);

					self.new_backtrace(format!("Interrupt: {}", interrupt));
					self.append_backtrace(format!("New State: {}", self.exec_state));

					// If recovering is possible, attempt to continue execution at the interrupt destination
					self.write_register(Register::PC, self.update_pc_perm(destination));
					continue;
				}

				State::Interrupted(_) => unreachable!("ExecSingle should not return an Interrupted State."),
			}
		}

		self.new_backtrace(format!("State: {}", self.exec_state));
		self.append_backtrace("Shutting down".to_string());
	}

	/// ExecSingle from Cerise.
	/// First performs necessary checks, then executes a single instruction.
	///
	/// Cerise specs:
	///   (Running, 𝜑) →
	///      if 𝜑.reg(pc) = (𝑝, 𝑏, 𝑒, 𝑎)  ∧  𝑏 ≤ 𝑎 < 𝑒  ∧  𝑝 ∈ {rx, rwx}  ∧  𝜑.mem(a) = 𝑧
	///      then [decode(𝑧)](𝜑)
	///      else (Failed, 𝜑)
	///
	/// Since we don't need decoding of instructions in our emulator, we replace decode() with exec_instruction() instead.
	fn exec_single(&mut self) -> State {
		self.new_backtrace(format!("State: {}", self.exec_state));
		self.append_backtrace(format!("PC: {}", self.read_register(Register::PC)));

		let Some(Capability {
			perm,
			base,
			end,
			address,
		}) = self.get_register_capability(Register::PC)
		else {
			self.append_backtrace("Error: Invalid PC, not a capability".to_string());
			return State::Failed;
		};

		if !(base <= address && address < end && perm >= Permission::RX) {
			self.append_backtrace(format!(
				"Error: Invalid PC, address ({}) out of bounds or invalid permission ({})",
				address, perm
			));
			return State::Failed;
		}

		let Row::Instruction(instruction) = self.memory[address].clone() else {
			self.append_backtrace(format!(
				"Error: Invalid instruction where PC is pointing to ({})",
				self.memory[address]
			));
			return State::Failed;
		};

		self.append_backtrace(format!("Instruction: {}", instruction));
		self.exec_instruction(instruction)
	}

	/// Executes a single instruction, applies any side effects of that instruction to the machine itself, and returns the resulting machine state.
	fn exec_instruction(&mut self, instruction: Instruction) -> State {
		match instruction {
			// Effect:
			// 	(Failed, 𝜑)
			Instruction::Fail => State::Failed,

			// Effect:
			// 	(Halted, 𝜑)
			Instruction::Halt => State::Halted,

			// Instruction:
			// 	mov 𝑟 𝜌
			// Conditions:
			// 	𝑤 = getWord(𝜑, 𝜌)
			// Effect:
			// 	updPC(𝜑[reg.𝑟 ↦ 𝑤])
			Instruction::Mov(r, p) => {
				let w = self.get_word(p);
				self.write_register(r, w);
				self.upd_pc()
			}

			// Instruction:
			// 	load 𝑟1 𝑟2
			// Conditions:
			// 	𝜑.reg(𝑟2) = (𝑝, 𝑏, 𝑒, 𝑎)
			// 	𝑝 ∈ {ro, rx, rw, rwx}
			// 	𝑏 ≤ 𝑎 < 𝑒
			// 	𝑤 = 𝜑.mem(𝑎)
			// Effect:
			// 	updPC(𝜑[reg.𝑟1 ↦ 𝑤])
			Instruction::Load(r1, r2) => {
				let Some(Capability {
					perm,
					base,
					end,
					address,
				}) = self.get_register_capability(r2)
				else {
					self.append_backtrace(format!(
						"Error: Invalid register r2 ({}), not a capability",
						self.read_register(r2)
					));
					return State::Failed;
				};

				if !(base <= address && address < end && perm >= Permission::RO) {
					self.append_backtrace(format!("Error: Invalid address ({}) or permission ({})", address, perm));
					return State::Failed;
				};

				let Row::Word(w) = self.memory[address].clone() else {
					self.append_backtrace(format!(
						"Error: Invalid address ({}), not a word ({})",
						address, self.memory[address]
					));
					return State::Failed;
				};

				self.write_register(r1, w);
				self.upd_pc()
			}

			// Instruction:
			// 	store 𝑟 𝜌
			// Conditions:
			// 	𝜑.reg(𝑟) = (𝑝, 𝑏, 𝑒, 𝑎)
			// 	𝑝 ∈ {rw, rwx}
			// 	𝑏 ≤ 𝑎 < 𝑒
			// 	𝑤 = getWord(𝜑, 𝜌)
			// Effect:
			// 	updPC(𝜑[mem.𝑎 ↦ 𝑤])
			Instruction::Store(r, p) => {
				let Some(Capability {
					perm,
					base,
					end,
					address,
				}) = self.get_register_capability(r)
				else {
					self.append_backtrace(format!(
						"Error: Invalid register r ({}), not a capability",
						self.read_register(r)
					));
					return State::Failed;
				};

				if !(base <= address && address < end && perm >= Permission::RW) {
					self.append_backtrace(format!("Error: Invalid address ({}) or permission ({})", address, perm));
					return State::Failed;
				}

				let w = self.get_word(p);

				self.memory[address] = Row::Word(w);
				self.upd_pc()
			}

			// Instruction:
			// 	jmp 𝑟
			// Conditions:
			// 	newPc = updatePcPerm(𝜑.reg(𝑟))
			// Effect:
			// 	(Running, 𝜑[reg.pc ↦ newPc])
			Instruction::Jmp(r) => {
				let value = self.read_register(r);
				let new_pc = self.update_pc_perm(value.clone());

				self.write_register(Register::PC, new_pc);

				self.append_backtrace(format!("Jumping to {}", value));

				State::Running
			}

			// Instruction:
			// 	jnz 𝑟1 𝑟2
			// Conditions:
			// 	newPc = updatePcPerm(𝜑.reg(𝑟1))
			// Effect:
			// 	if 𝜑.reg(𝑟2) ≠ 0,
			// 	then (Running, 𝜑[reg.pc ↦ newPc])
			// 	else updPC(𝜑)
			Instruction::Jnz(r1, r2) => {
				let value = self.read_register(r1);
				let new_pc = self.update_pc_perm(value.clone());

				if self.read_register(r2) != Word::Integer(0) {
					self.write_register(Register::PC, new_pc);

					self.append_backtrace(format!("Jumping to {}", value));

					State::Running
				} else {
					self.append_backtrace("NOT jumping".to_string());
					self.upd_pc()
				}
			}

			// Instruction:
			// 	restrict 𝑟 𝜌
			// Conditions (MODIFIED FROM CERISE):
			// 	𝜑.reg(𝑟) = (𝑝, 𝑏, 𝑒, 𝑎)
			// 	𝜌 ≼ 𝑝
			// 	𝑤 = (𝜌, 𝑏, 𝑒, 𝑎)
			// Effect:
			// 	updPC(𝜑[reg.𝑟 ↦ 𝑤])
			Instruction::Restrict(r, p) => {
				let Some(Capability {
					perm,
					base,
					end,
					address,
				}) = self.get_register_capability(r)
				else {
					self.append_backtrace(format!(
						"Error: Invalid register r ({}), not a capability",
						self.read_register(r)
					));
					return State::Failed;
				};

				#[allow(clippy::neg_cmp_op_on_partial_ord)]
				if !(p <= perm) {
					self.append_backtrace(format!("Error: Invalid permission ({})", perm));
					return State::Failed;
				}

				let w = self.sign_capability(Capability {
					perm: p,
					base,
					end,
					address,
				});

				self.write_register(r, Word::Capability(w));
				self.upd_pc()
			}

			// Instruction:
			// 	subseg 𝑟 𝜌1 𝜌2
			// Conditions:
			// 	𝜑.reg(𝑟) = (𝑝, 𝑏, 𝑒, 𝑎)
			// 	𝑧1 = getWord(𝜑, 𝜌1)
			// 	𝑧2 = getWord(𝜑, 𝜌2)
			// 	𝑧1 ∈ Z
			// 	𝑧2 ∈ Z
			// 	𝑏 ≤ 𝑧1 < AddrMax
			// 	0 ≤ 𝑧2 ≤ 𝑒
			// 	𝑝 ≠ e
			// 	𝑤 = (𝑝, 𝑧1, 𝑧2, 𝑎)
			// Effect:
			// 	updPC(𝜑[reg.𝑟 ↦ 𝑤])
			Instruction::Subseg(r, p1, p2) => {
				let Some(Capability {
					perm,
					base,
					end,
					address,
				}) = self.get_register_capability(r)
				else {
					self.append_backtrace(format!(
						"Error: Invalid register r ({}), not a capability",
						self.read_register(r)
					));
					return State::Failed;
				};

				let z1 = self.get_word(p1);
				let z2 = self.get_word(p2);

				let (Word::Integer(z1), Word::Integer(z2)) = (z1.clone(), z2.clone()) else {
					self.append_backtrace(format!("Error: Invalid p1 ({}) or p2 ({}), not integers", z1, z2));
					return State::Failed;
				};

				if !(base <= z1 as AddrInt
					&& (z1 as AddrInt) < self.memory.mem_size()
					&& 0 <= z2 && (z2 as AddrInt) <= end
					&& perm != Permission::E)
				{
					self.append_backtrace(format!(
						"Invalid addresses z1 ({}) or z2 ({}), or invalid permission ({})",
						z1, z2, perm
					));
					return State::Failed;
				}

				let w = self.sign_capability(Capability {
					perm,
					base: Address(z1 as AddrInt),
					end: Address(z2 as AddrInt),
					address,
				});

				self.write_register(r, Word::Capability(w));
				self.upd_pc()
			}

			// Instruction:
			// 	lea 𝑟 𝜌
			// Conditions:
			// 	𝜑.reg(𝑟) = (𝑝, 𝑏, 𝑒, 𝑎)
			// 	𝑧 = getWord(𝜑, 𝜌)
			// 	𝑝 ≠ e
			// 	𝑤 = (𝑝, 𝑏, 𝑒, 𝑎 + 𝑧)
			// Effect:
			// 	updPC(𝜑[reg.𝑟 ↦ 𝑤])
			Instruction::Lea(r, p) => {
				let Some(Capability {
					perm,
					base,
					end,
					address,
				}) = self.get_register_capability(r)
				else {
					self.append_backtrace(format!(
						"Error: Invalid register r ({}), not a capability",
						self.read_register(r)
					));
					return State::Failed;
				};

				if perm == Permission::E {
					self.append_backtrace(format!("Error: Invalid permission ({})", perm));
					return State::Failed;
				}

				let Word::Integer(z) = self.get_word(p.clone()) else {
					self.append_backtrace(format!("Error: Invalid p ({}), not an integer", p));
					return State::Failed;
				};

				let w = self.sign_capability(Capability {
					perm,
					base,
					end,
					address: Address((address.0 as WordInt + z) as AddrInt), // This mess is needed because z can be negative
				});

				self.write_register(r, Word::Capability(w));
				self.upd_pc()
			}

			// Instruction:
			// 	add 𝑟 𝜌1 𝜌2
			// Conditions:
			// 	𝑧1 = getWord(𝜑, 𝜌1)
			// 	𝑧2 = getWord(𝜑, 𝜌2)
			// 	𝑧1 ∈ Z
			// 	𝑧2 ∈ Z
			// 	𝑧 = 𝑧1 + 𝑧2
			// Effect:
			// 	updPC(𝜑[reg.𝑟 ↦ 𝑧])
			Instruction::Add(r, p1, p2) => {
				let z1 = self.get_word(p1);
				let z2 = self.get_word(p2);

				let (Word::Integer(z1), Word::Integer(z2)) = (z1.clone(), z2.clone()) else {
					self.append_backtrace(format!("Error: Invalid p1 ({}) or p2 ({}), not integers", z1, z2));
					return State::Failed;
				};

				let z = z1 + z2;

				self.write_register(r, Word::Integer(z));
				self.upd_pc()
			}

			// Instruction:
			// 	sub 𝑟 𝜌1 𝜌2
			// Conditions:
			// 	𝑧1 = getWord(𝜑, 𝜌1)
			// 	𝑧2 = getWord(𝜑, 𝜌2)
			// 	𝑧1 ∈ Z
			// 	𝑧2 ∈ Z
			// 	𝑧 = 𝑧1 - 𝑧2
			// Effect:
			// 	updPC(𝜑[reg.𝑟 ↦ 𝑧])
			Instruction::Sub(r, p1, p2) => {
				let z1 = self.get_word(p1);
				let z2 = self.get_word(p2);

				let (Word::Integer(z1), Word::Integer(z2)) = (z1.clone(), z2.clone()) else {
					self.append_backtrace(format!("Error: Invalid p1 ({}) or p2 ({}), not integers", z1, z2));
					return State::Failed;
				};

				let z = z1 - z2;

				self.write_register(r, Word::Integer(z));
				self.upd_pc()
			}

			// Instruction:
			// 	lt 𝑟 𝜌1 𝜌2
			// Conditions:
			// 	𝑧1 = getWord(𝜑, 𝜌1)
			// 	𝑧2 = getWord(𝜑, 𝜌2)
			// 	𝑧1 ∈ Z
			// 	𝑧2 ∈ Z
			// 	if 𝑧1 < 𝑧2 then 𝑧 = 1 else 𝑧 = 0
			// Effect:
			// 	updPC(𝜑[reg.𝑟 ↦ 𝑧])
			Instruction::Lt(r, p1, p2) => {
				let z1 = self.get_word(p1);
				let z2 = self.get_word(p2);

				let (Word::Integer(z1), Word::Integer(z2)) = (z1.clone(), z2.clone()) else {
					self.append_backtrace(format!("Error: Invalid p1 ({}) or p2 ({}), not integers", z1, z2));
					return State::Failed;
				};

				let z = if z1 < z2 { 1 } else { 0 };

				self.write_register(r, Word::Integer(z));
				self.upd_pc()
			}

			// Instruction:
			// 	getp 𝑟1 𝑟2
			// Conditions (MODIFIED FROM CERISE):
			// 	𝜑.reg(𝑟2) = (𝑝, _, _, _)
			// Effect:
			// 	updPC(𝜑[reg.𝑟1 ↦ 𝑝])
			Instruction::Getp(r1, r2) => {
				let Some(Capability { perm, .. }) = self.get_register_capability(r2) else {
					self.append_backtrace(format!(
						"Error: Invalid register r2 ({}), not a capability",
						self.read_register(r2)
					));
					return State::Failed;
				};

				self.write_register(r1, Word::Permission(perm));
				self.upd_pc()
			}

			// Instruction:
			// 	getb 𝑟1 𝑟2
			// Conditions:
			// 	𝜑.reg(𝑟2) = (_, 𝑏, _, _)
			// Effect:
			// 	updPC(𝜑[reg.𝑟1 ↦ 𝑏])
			Instruction::Getb(r1, r2) => {
				let Some(Capability { base, .. }) = self.get_register_capability(r2) else {
					self.append_backtrace(format!(
						"Error: Invalid register r2 ({}), not a capability",
						self.read_register(r2)
					));
					return State::Failed;
				};

				self.write_register(r1, Word::Integer(base.0 as WordInt));
				self.upd_pc()
			}

			// Instruction:
			// 	gete 𝑟1 𝑟2
			// Conditions:
			// 	𝜑.reg(𝑟2) = (_, _, 𝑒, _)
			// Effect:
			// 	updPC(𝜑[reg.𝑟1 ↦ 𝑒])
			Instruction::Gete(r1, r2) => {
				let Some(Capability { end, .. }) = self.get_register_capability(r2) else {
					self.append_backtrace(format!(
						"Error: Invalid register r2 ({}), not a capability",
						self.read_register(r2)
					));
					return State::Failed;
				};

				self.write_register(r1, Word::Integer(end.0 as WordInt));
				self.upd_pc()
			}

			// Instruction:
			// 	geta 𝑟1 𝑟2
			// Conditions:
			// 	𝜑.reg(𝑟2) = (_, _, _, 𝑎)
			// Effect:
			// 	updPC(𝜑[reg.𝑟1 ↦ 𝑎])
			Instruction::Geta(r1, r2) => {
				let Some(Capability { address, .. }) = self.get_register_capability(r2) else {
					self.append_backtrace(format!(
						"Error: Invalid register r2 ({}), not a capability",
						self.read_register(r2)
					));
					return State::Failed;
				};

				self.write_register(r1, Word::Integer(address.0 as WordInt));
				self.upd_pc()
			}

			// Instruction:
			// 	isptr 𝑟1 𝑟2
			// Conditions:
			// 	if 𝜑.reg(𝑟2) = (_, _, _, _) then 𝑧 = 1 else 𝑧 = 0
			// Effect:
			// 	updPC(𝜑[reg.𝑟1 ↦ 𝑧])
			Instruction::Isptr(r1, r2) => {
				let z = if let Word::Capability(_) = self.read_register(r2) {
					1
				} else {
					0
				};

				self.write_register(r1, Word::Integer(z));
				self.upd_pc()
			}
		}
	}

	/// getWord(𝜑, 𝜌) from Cerise.
	///
	/// Cerise specs:
	///   getWord(𝜑, 𝜌) =
	///     if 𝜌 ∈ Z       then 𝜌
	///     if 𝜌 ∈ RegName then 𝜑.reg(𝜌)
	fn get_word(&mut self, p: RegisterOrWord) -> Word {
		match p {
			RegisterOrWord::Register(r) => self.read_register(r),
			RegisterOrWord::Word(w) => w,
		}
	}

	/// updPC(𝜑) from Cerise.
	///
	/// Cerise specs:
	///   updPC(𝜑) =
	///     if 𝜑.reg(pc) = (𝑝, 𝑏, 𝑒, 𝑎)
	///     then (Running, 𝜑[reg.pc ↦ (𝑝, 𝑏, 𝑒, 𝑎 + 1)])
	///     else (Failed, 𝜑)
	fn upd_pc(&mut self) -> State {
		let Some(Capability {
			perm,
			base,
			end,
			address,
		}) = self.get_register_capability(Register::PC)
		else {
			self.append_backtrace("Error: Couldn't update PC, invalid PC".to_string());
			return State::Failed;
		};

		let new_capa = self.sign_capability(Capability {
			perm,
			base,
			end,
			address: address + 1,
		});

		self.write_register(Register::PC, Word::Capability(new_capa));
		State::Running
	}

	/// updatePcPerm(𝑤) from Cerise.
	///
	/// Cerise specs:
	///   updatePcPerm(𝑤) =
	///     if 𝑤 = (e, 𝑏, 𝑒, 𝑎)
	///     then (rx, 𝑏, 𝑒, 𝑎)
	///     else 𝑤
	fn update_pc_perm(&self, word: Word) -> Word {
		let Word::Capability(signed_capability) = word.clone() else {
			return word;
		};

		let capability = self.verify_capability(signed_capability);

		if let Some(Capability {
			perm: Permission::E,
			base,
			end,
			address,
		}) = capability
		{
			Word::Capability(self.sign_capability(Capability {
				perm: Permission::RX,
				base,
				end,
				address,
			}))
		} else {
			word
		}
	}
}
