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
  statement::Statement,
  visitor::Visitor,
  evaluator::EvaluatorResult,
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

  pub fn visit(&mut self, visitor: &mut dyn Visitor<EvaluatorResult>) {
    for statement in &self.statementes {
      let mut value: EvaluatorValue = EvaluatorValue::None;

      match statement {
        Statement::Expression(expression) => {
          match visitor.visit_expression_statement(expression) {
            EvaluatorResult::Error => return (),
            EvaluatorResult::Value(v) => value = v,
          };
        }
        Statement::Block(block) => match visitor.visit_block(block) {
          EvaluatorResult::Error => return (),
          EvaluatorResult::Value(s) => {
            value = s;
          }
        },
        Statement::Variable(variable) => {
          match visitor.visit_variable_statement(variable) {
            EvaluatorResult::Error => return (),
            EvaluatorResult::Value(v) => value = v,
          };
        }
        Statement::IfStatement(if_statement) => {
          match visitor.visit_if_statement(if_statement) {
            EvaluatorResult::Error => return (),
            EvaluatorResult::Value(v) => value = v,
          };
        }
        Statement::WhileStatement(while_statement) => {
          match visitor.visit_while_statement(while_statement) {
            EvaluatorResult::Error => return (),
            EvaluatorResult::Value(v) => value = v,
          };
        }
      };
      
      match value {
        EvaluatorValue::Int(v) => println!("{}", v),
        EvaluatorValue::Double(v) => println!("{}", v),
        EvaluatorValue::Boolean(v) => println!("{}", v),
        EvaluatorValue::String(v) => println!("{}", v),
        _ => (),
      }
    }
  }
}
