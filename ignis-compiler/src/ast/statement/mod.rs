pub mod expression;
pub mod variable;

use expression::ExpressionStatement;
use variable::Variable;

use super::{visitor::Visitor, expression::Expression};

#[derive(Debug)]
pub enum Statement {
  Expression(ExpressionStatement),
  Variable(Variable),
}

impl Statement {
  pub fn accept<R>(&self, visitor: &mut dyn Visitor<R>) -> R {
    match self {
      Statement::Expression(expression) => visitor.visit_expression_statement(expression),
      Statement::Variable(variable) => visitor.visit_variable_statement(variable),
    }
  }
}
