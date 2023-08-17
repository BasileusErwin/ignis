use super::{
  expression::{
    binary::Binary, literal::Literal, unary::Unary, grouping::Grouping,
    variable::VariableExpression, assign::Assign, logical::Logical,
  },
  statement::{
    expression::ExpressionStatement, variable::Variable, if_statement::IfStatement, block::Block,
    while_statement::WhileStatement,
  },
};

pub trait Visitor<R> {
  // Expression
  fn visit_binary_expression(&self, expression: &Binary) -> R;
  fn visit_grouping_expression(&self, expression: &Grouping) -> R;
  fn visit_literal_expression(&self, expression: &Literal) -> R;
  fn visit_unary_expression(&self, expression: &Unary) -> R;
  fn visit_variable_expressin(&self, variable: &VariableExpression) -> R;
  fn visit_assign_expression(&self, expression: &Assign) -> R;
  fn visit_logical_expression(&self, expression: &Logical) -> R;

  // Statements
  fn visit_expression_statement(&self, statement: &ExpressionStatement) -> R;
  fn visit_variable_statement(&self, variable: &Variable) -> R;
  fn visit_block(&mut self, block: &Block) -> R;
  fn visit_if_statement(&mut self, statement: &IfStatement) -> R;
  fn visit_while_statement(&mut self, statement: &WhileStatement) -> R;
}
