use std::{collections::HashMap, rc::Rc, cell::RefCell};

use crate::diagnostic::DiagnosticList;

use super::{evaluator::EvaluatorValue, lexer::token::Token};

pub enum EnvironmentResult<T> {
  Suscess(T),
  Error,
}

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
      return EnvironmentResult::Suscess(self.values.get(name.span.literal.as_str()));
    }

    if let Some(enclosing) = &self.enclosing {
      return enclosing.get(name);
    }

    EnvironmentResult::Error
  }

  pub fn define(&mut self, name: String, value: VariableEnvironment) -> EnvironmentResult<()> {
    if self.values.contains_key(name.as_str()) {
      return EnvironmentResult::Error;
    }

    self.values.insert(name, value);

    EnvironmentResult::Suscess(())
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
          diagnostics.report_invalid_reassigned_variable(&name);
          return EnvironmentResult::Error;
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
            diagnostics.report_assing_invalid_type(
              &value.values.to_data_type(),
              &env.values.to_data_type(),
              &name,
            );

            return EnvironmentResult::Error;
          }
        }
      }

      self.values.insert(name.span.literal.clone(), value);
      return EnvironmentResult::Suscess(());
    }

    if let Some(enclosing) = &mut self.enclosing {
      return enclosing.assign(name, value, diagnostics);
    }

    EnvironmentResult::Error
  }
}
