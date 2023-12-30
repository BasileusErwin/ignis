use super::{IRInstruction, variable::IRVariable, IRInstructionTrait};

#[derive(Debug, Clone)]
pub struct IRBlock {
  pub instructions: Vec<IRInstruction>,
  pub scopes_variables: Vec<IRVariable>,
}

impl IRBlock {
  pub fn new(instructions: Vec<IRInstruction>, scopes_variables: Vec<IRVariable>) -> Self {
    Self {
      instructions,
      scopes_variables,
    }
  }
}

impl IRInstructionTrait for IRBlock {
  fn to_json(&self) -> serde_json::Value {
    serde_json::json!({
      "type": "IRBlock",
      "instructions": self.instructions.iter().map(|i| i.to_json()).collect::<Vec<serde_json::Value>>(),
      "scopes_variables": self.scopes_variables.iter().map(|v| v.to_json()).collect::<Vec<serde_json::Value>>(),
    })
  }
}
