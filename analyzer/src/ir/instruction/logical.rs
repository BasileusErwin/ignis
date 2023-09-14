use crate::ir::instruction_type::IRInstructionType;

use super::IRInstruction;

#[derive(Debug, Clone)]
pub struct IRLogical {
  pub instruction_type: IRInstructionType,
  pub left: Box<IRInstruction>,
  pub right: Box<IRInstruction>,
}

impl IRLogical {
  pub fn new(
    instruction_type: IRInstructionType,
    left: Box<IRInstruction>,
    right: Box<IRInstruction>,
  ) -> Self {
    Self {
      instruction_type,
      left,
      right,
    }
  }
}
