use crate::ast::{expression::Expression, lexer::token::Token, data_type::DataType};

#[derive(Debug)]
pub struct Variable {
  name: Box<Token>,
  initializer: Box<Expression>,
  type_annotation: Box<DataType>,
}

impl Variable {
  pub fn new(
    name: Box<Token>,
    initializer: Box<Expression>,
    type_annotation: Box<DataType>,
  ) -> Self {
    Self {
      name,
      initializer,
      type_annotation,
    }
  }
}
