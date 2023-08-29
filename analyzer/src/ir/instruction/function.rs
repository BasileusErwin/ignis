use enums::data_type::DataType;

use super::{variable::IRVariable, block::IRBlock};

#[derive(Debug, Clone)]
pub struct IRFunction {
  pub name: String,
  pub parameters: Vec<IRVariable>,
  pub return_type: DataType,
  pub body: Option<Box<IRBlock>>,
}

impl IRFunction {
  pub fn new(
    name: String,
    parameters: Vec<IRVariable>,
    return_type: DataType,
    body: Option<Box<IRBlock>>,
  ) -> Self {
    Self {
      name,
      parameters,
      return_type,
      body,
    }
  }
}
