use self::{
  binary::Binary, grouping::Grouping, literal::Literal, unary::Unary, variable::VariableExpression,
  logical::Logical, assign::Assign, ternary::Ternary, call::Call,
};

use super::visitor::Visitor;

pub mod assign;
pub mod binary;
pub mod call;
pub mod grouping;
pub mod literal;
pub mod logical;
pub mod ternary;
pub mod unary;
pub mod variable;

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
  Binary(Binary),
  Grouping(Grouping),
  Literal(Literal),
  Unary(Unary),
  Variable(VariableExpression),
  Assign(Assign),
  Logical(Logical),
  Ternary(Ternary),
  Call(Call),
}

impl Expression {
  pub fn accept<R>(&self, visitor: &mut dyn Visitor<R>) -> R {
    match self {
      Expression::Grouping(grouping) => visitor.visit_grouping_expression(grouping),
      Expression::Binary(binary) => visitor.visit_binary_expression(binary),
      Expression::Literal(literal) => visitor.visit_literal_expression(literal),
      Expression::Unary(unary) => visitor.visit_unary_expression(unary),
      Expression::Variable(variable) => visitor.visit_variable_expression(variable),
      Expression::Assign(assign) => visitor.visit_assign_expression(assign),
      Expression::Logical(logical) => visitor.visit_logical_expression(logical),
      Expression::Ternary(ternary) => visitor.visit_ternary_expression(ternary),
      Expression::Call(call) => visitor.visit_call_expression(call),
    }
  }

  pub fn to_string(&self) -> String {
    match self {
      Expression::Binary(Binary {
        left,
        operator,
        right,
        data_type,
      }) => format!(
        "({} {} {}): {}",
        operator.span.literal,
        left.to_string(),
        right.to_string(),
        data_type.to_string(),
      ),
      Expression::Grouping(Grouping { expression }) => {
        format!("(group {})", (*expression).to_string())
      }
      Expression::Literal(Literal { value }) => format!("{}", value.to_string()),
      Expression::Unary(Unary {
        operator,
        right,
        data_type,
      }) => {
        let operator_str = operator.span.literal.clone();
        let right_str = (*right).to_string();
        format!(
          "({} {}): {}",
          operator_str,
          right_str,
          data_type.to_string()
        )
      }
      Expression::Variable(VariableExpression { name, data_type }) => {
        format!("{:?}: {:?}", name, data_type)
      }
      Expression::Assign(Assign { name, value, .. }) => {
        format!("{} = {}", name.span.literal, value.to_string())
      }
      Expression::Logical(Logical {
        left,
        operator,
        right,
        ..
      }) => format!(
        "({} {} {})",
        left.to_string(),
        operator.span.literal,
        right.to_string()
      ),
      Expression::Ternary(Ternary {
        condition,
        then_branch,
        else_branch,
        ..
      }) => format!(
        "({} ? {} : {})",
        condition.to_string(),
        then_branch.to_string(),
        else_branch.to_string()
      ),
      Expression::Call(call) => format!(
        "fn {}({})",
        call.callee.to_string(),
        call
          .arguments
          .iter()
          .map(|x| x.to_string())
          .collect::<Vec<String>>()
          .join(", ")
      ),
    }
  }
}
