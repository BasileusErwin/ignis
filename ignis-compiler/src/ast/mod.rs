pub mod parser;
pub mod statement;
pub mod visitor;
pub mod expression;
pub mod lexer;

use crate::{
  diagnostic::DiagnosticList,
  evaluator::{EvaluatorResult, EvaluatorValue},
};

use self::{statement::Statement, visitor::Visitor};

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
