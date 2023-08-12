use std::{rc::Rc, cell::RefCell};

use super::{
  visitor::Visitor,
  expression::{
    Expression, binary::Binary, literal::Literal, LiteralValue, grouping::Grouping, unary::Unary,
    variable::VariableExpression, assign::Assign,
  },
  lexer::token_type::TokenType,
  statement::{expression::ExpressionStatement, variable::Variable, Statement},
  environment::Environment,
};

#[derive(Debug, PartialEq, Clone)]
pub enum EvaluatorValue {
  String(String),
  Int(i64),
  Double(f64),
  Boolean(bool),
  None,
}

pub struct Evaluator {
  environment: Rc<RefCell<Environment>>,
}

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
      TokenType::Bang => EvaluatorValue::Boolean(!self.is_truthy(&right)),
      TokenType::Minus => match right {
        EvaluatorValue::Double(r) => EvaluatorValue::Double(-r),
        EvaluatorValue::Int(r) => EvaluatorValue::Int(-r),
        _ => EvaluatorValue::Boolean(false), // TODO: Error
      },
      _ => panic!("adssadad"), // TODO: Error
    }
  }

  fn visit_expression_statement(&self, statement: &ExpressionStatement) -> EvaluatorValue {
    self.evaluator(&statement.expression)
  }

  fn visit_variable_statement(&self, variable: &Variable) -> EvaluatorValue {
    let mut value: Option<EvaluatorValue> = None;

    if let Some(initializer) = &variable.initializer {
      value = Some(self.evaluator(initializer));
    }

    self
      .environment
      .borrow_mut()
      .define(variable.name.span.literal.clone(), value.unwrap());

    EvaluatorValue::None
  }

  fn visit_variable_expressin(&self, variable: &VariableExpression) -> EvaluatorValue {
    let environment = self.environment.borrow_mut();

    match environment.get(variable.name.clone()) {
      Ok(env) => {
        if let Some(e) = env {
          e.clone()
        } else {
          EvaluatorValue::None
        }
      }
      Err(e) => {
        println!("{:?}", e);
        EvaluatorValue::None
      }
    }
  }

  fn visit_assign_expression(&self, expression: &Assign) -> EvaluatorValue {
    let value = self.evaluator(&expression.value);
    let mut environment = self.environment.borrow_mut();

    match environment.assign(expression.name.clone(), value.clone()) {
      Ok(_) => value,
      Err(e) => {
        println!("{:?}", e);
        EvaluatorValue::None
      }
    }
  }

  fn visit_logical_expression(
    &self,
    expression: &super::expression::logical::Logical,
  ) -> EvaluatorValue {
    let left = self.evaluator(&expression.left);

    if expression.operator.kind == TokenType::Or {
      if self.is_truthy(&left) {
        return left;
      }
    } else {
      if !self.is_truthy(&left) {
        return left;
      }
    }

    self.evaluator(&expression.right)
  }
}

impl Evaluator {
  pub fn new() -> Self {
    Self {
      environment: Rc::new(RefCell::new(Environment::new(None))),
    }
  }
  
  pub fn evaluator(&self, expression: &Expression) -> EvaluatorValue {
    expression.accept(self)
  }

  pub fn execute(&mut self, statement: Statement) {
    statement.accept(self);
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

  fn is_truthy(&self, value: &EvaluatorValue) -> bool {
    match value {
      EvaluatorValue::Boolean(v) => v.clone(),
      EvaluatorValue::String(v) => !v.is_empty(),
      EvaluatorValue::None => false,
      EvaluatorValue::Int(_) | EvaluatorValue::Double(_) => true,
    }
  }
}
