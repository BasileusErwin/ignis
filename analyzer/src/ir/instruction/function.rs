use ast::statement::import::ImportSymbol;
use enums::data_type::DataType;

use super::{variable::IRVariable, block::IRBlock};

#[derive(Debug, Clone)]
pub struct IRFunctionMetadata {
  pub is_recursive: bool,
  pub is_exported: bool,
}

impl IRFunctionMetadata {
  pub fn new(is_recursive: bool, is_exported: bool) -> Self {
    Self {
      is_recursive,
      is_exported,
    }
  }
}

#[derive(Debug, Clone)]
pub struct IRFunction {
  pub name: String,
  pub parameters: Vec<IRVariable>,
  pub return_type: DataType,
  pub body: Option<Box<IRBlock>>,
  pub metadata: IRFunctionMetadata,
}

impl IRFunction {
  pub fn new(
    name: String,
    parameters: Vec<IRVariable>,
    return_type: DataType,
    body: Option<Box<IRBlock>>,
    metadata: IRFunctionMetadata,
  ) -> Self {
    Self {
      name,
      parameters,
      return_type,
      body,
      metadata,
    }
  }
}
