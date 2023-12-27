use lexer::token::Token;
use enums::data_type::DataType;

use super::Expression;

#[derive(Debug, PartialEq, Clone)]
pub struct Call {
  pub callee: Box<Expression>,
  pub paren: Token,
  pub arguments: Vec<Expression>,
  pub return_type: DataType,
  pub is_constructor: bool,
}

impl Call {
  pub fn new(
    callee: Box<Expression>,
    paren: Token,
    arguments: Vec<Expression>,
    return_type: DataType,
    is_constructor: bool,
  ) -> Self {
    Self {
      callee,
      paren,
      arguments,
      return_type,
      is_constructor,
    }
  }
}
