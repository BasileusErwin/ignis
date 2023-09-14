use enums::data_type::{DataType, self};

use crate::ir::instruction_type::IRInstructionType;

use super::IRInstruction;

#[derive(Debug, Clone)]
pub struct IRUnary {
  pub instruction_type: IRInstructionType,
  pub right: Box<IRInstruction>,
  pub data_type: DataType,
}

impl IRUnary {
  pub fn new(
    instruction_type: IRInstructionType,
    right: Box<IRInstruction>,
    data_type: DataType,
  ) -> Self {
    Self {
      instruction_type,
      right,
      data_type,
    }
  }
}
