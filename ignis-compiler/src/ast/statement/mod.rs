pub mod block;
pub mod expression;
pub mod if_statement;
pub mod variable;
pub mod while_statement;

use self::{
  expression::ExpressionStatement, variable::Variable, if_statement::IfStatement, block::Block,
  while_statement::WhileStatement,
};

use super::visitor::Visitor;

#[derive(Debug)]
pub enum Statement {
  Expression(ExpressionStatement),
  Variable(Variable),
  Block(Block),
  IfStatement(IfStatement),
  WhileStatement(WhileStatement),
}

impl Statement {
  pub fn accept<R>(&self, visitor: &mut dyn Visitor<R>) -> R {
    match self {
      Statement::Expression(expression) => visitor.visit_expression_statement(expression),
      Statement::Variable(variable) => visitor.visit_variable_statement(variable),
      Statement::Block(block) => visitor.visit_block(block),
      Statement::IfStatement(if_statement) => visitor.visit_if_statement(if_statement),
      Statement::WhileStatement(while_statement) => visitor.visit_while_statement(while_statement),
    }
  }
}
