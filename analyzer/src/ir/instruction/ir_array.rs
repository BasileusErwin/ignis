use enums::data_type::DataType;
use lexer::token::Token;

use super::IRInstruction;

#[derive(Debug, Clone)]
pub struct IRArray {
  pub elements: Vec<IRInstruction>,
  pub data_type: DataType,
  pub lenght: usize,
}

impl IRArray {
  pub fn new(elements: Vec<IRInstruction>, data_type: DataType) -> Self {
    Self {
      lenght: elements.len(),
      elements,
      data_type,
    }
  }
}
