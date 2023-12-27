use std::fmt::{Formatter, Display, self};

use crate::token_type::TokenType;

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
  Int(i64),
  Float(f64),
  Char(char),
  String(String),
  Boolean(bool),
  Null,
}

impl Display for LiteralValue {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      LiteralValue::Boolean(x) => write!(f, "{}", x),
      LiteralValue::Null => write!(f, "null"),
      LiteralValue::Float(x) => write!(f, "{}", x),
      LiteralValue::Int(x) => write!(f, "{}", x),
      LiteralValue::String(x) => write!(f, "{}", x),
      LiteralValue::Char(x) => write!(f, "{}", x),
    }
  }
}

impl LiteralValue {
  pub fn from_token_type(kind: TokenType, value: String) -> Self {
    match kind {
      TokenType::Int => Self::Int(value.parse().unwrap()),
      TokenType::Float => Self::Float(value.parse().unwrap()),
      TokenType::String => Self::String(value),
      TokenType::False | TokenType::True => Self::Boolean(value.parse().unwrap()),
      _ => Self::Null,
    }
  }
}
