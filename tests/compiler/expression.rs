use cerisemu::compiler::{ast::AstExpression::*, expressions::parse_expression, tokens::Token};
use logos::Logos;

macro_rules! impl_expression_test {
	($name:ident, $input:expr, $output:expr) => {
		#[test]
		fn $name() {
			let input = $input.to_owned() + "]";
			let expected = $output;

			let result = parse_expression(&mut Token::lexer(input.as_str())).unwrap();
			assert_eq!(result, expected);
		}
	};
}

macro_rules! impl_expression_error {
	($name:ident, $input:expr) => {
		#[test]
		fn $name() {
			let input = $input.to_owned() + "]";

			let result = parse_expression(&mut Token::lexer(input.as_str()));
			assert!(result.is_err());
		}
	};
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

impl_expression_test!(basic, "0", Integer(0));

impl_expression_test!(negative, "-1", OpNeg(Box::new(Integer(1))));

impl_expression_test!(positive, "+1", Integer(1));

impl_expression_test!(addition, "2+3", OpAdd(Box::new(Integer(2)), Box::new(Integer(3))));

impl_expression_test!(
	substraction,
	"515-6468",
	OpSub(Box::new(Integer(515)), Box::new(Integer(6468)))
);

impl_expression_test!(
	multiplication,
	"510*62",
	OpMul(Box::new(Integer(510)), Box::new(Integer(62)))
);

impl_expression_test!(division, "42/42", OpDiv(Box::new(Integer(42)), Box::new(Integer(42))));

impl_expression_test!(
	mixed_operators,
	"-2--4+5*6/5-5*8/6/7/8/9+1--0",
	OpSub(
		Box::new(OpAdd(
			Box::new(OpSub(
				Box::new(OpAdd(
					Box::new(OpSub(
						Box::new(OpNeg(Box::new(Integer(2)))),
						Box::new(OpNeg(Box::new(Integer(4)))),
					)),
					Box::new(OpDiv(
						Box::new(OpMul(Box::new(Integer(5)), Box::new(Integer(6)))),
						Box::new(Integer(5)),
					)),
				)),
				Box::new(OpDiv(
					Box::new(OpDiv(
						Box::new(OpDiv(
							Box::new(OpDiv(
								Box::new(OpMul(Box::new(Integer(5)), Box::new(Integer(8)))),
								Box::new(Integer(6)),
							)),
							Box::new(Integer(7)),
						)),
						Box::new(Integer(8)),
					)),
					Box::new(Integer(9)),
				)),
			)),
			Box::new(Integer(1)),
		)),
		Box::new(OpNeg(Box::new(Integer(0)))),
	)
);

impl_expression_test!(
	nested,
	"((((1+(4*(5)))/7)))-1",
	OpSub(
		Box::new(OpDiv(
			Box::new(OpAdd(
				Box::new(Integer(1)),
				Box::new(OpMul(Box::new(Integer(4)), Box::new(Integer(5)))),
			)),
			Box::new(Integer(7)),
		)),
		Box::new(Integer(1)),
	)
);

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

impl_expression_error!(bad_parens, "(1");
impl_expression_error!(bad_op_parens, "1+)");
impl_expression_error!(bad_binary_op_post1, "1+");
impl_expression_error!(bad_binary_op_post2, "1-");
impl_expression_error!(bad_binary_op_post3, "1*");
impl_expression_error!(bad_binary_op_post4, "1/");
impl_expression_error!(bad_binary_op_pre1, "*5");
impl_expression_error!(bad_binary_op_pre2, "/5");
impl_expression_error!(bad_character, "=");
