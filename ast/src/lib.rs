pub mod expression;
pub mod statement;
pub mod visitor;

use self::statement::Statement;

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
}
