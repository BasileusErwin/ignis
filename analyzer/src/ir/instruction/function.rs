use enums::data_type::DataType;

use super::{variable::IRVariable, block::IRBlock};

#[derive(Debug, Clone)]
pub struct IRFunctionMetadata {
  pub is_recursive: bool,
  pub is_exported: bool,
  pub is_imported: bool,
  pub is_extern: bool,
  pub is_static: bool,
  pub is_public: bool,
}

impl IRFunctionMetadata {
  pub fn new(
    is_recursive: bool,
    is_exported: bool,
    is_imported: bool,
    is_extern: bool,
    is_static: bool,
    is_public: bool,
  ) -> Self {
    Self {
      is_recursive,
      is_exported,
      is_imported,
      is_extern,
      is_public,
      is_static,
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
