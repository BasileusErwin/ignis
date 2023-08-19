pub mod callable;
pub mod data_type;
pub mod environment;
pub mod evaluator;
pub mod expression;
pub mod lexer;
pub mod parser;
pub mod statement;
pub mod visitor;

use std::{vec, rc::Rc, cell::RefCell};

use crate::{
  ast::evaluator::EvaluatorValue,
  diagnostic::{DiagnosticList, self, error::DiagnosticError},
};

use self::{statement::Statement, visitor::Visitor, evaluator::EvaluatorResult};

#[derive(Debug)]
pub struct Ast {
  pub statements: Vec<Statement>,
}

impl Ast {
  pub fn new(statements: Vec<Statement>) -> Self {
    Self { statements }
  }

  pub fn add(&mut self, statement: Statement) {
    self.statements.push(statement);
  }

  pub fn visit(
    &mut self,
    diagnostics: &mut DiagnosticList,
    visitor: &mut dyn Visitor<EvaluatorResult<EvaluatorValue>>,
  ) {
    for statement in &self.statements {
      match statement.accept(visitor) {
        Ok(_) => continue,
        Err(error) => error.report(diagnostics),
      }
    }
  }
}
