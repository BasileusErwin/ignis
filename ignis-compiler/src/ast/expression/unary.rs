use super::{Expression, Visitor};
use crate::ast::lexer::token::Token;

#[derive(Debug)]
pub struct Unary {
	pub operator: Token,
	pub right: Box<Expression>,
}

impl Unary {
  pub fn new(operator: Token, right: Box<Expression>) -> Self {
    Self {
      operator,
      right,
    }
  }

	pub fn accept<R>(&self, visitor: &dyn Visitor<R>) -> R {
			visitor.visit_unary_expression(self)
	}
}
