use crate::ast::lexer::token::Token;
use super::{Expression, Visitor};

#[derive(Debug)]
pub struct Binary {
  pub left: Box<Expression>,
  pub operator: Token,
  pub right: Box<Expression>,
}

impl Binary {
  pub fn new(left: Box<Expression>, operator: Token, right: Box<Expression>) -> Self {
    Self {
      left,
      operator,
      right,
    }
  }

  pub fn accept<R>(&self, visitor: &dyn Visitor<R>) -> R {
    visitor.visit_binary_expression(self)
  }
}
