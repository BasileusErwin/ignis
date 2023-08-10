use crate::ast::{lexer::token::Token, data_type::{DataType, self}};
use super::Expression;

#[derive(Debug, PartialEq)]
pub struct Binary {
  pub left: Box<Expression>,
  pub operator: Token,
  pub right: Box<Expression>,
  pub data_type: DataType,
}

impl Binary {
  pub fn new(left: Box<Expression>, operator: Token, right: Box<Expression>, data_type: DataType) -> Self {
    Self {
      left,
      operator,
      right,
      data_type,
    }
  }
}
