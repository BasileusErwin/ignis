use crate::ast::{lexer::token::Token, data_type::DataType};

use super::Statement;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParamater {
  pub name: Token,
  pub data_type: DataType,
}

impl FunctionParamater {
  pub fn new(name: Token, data_type: DataType) -> Self {
    Self { name, data_type }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionStatement {
  pub name: Token,
  pub parameters: Vec<FunctionParamater>,
  pub body: Vec<Statement>,
  pub return_type: Option<DataType>,
}

impl FunctionStatement {
  pub fn new(
    name: Token,
    parameters: Vec<FunctionParamater>,
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
