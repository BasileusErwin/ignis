use enums::data_type::DataType;

use super::IRInstruction;

#[derive(Debug, Clone)]
pub struct IRVariable {
  pub name: String,
  pub data_type: DataType,
  pub value: Option<Box<IRInstruction>>,

  // TODO: Implement inmutable parameters and reference
  pub is_mutable: bool,
  pub is_reference: bool,
}

impl IRVariable {
  pub fn new(
    name: String,
    data_type: DataType,
    is_mutable: bool,
    is_reference: bool,
    value: Option<Box<IRInstruction>>,
  ) -> Self {
    Self {
      name,
      data_type,
      is_mutable,
      is_reference,
      value,
    }
  }
}
