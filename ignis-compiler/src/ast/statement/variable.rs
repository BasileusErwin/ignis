use crate::ast::{expression::Expression, lexer::token::Token, data_type::DataType};

#[derive(Debug, PartialEq, Clone)]
pub struct Variable {
  pub name: Box<Token>,
  pub initializer: Option<Box<Expression>>,
  pub type_annotation: Box<DataType>,
  pub is_mutable: bool,
}

impl Variable {
  pub fn new(
    name: Box<Token>,
    initializer: Option<Box<Expression>>,
    type_annotation: Box<DataType>,
    is_mutable: bool,
  ) -> Self {
    Self {
      name,
      initializer,
      type_annotation,
      is_mutable,
    }
  }
}
