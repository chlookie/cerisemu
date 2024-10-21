mod compiler {
	mod expression;
	mod expression_eval;
}

mod emulator {
	mod instructions;
	mod malloc;
	mod memcpy;
	mod permission;
}

#[macro_export]
macro_rules! assert_register_capability {
	($machine:expr, $reg:expr, $pattern:pat) => {
		let capa = $machine.get_register_capability($reg).unwrap();
		assert!(matches!((capa.perm, capa.base.0, capa.end.0, capa.address.0), $pattern))
	};
}
