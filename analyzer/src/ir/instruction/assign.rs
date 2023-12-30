use super::{IRInstruction, IRInstructionTrait};
use serde_json::{json, Value};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct IRAssign {
  pub name: String,
  pub value: Box<IRInstruction>,
}

impl IRAssign {
  pub fn new(name: String, value: Box<IRInstruction>) -> Self {
    Self { name, value }
  }
}

impl IRInstructionTrait for IRAssign {
  fn to_json(&self) -> Value {
    json!({
      "type": "IRAssign",
      "name": self.name,
      "value": self.value.to_json(),
    })
  }
}
