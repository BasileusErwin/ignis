use crate::ir::instruction_type::IRInstructionType;

use super::{IRInstruction, IRInstructionTrait};

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

impl IRInstructionTrait for IRLogical {
  fn to_json(&self) -> serde_json::Value {
    serde_json::json!({
      "type": self.instruction_type.to_string(),
      "left": self.left.to_json(),
      "right": self.right.to_json(),
    })
  }
}
