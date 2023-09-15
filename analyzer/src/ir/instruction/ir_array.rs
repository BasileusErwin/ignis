use enums::data_type::DataType;
use lexer::token::Token;

use super::IRInstruction;

#[derive(Debug, Clone)]
pub struct IRArray {
  pub elements: Vec<IRInstruction>,
  pub token: Token,
  pub data_type: DataType,
}

impl IRArray {
  pub fn new(elements: Vec<IRInstruction>, token: Token, data_type: DataType) -> Self {
    Self {
      elements,
      token,
      data_type,
    }
  }
}
