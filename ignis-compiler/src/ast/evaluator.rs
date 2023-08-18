use std::{
  rc::Rc,
  cell::{RefCell, Ref},
  env, result,
};

use crate::diagnostic::DiagnosticList;

use super::{
  visitor::Visitor,
  expression::{
    Expression,
    binary::Binary,
    literal::Literal,
    LiteralValue,
    grouping::Grouping,
    unary::Unary,
    variable::VariableExpression,
    assign::Assign,
    ternary::Ternary,
    call::{Call, self},
    logical::Logical,
  },
  lexer::token_type::TokenType,
  statement::{
    expression::ExpressionStatement, variable::Variable, Statement, if_statement::IfStatement,
    block::Block, function::FunctionStatement, return_statement::Return,
  },
  environment::{Environment, VariableEnvironment, EnvironmentResult, self},
  data_type::DataType,
  callable::{Callable, function::Function, print::Println},
};

#[derive(Debug)]
pub enum EvaluatorValue {
  String(String),
  Int(i64),
  Double(f64),
  Boolean(bool),
  Callable(Box<dyn Callable>),
  Return(Box<EvaluatorValue>),
  Null,
  None,
}

impl EvaluatorValue {
  pub fn to_variable_environment(&self) -> VariableEnvironment {
    VariableEnvironment::new(self.clone(), true)
  }
}

impl Clone for EvaluatorValue {
  fn clone(&self) -> Self {
    match self {
      EvaluatorValue::String(s) => EvaluatorValue::String(s.clone()),
      EvaluatorValue::Int(i) => EvaluatorValue::Int(*i),
      EvaluatorValue::Double(d) => EvaluatorValue::Double(*d),
      EvaluatorValue::Boolean(b) => EvaluatorValue::Boolean(*b),
      EvaluatorValue::Null => EvaluatorValue::Null,
      EvaluatorValue::None => EvaluatorValue::None,
      EvaluatorValue::Callable(c) => EvaluatorValue::Callable(c.clone_box()),
      EvaluatorValue::Return(r) => r.as_ref().clone(),
    }
  }
}

impl EvaluatorValue {
  pub fn to_string(&self) -> String {
    match self {
      EvaluatorValue::String(_) => "string".to_string(),
      EvaluatorValue::Int(_) => "int".to_string(),
      EvaluatorValue::Double(_) => "double".to_string(),
      EvaluatorValue::Boolean(_) => "boolean".to_string(),
      EvaluatorValue::None | EvaluatorValue::Null => "null".to_string(),
      EvaluatorValue::Callable(_) => "callable".to_string(),
      EvaluatorValue::Return(_) => "return".to_string(),
    }
  }

  pub fn to_data_type(&self) -> DataType {
    match self {
      EvaluatorValue::String(_) => DataType::String,
      EvaluatorValue::Int(_) => DataType::Int,
      EvaluatorValue::Double(_) => DataType::Double,
      EvaluatorValue::Boolean(_) => DataType::Boolean,
      EvaluatorValue::None | EvaluatorValue::Null => DataType::None,
      EvaluatorValue::Callable(callee) => callee.get_type().unwrap(),
      EvaluatorValue::Return(r) => r.to_data_type(),
    }
  }
}

pub enum EvaluatorResult {
  Value(EvaluatorValue),
  Error,
}

pub struct Evaluator {
  environment: Rc<RefCell<Environment>>,
  pub diagnostics: Rc<RefCell<DiagnosticList>>,
}

impl Clone for Evaluator {
  fn clone(&self) -> Self {
    Self {
      environment: self.environment.clone(),
      diagnostics: self.diagnostics.clone(),
    }
  }
}

impl Visitor<EvaluatorResult> for Evaluator {
  fn visit_binary_expression(&self, expression: &Binary) -> EvaluatorResult {
    let result_left = self.evaluator(&*expression.left);
    let result_right = self.evaluator(&*expression.right);
    let left: EvaluatorValue;
    let right: EvaluatorValue;

    match result_left {
      EvaluatorResult::Value(v) => left = v,
      EvaluatorResult::Error => return EvaluatorResult::Error,
    }

    match result_right {
      EvaluatorResult::Value(v) => right = v,
      EvaluatorResult::Error => return EvaluatorResult::Error,
    }

    let mut diagnostics = self.diagnostics.borrow_mut();

    let result: EvaluatorValue = match expression.operator.kind {
      TokenType::BangEqual => {
        if let Ok(v) = self.is_equal(&left, &right) {
          EvaluatorValue::Boolean(!v)
        } else {
          EvaluatorValue::None
        }
      }
      TokenType::EqualEqual => {
        if let Ok(v) = self.is_equal(&left, &right) {
          EvaluatorValue::Boolean(v)
        } else {
          EvaluatorValue::None
        }
      }
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

        return EvaluatorResult::Error;
      }
    };

