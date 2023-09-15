pub mod callable;
pub mod environment;
pub mod evaluator_error;
pub mod evaluator_value;
pub mod execution_error;

use ast::visitor::Visitor;
use evaluator_value::EvaluatorValue;
use evaluator_error::EvaluatorDiagnosticError;
use execution_error::ExecutionError;

use std::{rc::Rc, cell::RefCell};

use enums::token_type::TokenType;

use {
  lexer::token::Token,
  enums::literal_value::LiteralValue,
  ast::{
    expression::{
      binary::Binary, grouping::Grouping, literal::Literal, unary::Unary,
      variable::VariableExpression, assign::Assign, logical::Logical, ternary::Ternary, call::Call,
      Expression,
    },
    statement::{
      expression::ExpressionStatement, variable::Variable, block::Block, if_statement::IfStatement,
      while_statement::WhileStatement, function::FunctionStatement, return_statement::Return,
      Statement,
    },
  },
};

use self::{
  callable::{Callable, function::Function, print::Println},
  environment::{VariableEnvironment, Environment},
};

pub type EvaluatorResult<T> = Result<T, ExecutionError>;

pub struct Evaluator {
  environment: Rc<RefCell<Environment>>,
  pub diagnostics: Vec<EvaluatorDiagnosticError>,
}

impl Clone for Evaluator {
  fn clone(&self) -> Self {
    Self {
      environment: self.environment.clone(),
      diagnostics: self.diagnostics.clone(),
    }
  }
}

impl Visitor<EvaluatorResult<EvaluatorValue>> for Evaluator {
  fn visit_binary_expression(&mut self, expression: &Binary) -> EvaluatorResult<EvaluatorValue> {
    let left = self.evaluator(&*expression.left)?;
    let right = self.evaluator(&*expression.right)?;

    let result: EvaluatorValue = match expression.operator.kind {
      TokenType::BangEqual => self.is_equal(&left, &right, true, expression.operator.clone())?,
      TokenType::EqualEqual => self.is_equal(&left, &right, false, expression.operator.clone())?,
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
        return Err(ExecutionError::DiagnosticError(
          EvaluatorDiagnosticError::InvalidOperator(expression.operator.clone()),
        ))
      }
    };

