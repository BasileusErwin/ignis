use self::{binary::Binary, grouping::Grouping, literal::Literal, unary::Unary};

use super::lexer::{token_type::TokenType, token::Token};

pub mod binary;
pub mod grouping;
pub mod literal;
pub mod unary;

#[derive(Debug)]
pub enum Expression {
  Binary(Binary),
  Grouping(Grouping),
  Literal(Literal),
  Unary(Unary),
}

impl Expression {
  pub fn to_string(&self) -> String {
    match self {
      Expression::Binary(Binary {
        left,
        operator,
        right,
      }) => format!(
        "({} {} {})",
        operator.span.literal,
        left.to_string(),
        right.to_string()
      ),
      Expression::Grouping(Grouping { expression }) => {
        format!("(group {})", (*expression).to_string())
      }
      Expression::Literal(Literal { value }) => format!("{}", value.to_string()),
      Expression::Unary(Unary { operator, right }) => {
        let operator_str = operator.span.literal.clone();
        let right_str = (*right).to_string();
        format!("({} {})", operator_str, right_str)
      }
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
  Number(f64),
  Int(i64),
  Double(f64),
  Char(char),
  String(String),
  Boolean(bool),
  None,
}

impl LiteralValue {
  pub fn to_string(&self) -> String {
    match self {
        LiteralValue::Boolean(x) => x.to_string(),
        LiteralValue::None => "null".to_string(),
        LiteralValue::Number(x) => x.to_string(),
        LiteralValue::Double(x) => x.to_string(),
        LiteralValue::Int(x) => x.to_string(),
        LiteralValue::String(x) => x.clone(),
        LiteralValue::Char(x) => x.to_string()
    }
  }

  pub fn from_token(token: Token) -> Self {
    match token.kind {
      TokenType::Number => Self::Number(token.span.literal.parse().unwrap()),
      TokenType::Null => Self::None,
      TokenType::Int => Self::Int(token.span.literal.parse().unwrap()),
      TokenType::String => Self::String(token.span.literal),
      TokenType::False | TokenType::True => Self::Boolean(token.span.literal.parse().unwrap()),
      _ => Self::None,
    }
  }
}

pub trait Visitor<R> {
  fn visit_binary_expression(&self, expression: &Binary) -> R;
  fn visit_grouping_expression(&self, expression: &Grouping) -> R;
  fn visit_literal_expression(&self, expression: &Literal) -> R;
  fn visit_unary_expression(&self, expression: &Unary) -> R;
}
