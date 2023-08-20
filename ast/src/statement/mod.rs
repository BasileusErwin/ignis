pub mod block;
pub mod expression;
pub mod function;
pub mod if_statement;
pub mod return_statement;
pub mod variable;
pub mod while_statement;

use self::{
  expression::ExpressionStatement, variable::Variable, if_statement::IfStatement, block::Block,
  while_statement::WhileStatement, function::FunctionStatement, return_statement::Return,
};

use crate::visitor::Visitor;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
  Expression(ExpressionStatement),
  Variable(Variable),
  Block(Block),
  IfStatement(IfStatement),
  WhileStatement(WhileStatement),
  FunctionStatement(FunctionStatement),
  Return(Return),
}

impl Statement {
  pub fn accept<R>(&self, visitor: &mut dyn Visitor<R>) -> R {
    match self {
      Statement::Expression(expression) => visitor.visit_expression_statement(expression),
      Statement::Variable(variable) => visitor.visit_variable_statement(variable),
      Statement::Block(block) => visitor.visit_block(block),
      Statement::IfStatement(if_statement) => visitor.visit_if_statement(if_statement),
      Statement::WhileStatement(while_statement) => visitor.visit_while_statement(while_statement),
      Statement::FunctionStatement(function_statement) => {
        visitor.visit_function_statement(function_statement)
      }
      Statement::Return(r) => visitor.visit_return_statement(r),
    }
  }
}
