use crate::ast::{expression::Expression, lexer::token::Token};

use super::Visitor;

#[derive(Debug)]
pub struct Variable {
	name: Box<Token>,
	initializer: Box<Expression>
}

impl Variable {
	pub fn new(name: Box<Token>, initializer: Box<Expression>) -> Self{
		Self { name, initializer }
	}
	
	pub fn accept<R>(&self, visitor: &dyn Visitor<R>) -> R {
			visitor.visit_variable_statement(self)
  }
}
