use std::{
  rc::Rc,
  cell::{RefCell, Ref},
  env,
};

use crate::diagnostic::DiagnosticList;

use super::{
  visitor::Visitor,
  expression::{
    Expression, binary::Binary, literal::Literal, LiteralValue, grouping::Grouping, unary::Unary,
    variable::VariableExpression, assign::Assign,
  },
  lexer::token_type::TokenType,
  statement::{expression::ExpressionStatement, variable::Variable, Statement},
  environment::{Environment, VariableEnvironment},
};

#[derive(Debug, PartialEq, Clone)]
pub enum EvaluatorValue {
  String(String),
  Int(i64),
  Double(f64),
  Boolean(bool),
  Null,
  None,
}

impl EvaluatorValue {
  pub fn to_string(&self) -> String {
    match self {
      EvaluatorValue::String(_) => "string".to_string(),
      EvaluatorValue::Int(_) => "int".to_string(),
      EvaluatorValue::Double(_) => "double".to_string(),
      EvaluatorValue::Boolean(_) => "boolean".to_string(),
      EvaluatorValue::None | EvaluatorValue::Null => "null".to_string(),
    }
  }
}

pub struct Evaluator {
  environment: Rc<RefCell<Environment>>,
  pub diagnostics: Rc<RefCell<DiagnosticList>>,
}

impl Visitor<Result<EvaluatorValue, ()>> for Evaluator {
  fn visit_binary_expression(&self, expression: &Binary) -> Result<EvaluatorValue, ()> {
    let left = self.evaluator(&*expression.left)?;
    let right = self.evaluator(&*expression.right)?;
    let mut diagnostics = self.diagnostics.borrow_mut();

    let result: EvaluatorValue = match expression.operator.kind {
      TokenType::BangEqual => EvaluatorValue::Boolean(!self.is_equal(&left, &right)?),
      TokenType::EqualEqual => EvaluatorValue::Boolean(self.is_equal(&left, &right)?),
      TokenType::Greater => match (&left, &right) {
        (EvaluatorValue::Int(l), EvaluatorValue::Int(r)) => EvaluatorValue::Boolean(l > r),
        (EvaluatorValue::Double(l), EvaluatorValue::Double(r)) => EvaluatorValue::Boolean(l > r),
        _ => EvaluatorValue::None,
      },
      TokenType::GreaterEqual => match (&left, &right) {
        (EvaluatorValue::Int(l), EvaluatorValue::Int(r)) => EvaluatorValue::Boolean(l >= r),
        (EvaluatorValue::Double(l), EvaluatorValue::Double(r)) => EvaluatorValue::Boolean(l >= r),
        _ => EvaluatorValue::None,
      },
      TokenType::Less => match (&left, &right) {
        (EvaluatorValue::Int(l), EvaluatorValue::Int(r)) => EvaluatorValue::Boolean(l < r),
        (EvaluatorValue::Double(l), EvaluatorValue::Double(r)) => EvaluatorValue::Boolean(l < r),
        _ => EvaluatorValue::None,
      },
      TokenType::LessEqual => match (&left, &right) {
        (EvaluatorValue::Int(l), EvaluatorValue::Int(r)) => EvaluatorValue::Boolean(l <= r),
        (EvaluatorValue::Double(l), EvaluatorValue::Double(r)) => EvaluatorValue::Boolean(l <= r),
        _ => EvaluatorValue::None,
      },
      TokenType::Plus => match (&left, &right) {
        (EvaluatorValue::Int(l), EvaluatorValue::Int(r)) => EvaluatorValue::Int(l + r),
        (EvaluatorValue::Double(l), EvaluatorValue::Double(r)) => EvaluatorValue::Double(l + r),
        (EvaluatorValue::String(l), EvaluatorValue::String(r)) => {
          EvaluatorValue::String(format!("{}{}", l, r))
        }
        _ => EvaluatorValue::None,
      },
      TokenType::Minus => match (&left, &right) {
        (EvaluatorValue::Int(l), EvaluatorValue::Int(r)) => EvaluatorValue::Int(l - r),
        (EvaluatorValue::Double(l), EvaluatorValue::Double(r)) => EvaluatorValue::Double(l - r),
        _ => EvaluatorValue::None,
      },
      TokenType::Asterisk => match (&left, &right) {
        (EvaluatorValue::Int(l), EvaluatorValue::Int(r)) => EvaluatorValue::Int(l * r),
        (EvaluatorValue::Double(l), EvaluatorValue::Double(r)) => EvaluatorValue::Double(l * r),
        _ => EvaluatorValue::None,
      },
      TokenType::Slash => match (&left, &right) {
        (EvaluatorValue::Int(l), EvaluatorValue::Int(r)) => EvaluatorValue::Int(l / r),
        (EvaluatorValue::Double(l), EvaluatorValue::Double(r)) => EvaluatorValue::Double(l / r),
        _ => EvaluatorValue::None,
      },
      _ => {
        diagnostics.report_invalid_operator(&expression.operator);

        return Err(());
      }
    };

    if result == EvaluatorValue::None {
      diagnostics.report_invalid_operator_for_data_type(&expression.operator, &left, &right);

      return Err(());
    }

    Ok(result)
  }

  fn visit_grouping_expression(&self, expression: &Grouping) -> Result<EvaluatorValue, ()> {
    self.evaluator(&expression.expression)
  }

