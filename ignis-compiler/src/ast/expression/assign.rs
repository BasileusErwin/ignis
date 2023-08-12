use crate::ast::{lexer::token::Token, data_type::DataType};

use super::Expression;

#[derive(Debug, PartialEq)]
pub struct Assign {
	pub name: Token,
	pub value: Box<Expression>,
	pub data_type: DataType,
}

impl Assign {
	pub fn new(name: Token, value: Box<Expression>, data_type: DataType) -> Self {
		Self {
			name,
			value,
			data_type,
		}
	}
}
