use serde_json::json;

use self::{
  binary::Binary, grouping::Grouping, literal::Literal, unary::Unary, variable::VariableExpression,
  logical::Logical, assign::Assign, ternary::Ternary, call::Call, array::Array, get::Get
};

use super::visitor::Visitor;

pub mod array;
pub mod assign;
pub mod binary; pub mod call;
pub mod grouping;
pub mod literal;
pub mod logical;
pub mod new;
pub mod ternary;
pub mod unary;
pub mod variable;
pub mod get;

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
  Array(Array),
  Get(Get)
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
      Expression::Array(array) => visitor.visit_array_expression(array),
      Expression::Get(get) => visitor.visit_get_expression(get),
          
    }
  }

  pub fn to_json(&self) -> serde_json::Value {
    match self {
      Expression::Binary(binary) => {
        json!({
          "type": "Binary",
          "left": binary.left.to_json(),
          "operator": binary.operator.span.literal,
          "right": binary.right.to_json(),
        })
      }
      Expression::Grouping(grouping) => {
        json!({
          "type": "Grouping",
          "expression": grouping.expression.to_json(),
        })
      }
      Expression::Literal(literal) => {
        json!({
          "type": "Literal",
          "value": literal.value.to_string(),
        })
      }
      Expression::Unary(unary) => {
        json!({
          "type": "Unary",
          "operator": unary.operator.span.literal,
          "right": unary.right.to_json(),
          "data_type": unary.data_type.to_string(),
        })
      }
      Expression::Variable(variable) => {
        json!({
          "type": "Variable",
          "name": variable.name.span.literal,
          "data_type": variable.data_type.to_string(),
        })
      }
      Expression::Assign(assign) => {
        json!({
          "type": "Assign",
          "name": assign.name.span.literal,
          "value": assign.value.to_json(),
          "data_type": assign.data_type.to_string(),
        })
      }
      Expression::Logical(logical) => {
        json!({
          "type": "Logical",
          "left": logical.left.to_json(),
          "operator": logical.operator.span.literal,
          "right": logical.right.to_json(),
          "data_type": logical.data_type.to_string(),
        })
      }
      Expression::Ternary(ternary) => {
        json!({
          "type": "Ternary",
          "condition": ternary.condition.to_json(),
          "then_branch": ternary.then_branch.to_json(),
          "else_branch": ternary.else_branch.to_json(),
          "data_type": ternary.data_type.to_string(),
        })
      }
      Expression::Call(call) => {
        json!({
          "type": "Call",
          "callee": call.callee.to_json(),
          "arguments": call.arguments.iter().map(|x| x.to_json()).collect::<Vec<serde_json::Value>>(),
          "return_type": call.return_type.to_string(),
          "is_constructor": call.is_constructor,
        })
      }
      Expression::Array(array) => {
        json!({
          "type": "Array",
          "elements": array.elements.iter().map(|x| x.to_json()).collect::<Vec<serde_json::Value>>(),
          "data_type": array.data_type.to_string(),
        })
      }
      Expression::Get(get) => {
        json!({
          "type": "Get",
          "object": get.object.to_json(),
          "name": get.name.span.literal,
        })
      }
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
      Expression::Array(array) => {
        format!(
          "[{}]",
          array
            .elements
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ")
        )
      }
      Expression::Get(get) => {
        format!("{}.{}", get.object.to_string(), get.name.span.literal)
      }
    }
  }
}
