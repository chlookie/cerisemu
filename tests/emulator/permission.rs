use cerisemu::{
	emulator::permission::Permission::{self, *},
	util::Lattice,
};

#[test]
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn ordering() {
	assert!(E > O);
	assert!(RW > RO);
	assert!(RW >= RO);
	assert!(RX > RO);
	assert!(RX >= RO);
	assert!(E <= RX);
	assert!(RWX > O);
	assert!(RO >= RO);
}

#[test]
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn non_ordering() {
	assert!(!(RO > E));
	assert!(!(RO < E));
	assert!(!(RO >= E));
	assert!(!(RO <= E));
	assert!(RO != E);
	assert!(E != RO);

	assert!(!(RW > RX));
	assert!(!(RW < RX));
	assert!(!(RW >= RX));
	assert!(!(RW <= RX));
	assert!(RW != RX);
	assert!(RX != RW);

	assert!(!(RW > E));
	assert!(!(RW < E));
	assert!(!(RW >= E));
	assert!(!(RW <= E));
	assert!(RW != E);
	assert!(E != RW);
}

#[test]
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn top() {
	assert!(RWX == Permission::top());

	assert!(O < RWX);
	assert!(RO < RWX);
	assert!(E < RWX);
	assert!(RW < RWX);
	assert!(RX < RWX);

	assert!(O <= RWX);
	assert!(RO <= RWX);
	assert!(E <= RWX);
	assert!(RW <= RWX);
	assert!(RX <= RWX);
	assert!(RWX <= RWX);

	assert!(RWX > O);
	assert!(RWX > RO);
	assert!(RWX > E);
	assert!(RWX > RW);
	assert!(RWX > RX);

	assert!(RWX >= O);
	assert!(RWX >= RO);
	assert!(RWX >= E);
	assert!(RWX >= RW);
	assert!(RWX >= RX);
	assert!(RWX >= RWX);
}

#[test]
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn bot() {
	assert!(O == Permission::bot());

	assert!(O < RO);
	assert!(O < E);
	assert!(O < RW);
	assert!(O < RX);
	assert!(O < RWX);

	assert!(O <= O);
	assert!(O <= RO);
	assert!(O <= E);
	assert!(O <= RW);
	assert!(O <= RX);
	assert!(O <= RWX);

	assert!(RO > O);
	assert!(E > O);
	assert!(RW > O);
	assert!(RX > O);
	assert!(RWX > O);

	assert!(O >= O);
	assert!(RO >= O);
	assert!(E >= O);
	assert!(RW >= O);
	assert!(RX >= O);
	assert!(RWX >= O);
}

#[test]
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn join() {
	assert_eq!(O.join(O), O);
	assert_eq!(RWX.join(RWX), RWX);

	assert_eq!(O.join(RWX), RWX);
	assert_eq!(RWX.join(O), RWX);

	assert_eq!(E.join(O), E);
	assert_eq!(E.join(RO), RX);
	assert_eq!(E.join(RW), RWX);
	assert_eq!(O.join(E), E);
	assert_eq!(RO.join(E), RX);
	assert_eq!(RW.join(E), RWX);
}

#[test]
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn meet() {
	assert_eq!(O.meet(O), O);
	assert_eq!(RWX.meet(RWX), RWX);

	assert_eq!(O.meet(RWX), O);
	assert_eq!(RWX.meet(O), O);

	assert_eq!(E.meet(O), O);
	assert_eq!(E.meet(RO), O);
	assert_eq!(E.meet(RW), O);
	assert_eq!(O.meet(E), O);
	assert_eq!(RO.meet(E), O);
	assert_eq!(RW.meet(E), O);
}
