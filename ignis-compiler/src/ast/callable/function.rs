use std::{cell::RefCell, rc::Rc, borrow::BorrowMut};

use crate::ast::{
  data_type::DataType,
  evaluator::{Evaluator, EvaluatorValue, EvaluatorResult},
  statement::function::FunctionStatement,
  environment::{Environment, self},
};

use super::Callable;

#[derive(Debug)]
pub struct Function {
  declaration: FunctionStatement,
  closure: Box<Environment>,
}

impl Clone for Function {
  fn clone(&self) -> Self {
    Self {
      declaration: self.declaration.clone(),
      closure: self.closure.clone(),
    }
  }
}

impl Function {
  pub fn new(declaration: FunctionStatement, closure: Box<Environment>) -> Self {
    Self {
      declaration,
      closure,
    }
  }
}

impl Callable for Function {
  fn call(
    &self,
    arguments: Vec<EvaluatorValue>,
    evaluator: &mut Box<Evaluator>,
  ) -> EvaluatorResult {
    let mut environment = Environment::new(Some(self.closure.clone()));

    for (i, parameter) in self.declaration.parameters.iter().enumerate() {
      match arguments.get(i) {
        Some(argument) => {
          environment.define(
            parameter.span.literal.clone(),
            argument.to_variable_environment(),
          );
        }
        None => {
          return EvaluatorResult::Error;
        }
      };
    }

    let result =
      evaluator.execute_block(&self.declaration.body, Rc::new(RefCell::new(environment)));

    // TODO: Return value
    match result {
      EvaluatorResult::Error => EvaluatorResult::Error,
      EvaluatorResult::Value(value) => EvaluatorResult::Value(value),
    }
  }

  fn arity(&self) -> usize {
    self.declaration.parameters.len()
  }

  fn get_type(&self) -> Option<DataType> {
    self.declaration.return_type.clone()
  }

  fn clone_box(&self) -> Box<dyn Callable> {
    Box::new(self.clone())
  }
}
