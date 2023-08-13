use crate::ast::{
  lexer::token::Token,
  data_type::{DataType, self},
};

#[derive(Debug, PartialEq)]
pub struct VariableExpression {
  pub name: Token,
  pub data_type: DataType,
}

impl VariableExpression {
  pub fn new(name: Token, data_type: DataType) -> Self {
    Self { name, data_type }
  }
}
