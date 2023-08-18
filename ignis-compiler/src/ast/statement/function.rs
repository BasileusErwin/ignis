use crate::ast::{lexer::token::Token, data_type::DataType};

use super::Statement;


#[derive(Debug, Clone, PartialEq)]
pub struct FunctionStatement {
	pub name: Token,
	pub parameters: Vec<Token>,
	pub body: Vec<Statement>,
	pub return_type: Option<DataType>,
}

impl FunctionStatement {
	pub fn new(
		name: Token,
		parameters: Vec<Token>,
		body: Vec<Statement>,
		return_type: Option<DataType>,
	) -> Self {
		Self {
			name,
			parameters,
			body,
			return_type,
		}
	}
}
