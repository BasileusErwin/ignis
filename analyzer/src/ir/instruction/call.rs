use super::IRInstruction;

#[derive(Debug, Clone)]
pub struct IRCall {
  pub callee: Box<IRInstruction>,
  pub arguments: Vec<IRInstruction>,
}

impl IRCall {
  pub fn new(callee: Box<IRInstruction>, arguments: Vec<IRInstruction>) -> Self {
    Self { callee, arguments }
  }
}
