use crate::ast::{lexer::token::Token, data_type::DataType};

use super::Expression;

#[derive(Debug, PartialEq)]
pub struct Logical {
  pub left: Box<Expression>,
  pub operator: Token,
  pub right: Box<Expression>,
  pub data_type: DataType,
}

impl Logical {
  pub fn new(
    left: Box<Expression>,
    operator: Token,
    right: Box<Expression>,
  ) -> Self {
    Self {
      left,
      operator,
      right,
      data_type: DataType::Boolean,
    }
  }
}
