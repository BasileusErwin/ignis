use std::collections::HashMap;

use super::{evaluator::EvaluatorValue, lexer::token::Token};

#[derive(Debug)]
pub struct VariableEnvironment {
  pub values: EvaluatorValue,
  pub is_mutable: bool,
}

impl VariableEnvironment {
  pub fn new(values: EvaluatorValue, is_mutable: bool) -> Self {
    Self { values, is_mutable }
  }
}

#[derive(Debug)]
pub struct Environment {
  pub values: HashMap<String, VariableEnvironment>,
  pub enclosing: Option<Box<Environment>>,
}

impl Environment {
  pub fn new(enclosing: Option<Box<Environment>>) -> Self {
    Self {
      values: HashMap::new(),
      enclosing,
    }
  }

  pub fn get(&self, name: Token) -> Result<Option<&VariableEnvironment>, ()> {
    if self.values.contains_key(name.span.literal.as_str()) {
      return Ok(self.values.get(name.span.literal.as_str()));
    }

    if let Some(enclosing) = &self.enclosing {
      return Ok(enclosing.get(name)?);
    }

    Err(())
  }

  pub fn define(&mut self, name: String, value: VariableEnvironment) -> Result<(), ()> {
    if self.values.contains_key(name.as_str()) {
      return Err(());
    }

    self.values.insert(name, value);

    Ok(())
  }

  pub fn assign(&mut self, name: Token, value: VariableEnvironment) -> Result<(), ()> {
    if self.values.contains_key(name.span.literal.as_str()) {
      if let Some(env) = self.values.get(name.span.literal.as_str()) {
        if !env.is_mutable {
          return Err(());
        }
      }

      self.values.insert(name.span.literal, value);
      return Ok(());
    }

    if let Some(enclosing) = &mut self.enclosing {
      enclosing.assign(name, value)?;
      return Ok(());
    }

    Err(())
  }
}
