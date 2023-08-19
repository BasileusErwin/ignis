use crate::ast::{expression::Expression, lexer::token::Token};

#[derive(Debug, PartialEq, Clone)]
pub struct Return {
  pub value: Option<Box<Expression>>,
  pub keyword: Box<Token>,
}

impl Return {
  pub fn new(value: Option<Box<Expression>>, keyword: Box<Token>) -> Self {
    Self { value, keyword }
  }
}
