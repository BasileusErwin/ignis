use crate::ast::{
  expression::Expression,
  lexer::{token::Token, token_type::TokenType},
};

use super::Visitor;

#[derive(Debug)]
pub enum StatementType {
  Number,
  String,
  Int,
  Double,
  Boolean,
  Char,
  None,
  // TODO: Type non-primitive
  ClassType(String),
  GenericType {
    base: Box<StatementType>,
    parameters: Vec<StatementType>,
  },
  UnionType(Vec<StatementType>),
  IntersectionType(Vec<StatementType>),
  TupleType(Vec<StatementType>),
  LiteralType(String),
  AliasType(String),
}

impl StatementType {
  pub fn from_token_type(kind: TokenType) -> Self {
    match kind {
      TokenType::NumberType => StatementType::Number,
      TokenType::StringType => StatementType::String,
      TokenType::DoubleType => StatementType::Double,
      TokenType::CharType => StatementType::Char,
      TokenType::BooleanType => StatementType::Boolean,
      TokenType::IntType => StatementType::Int,
      _ => StatementType::None,
    }
  }
}

#[derive(Debug)]
pub struct Variable {
  name: Box<Token>,
  initializer: Box<Expression>,
  type_annotation: Box<StatementType>,
}

impl Variable {
  pub fn new(
    name: Box<Token>,
    initializer: Box<Expression>,
    type_annotation: Box<StatementType>,
  ) -> Self {
    Self {
      name,
      initializer,
      type_annotation,
    }
  }

  pub fn accept<R>(&self, visitor: &dyn Visitor<R>) -> R {
    visitor.visit_variable_statement(self)
  }
}
