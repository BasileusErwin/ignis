use super::Expression;
use crate::ast::{lexer::token::Token, data_type::DataType};

#[derive(Debug, PartialEq)]
pub struct Unary {
  pub operator: Token,
  pub right: Box<Expression>,
  pub data_type: DataType,
}

impl Unary {
  pub fn new(operator: Token, right: Box<Expression>, data_type: DataType) -> Self {
    Self {
      operator,
      right,
      data_type,
    }
  }
}
