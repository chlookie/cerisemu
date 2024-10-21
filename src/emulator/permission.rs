use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::util::Lattice;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Permission {
	/// BOT; No permissions
	#[default]
	O,

	/// Execute
	E,

	/// Read Only
	RO,

	/// Read, Execute
	RX,

	/// Read, Write
	RW,

	/// TOP; Read, Write, Execute
	RWX,
}

fn flows(a: Permission, b: Permission) -> bool {
	match (a, b) {
		_ if a == b => true,
		(Permission::O, _) => true,
		(_, Permission::RWX) => true,
		(Permission::E, _) => flows(Permission::RX, b),
		(Permission::RO, _) => flows(Permission::RW, b) || flows(Permission::RX, b),
		(Permission::RW, _) => flows(Permission::RWX, b),
		(Permission::RX, _) => flows(Permission::RWX, b),
		_ => false,
	}
}

impl PartialOrd for Permission {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		if self == other {
			Some(Ordering::Equal)
		} else if flows(*self, *other) {
			Some(Ordering::Less)
		} else if flows(*other, *self) {
			Some(Ordering::Greater)
		} else {
			None
		}
	}
}

impl Lattice for Permission {
	fn top() -> Self {
		Permission::RWX
	}

	fn bot() -> Self {
		Permission::O
	}

	fn join(&self, other: Self) -> Self {
		match (self, other) {
			(_, _) if self <= &other => other,
			(_, _) if self >= &other => *self,
			(Permission::E, _) => Permission::RX.join(other),
			(_, Permission::E) => self.join(Permission::RX),
			(Permission::RW, _) => Permission::RWX.join(other),
			(_, Permission::RW) => self.join(Permission::RWX),
			_ => Self::top(),
		}
	}

	fn meet(&self, other: Self) -> Self {
		match (self, other) {
			(_, _) if self >= &other => other,
			(_, _) if self <= &other => *self,
			(Permission::E, _) => Permission::O.meet(other),
			(_, Permission::E) => self.meet(Permission::O),
			(Permission::RW, _) => Permission::RO.meet(other),
			(_, Permission::RW) => self.meet(Permission::RO),
			_ => Self::bot(),
		}
	}
}
