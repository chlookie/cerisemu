use std::collections::HashMap;

use cerisemu::compiler::{
	expressions::{evaluate, parse_expression},
	tokens::Token,
};
use logos::Logos;

macro_rules! impl_expression_test {
	($name:ident, $input:expr, $output:expr) => {
		#[test]
		fn $name() {
			let input = $input.to_owned() + "]";
			let expected = $output;

			let result = evaluate(
				parse_expression(&mut Token::lexer(input.as_str())).unwrap(),
				&HashMap::new(),
			)
			.unwrap();
			assert_eq!(result, expected);
		}
	};
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

impl_expression_test!(basic, "0", 0);

impl_expression_test!(negative, "-1", -1);

impl_expression_test!(positive, "+1", 1);

impl_expression_test!(addition, "2+3", 5);

impl_expression_test!(substraction, "515-6468", -5953);

impl_expression_test!(multiplication, "510*62", 31620);

impl_expression_test!(division, "42/42", 1);

impl_expression_test!(mixed_operators, "-2--4+5*6/6-5*8/1/1/1/1+1--0", -32);

impl_expression_test!(nested, "((((1+(4*(5)))/7)))-1", 2);