    match result {
      EvaluatorValue::None => {
        return Err(ExecutionError::DiagnosticError(
          EvaluatorDiagnosticError::InvalidComparison(
            left.clone(),
            right.clone(),
            expression.operator.clone(),
          ),
        ))
      }
      _ => Ok(result),
    }
  }

  fn visit_grouping_expression(
    &mut self,
    expression: &Grouping,
  ) -> EvaluatorResult<EvaluatorValue> {
    self.evaluator(&expression.expression)
  }

  fn visit_literal_expression(&mut self, expression: &Literal) -> EvaluatorResult<EvaluatorValue> {
    match expression.value.clone() {
      LiteralValue::Boolean(value) => Ok(EvaluatorValue::Boolean(value)),
      LiteralValue::Double(value) => Ok(EvaluatorValue::Double(value)),
      LiteralValue::Int(value) => Ok(EvaluatorValue::Int(value)),
      LiteralValue::String(value) => Ok(EvaluatorValue::String(value)),
      LiteralValue::Char(_) | LiteralValue::Null => Ok(EvaluatorValue::None),
    }
  }

  fn visit_unary_expression(&mut self, expression: &Unary) -> EvaluatorResult<EvaluatorValue> {
    let right: EvaluatorValue = self.evaluator(&expression.right)?;

    match expression.operator.kind {
      TokenType::Bang => return Ok(EvaluatorValue::Boolean(!self.is_truthy(&right))),
      TokenType::Minus => match right {
        EvaluatorValue::Double(r) => return Ok(EvaluatorValue::Double(-r)),
        EvaluatorValue::Int(r) => return Ok(EvaluatorValue::Int(-r)),
        _ => {
          return Err(ExecutionError::DiagnosticError(
            EvaluatorDiagnosticError::InvalidUnaryOperatorForDataType(
              expression.operator.clone(),
              right.clone(),
            ),
          ))
        }
      },
      _ => (),
    };

    Err(ExecutionError::DiagnosticError(
      EvaluatorDiagnosticError::InvalidUnaryOperator(expression.operator.clone()),
    ))
  }

  fn visit_expression_statement(
    &mut self,
    statement: &ExpressionStatement,
  ) -> EvaluatorResult<EvaluatorValue> {
    self.evaluator(&statement.expression)
  }

  fn visit_variable_statement(&mut self, variable: &Variable) -> EvaluatorResult<EvaluatorValue> {
    let mut value: EvaluatorValue = EvaluatorValue::Null;

    if let Some(initializer) = &variable.initializer {
      value = self.evaluator(initializer)?;
    }

    let mut environment = self.environment.borrow_mut();

    match environment.define(
      variable.name.span.literal.clone(),
      VariableEnvironment::new(value.clone(), variable.metadata.is_mutable),
    ) {
      Ok(_) => Ok(value),
      Err(error) => Err(ExecutionError::DiagnosticError(error)),
    }
  }

  fn visit_variable_expression(
    &mut self,
    variable: &VariableExpression,
  ) -> EvaluatorResult<EvaluatorValue> {
    let environment = self.environment.borrow_mut();

    match environment.get(variable.name.clone()) {
      Ok(env) => {
        if let Some(e) = env {
          return Ok(e.values.clone());
        }
      }
      Err(error) => return Err(ExecutionError::DiagnosticError(error)),
    }

    Err(ExecutionError::DiagnosticError(
      EvaluatorDiagnosticError::UndeclaredVariable(variable.clone()),
    ))
  }

  // TODO: Validate if a variable is being declared for the first time without using the let or const keyword
  fn visit_assign_expression(&mut self, expression: &Assign) -> EvaluatorResult<EvaluatorValue> {
    let value: EvaluatorValue = self.evaluator(&expression.value)?;

    let mut environment = self.environment.borrow_mut();
    let mut diagnostics = self.diagnostics.clone();

    match environment.assign(
      &expression.name,
      VariableEnvironment::new(value.clone(), true),
      &mut diagnostics,
    ) {
      Ok(_) => Ok(value),
      Err(error) => Err(ExecutionError::DiagnosticError(error)),
    }
  }

  fn visit_logical_expression(&mut self, expression: &Logical) -> EvaluatorResult<EvaluatorValue> {
    let left: EvaluatorValue = self.evaluator(&expression.left)?;

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

  fn visit_block(&mut self, block: &Block) -> EvaluatorResult<EvaluatorValue> {
    self.execute_block(&block.statements, self.environment.clone())
  }

  fn visit_if_statement(&mut self, statement: &IfStatement) -> EvaluatorResult<EvaluatorValue> {
    let condition: EvaluatorValue = self.evaluator(&statement.condition)?;

    if self.is_truthy(&condition) {
      self.execute(statement.then_branch.as_ref())?;
    } else if let Some(else_branch) = &statement.else_branch {
      self.execute(else_branch.as_ref())?;
    }

    Ok(EvaluatorValue::None)
  }

  fn visit_while_statement(
    &mut self,
    statement: &WhileStatement,
  ) -> EvaluatorResult<EvaluatorValue> {
    loop {
      let evaluator = self.evaluator(&statement.condition)?;

      if !self.is_truthy(&evaluator) {
        break;
      }

      self.execute(statement.body.as_ref())?;
    }

    Ok(EvaluatorValue::None)
  }

  fn visit_ternary_expression(&mut self, expression: &Ternary) -> EvaluatorResult<EvaluatorValue> {
    let condition: EvaluatorValue = self.evaluator(&expression.condition)?;

    if self.is_truthy(&condition) {
      return self.evaluator(&expression.then_branch);
    } else {
      return self.evaluator(&expression.else_branch);
    }
  }

  fn visit_call_expression(&mut self, expression: &Call) -> EvaluatorResult<EvaluatorValue> {
    let calle = self.evaluator(&expression.callee)?;

    let mut arguments: Vec<EvaluatorValue> = Vec::new();

    for argument in &expression.arguments {
      arguments.push(self.evaluator(argument)?);
    }

    let function = match calle {
      EvaluatorValue::Callable(func) => func,
      _ => {
        return Err(ExecutionError::DiagnosticError(
          EvaluatorDiagnosticError::NotCallable(expression.paren.clone()),
        ))
      }
    };

    if arguments.len() != function.arity() {
      return Err(ExecutionError::DiagnosticError(
        EvaluatorDiagnosticError::InvalidNumberOfArguments(
          function.arity(),
          arguments.len(),
          expression.paren.clone(),
        ),
      ));
    }

    match function.call(arguments.clone(), &mut Box::new(self.clone())) {
      Ok(value) => Ok(value),
      Err(error) => match error {
        ExecutionError::Return(value) => Ok(value),
        _ => Err(error),
      },
    }
  }

  fn visit_function_statement(
    &mut self,
    statement: &FunctionStatement,
  ) -> EvaluatorResult<EvaluatorValue> {
    let environment = self.environment.borrow().clone();

    let function = Function::new(statement.clone(), Box::new(environment));

    let mut environment_mut = self.environment.borrow_mut();

    let _ = environment_mut.define(
      statement.name.span.literal.clone(),
      VariableEnvironment::new(EvaluatorValue::Callable(Box::new(function)), false),
    );

    Ok(EvaluatorValue::None)
  }

  fn visit_return_statement(&mut self, statement: &Return) -> EvaluatorResult<EvaluatorValue> {
    let mut value: Option<EvaluatorValue> = None;

    if let Some(expression) = &statement.value.clone() {
      value = Some(self.evaluator(expression)?);
    }

    match value {
      Some(v) => Err(ExecutionError::Return(v)),
      None => Err(ExecutionError::Return(EvaluatorValue::Null)),
    }
  }

  fn visit_class_statement(
    &mut self,
    statement: &ast::statement::class::Class,
  ) -> EvaluatorResult<EvaluatorValue> {
    todo!()
  }
}