  fn visit_literal_expression(&self, expression: &Literal) -> Result<EvaluatorValue, ()> {
    match expression.value.clone() {
      LiteralValue::Boolean(value) => Ok(EvaluatorValue::Boolean(value)),
      LiteralValue::Double(value) => Ok(EvaluatorValue::Double(value)),
      LiteralValue::Int(value) => Ok(EvaluatorValue::Int(value)),
      LiteralValue::String(value) => Ok(EvaluatorValue::String(value)),
      LiteralValue::Char(_) | LiteralValue::None => Err(()),
    }
  }

  fn visit_unary_expression(&self, expression: &Unary) -> Result<EvaluatorValue, ()> {
    let right = self.evaluator(&expression.right)?;
    let mut diagnostics = self.diagnostics.borrow_mut();

    match expression.operator.kind {
      TokenType::Bang => Ok(EvaluatorValue::Boolean(!self.is_truthy(&right))),
      TokenType::Minus => match right {
        EvaluatorValue::Double(r) => Ok(EvaluatorValue::Double(-r)),
        EvaluatorValue::Int(r) => Ok(EvaluatorValue::Int(-r)),
        _ => {
          diagnostics.report_invalid_unary_operator_for_data_type(&expression.operator, &right);
          Err(())
        }
      },
      _ => {
        diagnostics.report_invalid_operator(&expression.operator);
        Err(())
      }
    }
  }

  fn visit_expression_statement(
    &self,
    statement: &ExpressionStatement,
  ) -> Result<EvaluatorValue, ()> {
    self.evaluator(&statement.expression)
  }

  fn visit_variable_statement(&self, variable: &Variable) -> Result<EvaluatorValue, ()> {
    let mut value: Option<EvaluatorValue> = None;

    if let Some(initializer) = &variable.initializer {
      value = Some(self.evaluator(initializer)?);
    }

    let mut environment = self.environment.borrow_mut();

    match environment.define(
      variable.name.span.literal.clone(),
      VariableEnvironment::new(value.unwrap(), variable.is_mutable),
    ) {
      Ok(_) => return Ok(EvaluatorValue::None),
      Err(_) => {
        self
          .diagnostics
          .borrow_mut()
          .report_invalid_redeclared_variable(&variable.name);

        return Err(());
      }
    };
  }

  fn visit_variable_expressin(&self, variable: &VariableExpression) -> Result<EvaluatorValue, ()> {
    let environment = self.environment.borrow_mut();
    let mut diagnostics = self.diagnostics.borrow_mut();

    match environment.get(variable.name.clone()) {
      Ok(env) => {
        if let Some(e) = env {
          Ok(e.values.clone())
        } else {
          Err(())
        }
      }
      Err(_) => {
        diagnostics.report_undeclared_variable(&variable.name);
        Err(())
      }
    }
  }

  fn visit_assign_expression(&self, expression: &Assign) -> Result<EvaluatorValue, ()> {
    let value = self.evaluator(&expression.value)?;
    let mut environment = self.environment.borrow_mut();
    let mut diagnostics = self.diagnostics.borrow_mut();

    match environment.assign(
      expression.name.clone(),
      VariableEnvironment::new(value.clone(), true),
    ) {
      Ok(_) => Ok(value),
      Err(_) => {
        diagnostics.report_invalid_redeclared_variable(&expression.name);
        Err(())
      }
    }
  }

  fn visit_logical_expression(
    &self,
    expression: &super::expression::logical::Logical,
  ) -> Result<EvaluatorValue, ()> {
    let left = self.evaluator(&expression.left)?;

    if expression.operator.kind == TokenType::Or {
      if self.is_truthy(&left) {
        return Ok(left);
      }
    } else {
      if !self.is_truthy(&left) {
        return Ok(left);
      }
    }

    self.evaluator(&expression.right)
  }
}

impl Evaluator {
  pub fn new() -> Self {
    Self {
      environment: Rc::new(RefCell::new(Environment::new(None))),
      diagnostics: Rc::new(RefCell::new(DiagnosticList::new())),
    }
  }

  pub fn evaluator(&self, expression: &Expression) -> Result<EvaluatorValue, ()> {
    expression.accept(self)
  }

  pub fn execute(&mut self, statement: Statement) {
    statement.accept(self);
  }

  fn is_equal(&self, left: &EvaluatorValue, right: &EvaluatorValue) -> Result<bool, ()> {
    match (left, right) {
      (EvaluatorValue::Boolean(l), EvaluatorValue::Boolean(r)) => Ok(l == r),
      (EvaluatorValue::Double(l), EvaluatorValue::Double(r)) => Ok(l == r),
      (EvaluatorValue::Int(l), EvaluatorValue::Int(r)) => Ok(l == r),
      (EvaluatorValue::String(l), EvaluatorValue::String(r)) => Ok(l == r),
      (EvaluatorValue::None, EvaluatorValue::None) => Ok(true),
      _ => Err(()),
    }
  }

  fn is_truthy(&self, value: &EvaluatorValue) -> bool {
    match value {
      EvaluatorValue::Boolean(v) => v.clone(),
      EvaluatorValue::String(v) => !v.is_empty(),
      EvaluatorValue::Int(_) | EvaluatorValue::Double(_) => true,
      EvaluatorValue::None | EvaluatorValue::Null => false,
    }
  }
}
