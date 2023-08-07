pub mod expression;
pub mod variable;

use expression::ExpressionStatement;
use variable::Variable;

#[derive(Debug)]
pub enum Statement {
  Expression(ExpressionStatement),
  Variable(Variable),
}

pub trait Visitor<R> {
  fn visit_expression_statement(&self, statement: &ExpressionStatement) -> R;
  fn visit_variable_statement(&self, variable: &Variable) -> R;
}