    match result {
      EvaluatorValue::None => {
        diagnostics.report_invalid_operator_for_data_type(&expression.operator, &left, &right);

        return EvaluatorResult::Error;
      }
      _ => EvaluatorResult::Value(result),
    }
  }

  fn visit_grouping_expression(&self, expression: &Grouping) -> EvaluatorResult {
    self.evaluator(&expression.expression)
  }

  fn visit_literal_expression(&self, expression: &Literal) -> EvaluatorResult {
    match expression.value.clone() {
      LiteralValue::Boolean(value) => EvaluatorResult::Value(EvaluatorValue::Boolean(value)),
      LiteralValue::Double(value) => EvaluatorResult::Value(EvaluatorValue::Double(value)),
      LiteralValue::Int(value) => EvaluatorResult::Value(EvaluatorValue::Int(value)),
      LiteralValue::String(value) => EvaluatorResult::Value(EvaluatorValue::String(value)),
      LiteralValue::Char(_) | LiteralValue::None => EvaluatorResult::Error,
    }
  }

  fn visit_unary_expression(&self, expression: &Unary) -> EvaluatorResult {
    let result = self.evaluator(&expression.right);
    let right: EvaluatorValue;
    let mut diagnostics = self.diagnostics.borrow_mut();

    match result {
      EvaluatorResult::Value(v) => right = v,
      EvaluatorResult::Error => return EvaluatorResult::Error,
    }

    match expression.operator.kind {
      TokenType::Bang => {
        return EvaluatorResult::Value(EvaluatorValue::Boolean(!self.is_truthy(&right)))
      }
      TokenType::Minus => match right {
        EvaluatorValue::Double(r) => return EvaluatorResult::Value(EvaluatorValue::Double(-r)),
        EvaluatorValue::Int(r) => return EvaluatorResult::Value(EvaluatorValue::Int(-r)),
        _ => diagnostics.report_invalid_unary_operator_for_data_type(&expression.operator, &right),
      },
      _ => diagnostics.report_invalid_operator(&expression.operator),
    };

    EvaluatorResult::Error
  }

  fn visit_expression_statement(&self, statement: &ExpressionStatement) -> EvaluatorResult {
    self.evaluator(&statement.expression)
  }

  fn visit_variable_statement(&self, variable: &Variable) -> EvaluatorResult {
    let mut value: EvaluatorValue = EvaluatorValue::Null;

    if let Some(initializer) = &variable.initializer {
      match self.evaluator(initializer) {
        EvaluatorResult::Value(v) => value = v,
        EvaluatorResult::Error => return EvaluatorResult::Error,
      }
    }

    let mut environment = self.environment.borrow_mut();

    match environment.define(
      variable.name.span.literal.clone(),
      VariableEnvironment::new(value.clone(), variable.is_mutable),
    ) {
      EnvironmentResult::Suscess(_) => {
        return EvaluatorResult::Value(value);
      }
      EnvironmentResult::Error => (),
    };

    return EvaluatorResult::Error;
  }

  fn visit_variable_expression(&self, variable: &VariableExpression) -> EvaluatorResult {
    let environment = self.environment.borrow_mut();
    let mut diagnostics = self.diagnostics.borrow_mut();

    match environment.get(variable.name.clone()) {
      EnvironmentResult::Suscess(env) => {
        if let Some(e) = env {
          return EvaluatorResult::Value(e.values.clone());
        }
      }
      EnvironmentResult::Error => {
        diagnostics.report_undeclared_variable(&variable.name);
      }
    };

    return EvaluatorResult::Error;
  }

  // TODO: Validate if a variable is being declared for the first time without using the let or const keyword
  fn visit_assign_expression(&self, expression: &Assign) -> EvaluatorResult {
    let value: EvaluatorValue = match self.evaluator(&expression.value) {
      EvaluatorResult::Value(v) => v,
      EvaluatorResult::Error => return EvaluatorResult::Error,
    };

    let mut environment = self.environment.borrow_mut();
    let mut diagnostics = self.diagnostics.borrow_mut();

    match environment.assign(
      &expression.name,
      VariableEnvironment::new(value.clone(), true),
      &mut diagnostics,
    ) {
      EnvironmentResult::Suscess(_) => return EvaluatorResult::Value(value),
      EnvironmentResult::Error => (),
    };

    return EvaluatorResult::Error;
  }

  fn visit_logical_expression(&self, expression: &Logical) -> EvaluatorResult {
    let left: EvaluatorValue = match self.evaluator(&expression.left) {
      EvaluatorResult::Value(value) => value,
      EvaluatorResult::Error => return EvaluatorResult::Error,
    };

    if expression.operator.kind == TokenType::Or {
      if self.is_truthy(&left) {
        return EvaluatorResult::Value(left);
      }
    } else {
      if !self.is_truthy(&left) {
        return EvaluatorResult::Value(left);
      }
    }

    self.evaluator(&expression.right)
  }

  fn visit_block(&mut self, block: &Block) -> EvaluatorResult {
    self.execute_block(&block.statements, self.environment.clone())
  }

  fn visit_if_statement(&mut self, statement: &IfStatement) -> EvaluatorResult {
    let condition: EvaluatorValue = match self.evaluator(&statement.condition) {
      EvaluatorResult::Value(value) => value,
      EvaluatorResult::Error => return EvaluatorResult::Error,
    };

    if self.is_truthy(&condition) {
      self.execute(statement.then_branch.as_ref());
    } else if let Some(else_branch) = &statement.else_branch {
      self.execute(else_branch.as_ref());
    }

    EvaluatorResult::Value(EvaluatorValue::None)
  }

  fn visit_while_statement(
    &mut self,
    statement: &super::statement::while_statement::WhileStatement,
  ) -> EvaluatorResult {
    loop {
      let evaluator = self.evaluator(&statement.condition);

      match evaluator {
        EvaluatorResult::Value(value) => {
          if !self.is_truthy(&value) {
            break;
          }

          self.execute(statement.body.as_ref());
        }
        EvaluatorResult::Error => return EvaluatorResult::Error,
      }
    }

    EvaluatorResult::Value(EvaluatorValue::None)
  }

  fn visit_ternary_expression(&self, expression: &Ternary) -> EvaluatorResult {
    let condition: EvaluatorValue = match self.evaluator(&expression.condition) {
      EvaluatorResult::Value(value) => value,
      EvaluatorResult::Error => return EvaluatorResult::Error,
    };

    if self.is_truthy(&condition) {
      return self.evaluator(&expression.then_branch);
    } else {
      return self.evaluator(&expression.else_branch);
    }
  }

  fn visit_call_expression(&self, expression: &Call) -> EvaluatorResult {
    let calle = match self.evaluator(&expression.callee) {
      EvaluatorResult::Value(value) => value,
      EvaluatorResult::Error => return EvaluatorResult::Error,
    };

    let mut arguments: Vec<EvaluatorValue> = Vec::new();

    for argument in &expression.arguments {
      match self.evaluator(argument) {
        EvaluatorResult::Value(value) => arguments.push(value),
        EvaluatorResult::Error => return EvaluatorResult::Error,
      }
    }

    let function = match calle {
      EvaluatorValue::Callable(func) => func,
      _ => return EvaluatorResult::Error,
    };

    if arguments.len() != function.arity() {
      let mut diagnostics = self.diagnostics.borrow_mut();

      diagnostics.report_invalid_number_of_arguments(
        function.arity(),
        arguments.len(),
        &expression.paren,
      );

      return EvaluatorResult::Error;
    }

    function.call(arguments.clone(), &mut Box::new(self.clone()))
  }

  fn visit_function_statement(&mut self, statement: &FunctionStatement) -> EvaluatorResult {
    let environment = self.environment.borrow().clone();

    let function = Function::new(statement.clone(), Box::new(environment));

    let mut environment_mut = self.environment.borrow_mut();

    environment_mut.define(
      statement.name.span.literal.clone(),
      VariableEnvironment::new(EvaluatorValue::Callable(Box::new(function)), false),
    );

    EvaluatorResult::Value(EvaluatorValue::None)
  }

  fn visit_return_statement(&mut self, statement: &Return) -> EvaluatorResult {
    let mut value: Option<EvaluatorValue> = None;

    if let Some(expression) = &statement.value {
      match self.evaluator(expression) {
        EvaluatorResult::Value(v) => value = Some(v),
        EvaluatorResult::Error => return EvaluatorResult::Error,
      };
    }

    match value {
      Some(v) => EvaluatorResult::Value(v),
      None => EvaluatorResult::Value(EvaluatorValue::None),
    }
  }
}

impl Evaluator {
  pub fn new() -> Self {
    let mut environment = Environment::new(None);

    let print = VariableEnvironment::new(EvaluatorValue::Callable(Box::new(Println::new())), false);

    environment.define("println".to_string(), print);

    Self {
      environment: Rc::new(RefCell::new(environment)),
      diagnostics: Rc::new(RefCell::new(DiagnosticList::new())),
    }
  }

  pub fn evaluator(&self, expression: &Expression) -> EvaluatorResult {
    expression.accept(self)
  }

  pub fn execute(&mut self, statement: &Statement) {
    statement.accept(self);
  }

  pub fn execute_block(
    &mut self,
    statement: &Vec<Statement>,
    environment: Rc<RefCell<Environment>>,
  ) -> EvaluatorResult {
    let previous = self.environment.clone();

    self.environment = environment;

    for statement in statement {
      self.execute(&statement);
    }

    self.environment = previous;

    EvaluatorResult::Value(EvaluatorValue::None)
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
      EvaluatorValue::Callable(_) => false,
      EvaluatorValue::Return(_) => false,
    }
  }
}
