use std::collections::HashMap;

use crate::diagnostic::{DiagnosticList, error::DiagnosticError};

use super::{evaluator::EvaluatorValue, lexer::token::Token};

type EnvironmentResult<T> = Result<T, DiagnosticError>;

#[derive(Debug)]
pub struct VariableEnvironment {
  pub values: EvaluatorValue,
  pub is_mutable: bool,
}

impl Clone for VariableEnvironment {
  fn clone(&self) -> Self {
    Self {
      values: self.values.clone(),
      is_mutable: self.is_mutable,
    }
  }
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

impl Clone for Environment {
  fn clone(&self) -> Self {
    Self {
      values: self.values.clone(),
      enclosing: self.enclosing.clone(),
    }
  }
}

impl Environment {
  pub fn new(enclosing: Option<Box<Environment>>) -> Self {
    Self {
      values: HashMap::new(),
      enclosing,
    }
  }

  pub fn get(&self, name: Token) -> EnvironmentResult<Option<&VariableEnvironment>> {
    if self.values.contains_key(name.span.literal.as_str()) {
      return Ok(self.values.get(name.span.literal.as_str()));
    }

    if let Some(enclosing) = &self.enclosing {
      return enclosing.get(name);
    }

    Err(DiagnosticError::UndefinedVariable(name))
  }

  pub fn define(&mut self, name: String, value: VariableEnvironment) -> EnvironmentResult<()> {
    let name_string = name.clone();
    let name_str = name.as_str();
    if self.values.contains_key(name_str) {
      return Err(DiagnosticError::VariableAlreadyDefined(
        name_string,
        self.values.get(name_str).unwrap().values.to_data_type(),
      ));
    }

    self.values.insert(name, value);

    Ok(())
  }

  pub fn assign(
    &mut self,
    name: &Token,
    value: VariableEnvironment,
    diagnostics: &mut DiagnosticList,
  ) -> EnvironmentResult<()> {
    if self.values.contains_key(name.span.literal.as_str()) {
      if let Some(env) = self.values.get(name.span.literal.as_str()) {
        if !env.is_mutable {
          return Err(DiagnosticError::InvalidAssignmentTarget(name.span.clone()));
        }

        match (&value, env) {
          (
            VariableEnvironment {
              values: EvaluatorValue::Int { .. },
              ..
            },
            VariableEnvironment {
              values: EvaluatorValue::Int { .. },
              ..
            },
          )
          | (
            VariableEnvironment {
              values: EvaluatorValue::String { .. },
              ..
            },
            VariableEnvironment {
              values: EvaluatorValue::String { .. },
              ..
            },
          )
          | (
            VariableEnvironment {
              values: EvaluatorValue::Boolean { .. },
              ..
            },
            VariableEnvironment {
              values: EvaluatorValue::Boolean { .. },
              ..
            },
          )
          | (
            VariableEnvironment {
              values: EvaluatorValue::Double { .. },
              ..
            },
            VariableEnvironment {
              values: EvaluatorValue::Double { .. },
              ..
            },
          ) => (),
          _ => {
            return Err(DiagnosticError::AssingInvalidType(
              value.values.to_data_type(),
              env.values.to_data_type(),
              name.clone(),
            ))
          }
        }
      }

      self.values.insert(name.span.literal.clone(), value);

      return Ok(());
    }

    if let Some(enclosing) = &mut self.enclosing {
      return enclosing.assign(name, value, diagnostics);
    }

    Err(DiagnosticError::UndefinedVariable(name.clone()))
  }
}
