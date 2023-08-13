pub mod data_type;
pub mod environment;
pub mod evaluator;
pub mod expression;
pub mod lexer;
pub mod parser;
pub mod statement;
pub mod visitor;

use crate::ast::evaluator::EvaluatorValue;

use self::{
  statement::{Statement, variable},
  visitor::Visitor,
};

#[derive(Debug)]
pub struct Ast {
  pub statementes: Vec<Statement>,
}

impl Ast {
  pub fn new() -> Self {
    Self {
      statementes: Vec::new(),
    }
  }

  pub fn add(&mut self, statement: Statement) {
    self.statementes.push(statement);
  }

  pub fn visit(&mut self, visitor: &mut dyn Visitor<Result<EvaluatorValue, ()>>) {
    for statement in &self.statementes {
      match statement {
        Statement::Expression(expression) => {
          let value;

          match visitor.visit_expression_statement(expression) {
            Ok(e) => value = e,
            Err(_) => return,
          };

          match value {
            EvaluatorValue::Int(v) => println!("{}", v),
            EvaluatorValue::Double(v) => println!("{}", v),
            EvaluatorValue::Boolean(v) => println!("{}", v),
            EvaluatorValue::String(v) => println!("{}", v),
            _ => (),
          }
        }
        Statement::Variable(variable) => {
          let value;

          match visitor.visit_variable_statement(variable) {
            Ok(v) => value = v,
            Err(_) => return,
          };

          match value {
            EvaluatorValue::Int(v) => println!("{}", v),
            EvaluatorValue::Double(v) => println!("{}", v),
            EvaluatorValue::Boolean(v) => println!("{}", v),
            EvaluatorValue::String(v) => println!("{}", v),
            _ => (),
          }
        }
        _ => (),
      };
    }
  }
}
