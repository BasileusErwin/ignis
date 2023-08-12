use std::collections::HashMap;

use super::{evaluator::EvaluatorValue, lexer::token::Token};

#[derive(Debug)]
pub struct Environment {
  pub values: HashMap<String, EvaluatorValue>,
  pub enclosing: Option<Box<Environment>>,
}

impl Environment {
  pub fn new(enclosing: Option<Box<Environment>>) -> Self {
    Self {
      values: HashMap::new(),
      enclosing,
    }
  }

  pub fn get(&self, name: Token) -> Result<Option<&EvaluatorValue>, String> {
    if self.values.contains_key(name.span.literal.as_str()) {
      return Ok(self.values.get(name.span.literal.as_str()));
    }

    if let Some(enclosing) = &self.enclosing {
      return Ok(enclosing.get(name)?);
    }

    Err(format!("Undefined variable '{}'.", name.span.literal))
  }

  pub fn define(&mut self, name: String, value: EvaluatorValue) {
    self.values.insert(name, value);
  }

  pub fn assign(&mut self, name: Token, value: EvaluatorValue) -> Result<(), String> {
    if self.values.contains_key(name.span.literal.as_str()) {
      self.define(name.span.literal, value);
      return Ok(());
    }

    if let Some(enclosing) = &mut self.enclosing {
      enclosing.assign(name, value)?;
      return Ok(());
    }

    Err(format!("Undefined variable '{}'.", name.span.literal))
  }
}
