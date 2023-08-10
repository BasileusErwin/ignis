use std::f32::consts::E;

use super::{
  visitor::Visitor,
  expression::{
    Expression, binary::Binary, literal::Literal, LiteralValue, grouping::Grouping, unary::Unary,
  },
  lexer::token_type::TokenType,
  statement::{expression::ExpressionStatement, variable::Variable},
};

#[derive(Debug, PartialEq)]
pub enum EvaluatorValue {
  String(String),
  Int(i64),
  Double(f64),
  Boolean(bool),
  None,
}

pub struct Evaluator;

impl Visitor<EvaluatorValue> for Evaluator {
  fn visit_binary_expression(&self, expression: &Binary) -> EvaluatorValue {
    let left = self.evaluator(&*expression.left);
    let right = self.evaluator(&*expression.right);

    // TODO: Error by type
    match expression.operator.kind {
      TokenType::BangEqual => EvaluatorValue::Boolean(!self.is_equal(left, right)),
      TokenType::EqualEqual => EvaluatorValue::Boolean(self.is_equal(left, right)),
      TokenType::Greater => match (left, right) {
        (EvaluatorValue::Int(l), EvaluatorValue::Int(r)) => EvaluatorValue::Boolean(l > r),
        (EvaluatorValue::Double(l), EvaluatorValue::Double(r)) => EvaluatorValue::Boolean(l > r),
        _ => EvaluatorValue::Boolean(false), // TODO: Error
      },
      TokenType::GreaterEqual => match (left, right) {
        (EvaluatorValue::Int(l), EvaluatorValue::Int(r)) => EvaluatorValue::Boolean(l >= r),
        (EvaluatorValue::Double(l), EvaluatorValue::Double(r)) => EvaluatorValue::Boolean(l >= r),
        _ => EvaluatorValue::Boolean(false), // TODO: Error
      },
      TokenType::Less => match (left, right) {
        (EvaluatorValue::Int(l), EvaluatorValue::Int(r)) => EvaluatorValue::Boolean(l < r),
        (EvaluatorValue::Double(l), EvaluatorValue::Double(r)) => EvaluatorValue::Boolean(l < r),
        _ => EvaluatorValue::Boolean(false), // TODO: Error
      },
      TokenType::LessEqual => match (left, right) {
        (EvaluatorValue::Int(l), EvaluatorValue::Int(r)) => EvaluatorValue::Boolean(l <= r),
        (EvaluatorValue::Double(l), EvaluatorValue::Double(r)) => EvaluatorValue::Boolean(l <= r),
        _ => EvaluatorValue::Boolean(false), // TODO: Error
      },
      TokenType::Plus => match (left, right) {
        (EvaluatorValue::Int(l), EvaluatorValue::Int(r)) => EvaluatorValue::Int(l + r),
        (EvaluatorValue::Double(l), EvaluatorValue::Double(r)) => EvaluatorValue::Double(l + r),
        (EvaluatorValue::String(l), EvaluatorValue::String(r)) => {
          EvaluatorValue::String(format!("{}{}", l, r))
        }
        _ => EvaluatorValue::Boolean(false),
      },
      TokenType::Minus => match (left, right) {
        (EvaluatorValue::Int(l), EvaluatorValue::Int(r)) => EvaluatorValue::Int(l - r),
        (EvaluatorValue::Double(l), EvaluatorValue::Double(r)) => EvaluatorValue::Double(l - r),
        _ => EvaluatorValue::Boolean(false),
      },
      TokenType::Asterisk => match (left, right) {
        (EvaluatorValue::Int(l), EvaluatorValue::Int(r)) => EvaluatorValue::Int(l * r),
        (EvaluatorValue::Double(l), EvaluatorValue::Double(r)) => EvaluatorValue::Double(l * r),
        _ => EvaluatorValue::Boolean(false),
      },
      TokenType::Slash => match (left, right) {
        (EvaluatorValue::Int(l), EvaluatorValue::Int(r)) => EvaluatorValue::Int(l / r),
        (EvaluatorValue::Double(l), EvaluatorValue::Double(r)) => EvaluatorValue::Double(l / r),
        _ => EvaluatorValue::Boolean(false),
      },
      _ => panic!("Unsupported binary operator!"), // Error
    }
  }

  fn visit_grouping_expression(&self, expression: &Grouping) -> EvaluatorValue {
		self.evaluator(&expression.expression)
  }

  fn visit_literal_expression(&self, expression: &Literal) -> EvaluatorValue {
    match expression.value.clone() {
      LiteralValue::Boolean(value) => EvaluatorValue::Boolean(value),
      LiteralValue::Double(value) => EvaluatorValue::Double(value),
      LiteralValue::Int(value) => EvaluatorValue::Int(value),
      LiteralValue::String(value) => EvaluatorValue::String(value),
      LiteralValue::Char(_) | LiteralValue::None => EvaluatorValue::None,
    }
  }

  fn visit_unary_expression(&self, expression: &Unary) -> EvaluatorValue {
    let right = self.evaluator(&expression.right);

    match expression.operator.kind {
      TokenType::Bang => EvaluatorValue::Boolean(!self.is_truthy(right)),
			TokenType::Minus => match right {
				EvaluatorValue::Double(r) => EvaluatorValue::Double(-r),
				EvaluatorValue::Int(r) => EvaluatorValue::Int(-r),
				_ => EvaluatorValue::Boolean(false), // TODO: Error
			}
			_ => panic!("adssadad"), // TODO: Error
    }
  }

  fn visit_expression_statement(&self, statement: &ExpressionStatement) -> EvaluatorValue {
    self.evaluator(&statement.expression)
  }

  fn visit_variable_statement(&self, variable: &Variable) -> EvaluatorValue {
    todo!()
  }
}

impl Evaluator {
  pub fn new() -> Self {
    Self {}
  }

  pub fn evaluator(&self, expression: &Expression) -> EvaluatorValue {
    expression.accept(self)
  }

  fn is_equal(&self, left: EvaluatorValue, right: EvaluatorValue) -> bool {
    match (left, right) {
      (EvaluatorValue::Boolean(l), EvaluatorValue::Boolean(r)) => l == r,
      (EvaluatorValue::Double(l), EvaluatorValue::Double(r)) => l == r,
      (EvaluatorValue::Int(l), EvaluatorValue::Int(r)) => l == r,
      (EvaluatorValue::String(l), EvaluatorValue::String(r)) => l == r,
      (EvaluatorValue::None, EvaluatorValue::None) => true,
      _ => false, // Error
    }
  }

  fn is_truthy(&self, value: EvaluatorValue) -> bool {
    match value {
      EvaluatorValue::Boolean(v) => v,
      EvaluatorValue::None => false,
      EvaluatorValue::String(_) | EvaluatorValue::Int(_) | EvaluatorValue::Double(_) => true,
    }
  }
}
