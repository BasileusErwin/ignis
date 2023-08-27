use super::IRInstruction;

#[derive(Debug, Clone)]
pub struct IRReturn {
  pub value: Box<IRInstruction>,
}

impl IRReturn {
  pub fn new(value: Box<IRInstruction>) -> Self {
    Self { value }
  }
}