impl Evaluator {
  pub fn new() -> Self {
    let mut environment = Environment::new(None);

    let print = VariableEnvironment::new(EvaluatorValue::Callable(Box::new(Println::new())), false);

    let _ = environment.define("println".to_string(), print);

    Self {
      environment: Rc::new(RefCell::new(environment)),
      diagnostics: Vec::new(),
    }
  }

  fn report_error(&mut self, error: EvaluatorDiagnosticError) {
    self.diagnostics.push(error);
  }

  pub fn evaluator(&mut self, expression: &Expression) -> EvaluatorResult<EvaluatorValue> {
    expression.accept(self)
  }

  pub fn execute(&mut self, statement: &Statement) -> EvaluatorResult<EvaluatorValue> {
    statement.accept(self)
  }

  pub fn execute_block(
    &mut self,
    statement: &Vec<Statement>,
    environment: Rc<RefCell<Environment>>,
  ) -> EvaluatorResult<EvaluatorValue> {
    let previous = self.environment.clone();

    self.environment = environment;

    for statement in statement {
      let result = self.execute(statement)?;

      if let EvaluatorValue::Return(_) = result {
        self.environment = previous;
        return Ok(result);
      }
    }

    self.environment = previous;

    Ok(EvaluatorValue::None)
  }

  fn is_equal(
    &self,
    left: &EvaluatorValue,
    right: &EvaluatorValue,
    bang: bool,
    token: Token,
  ) -> EvaluatorResult<EvaluatorValue> {
    let mut value = match (left, right) {
      (EvaluatorValue::Boolean(l), EvaluatorValue::Boolean(r)) => l == r,
      (EvaluatorValue::Double(l), EvaluatorValue::Double(r)) => l == r,
      (EvaluatorValue::Int(l), EvaluatorValue::Int(r)) => l == r,
      (EvaluatorValue::String(l), EvaluatorValue::String(r)) => l == r,
      (EvaluatorValue::None, EvaluatorValue::None) => true,
      _ => {
        return Err(ExecutionError::DiagnosticError(
          EvaluatorDiagnosticError::InvalidComparison(left.clone(), right.clone(), token.clone()),
        ))
      }
    };

    if bang {
      value = !value;
    }

    Ok(EvaluatorValue::Boolean(value))
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
