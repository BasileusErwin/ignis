use lexer::token::Token;
use enums::data_type::DataType;

use super::Statement;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParamater {
  pub name: Token,
  pub data_type: DataType,
  // TODO:
  pub is_mutable: bool,
  pub is_reference: bool,
}

impl FunctionParamater {
  pub fn new(name: Token, data_type: DataType) -> Self {
    Self {
      name,
      data_type,
      is_mutable: false,
      is_reference: false,
    }
  }

  pub fn to_string(&self) -> String {
    format!("{}: {}", self.name.span.literal, self.data_type.to_string())
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
