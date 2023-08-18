use crate::ast::expression::Expression;

#[derive(Debug, PartialEq, Clone)]
pub struct Return {
	pub value: Option<Box<Expression>>,
}

impl Return {
	pub fn new(value: Option<Box<Expression>>) -> Self {
		Self { value }
	}
}
