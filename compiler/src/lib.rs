use std::collections::HashMap;

use analyzer::{
  ir::instruction::{IRInstruction, literal::IRLiteral, function},
  analyzer_value::AnalyzerValue,
};

pub struct Compiler {
  pub custom_ir: HashMap<String, Vec<IRInstruction>>,
  pub in_function: bool,
}

impl Compiler {
  pub fn new(custom_ir: HashMap<String, Vec<IRInstruction>>) -> Self {
    Self {
      custom_ir,
      in_function: false,
    }
  }
}
