pub mod expression;
pub mod lexer;
pub mod parser;
pub mod statement;
pub mod visitor;
pub mod evaluator;
pub mod data_type;

use crate::ast::evaluator::EvaluatorValue;

use self::{statement::Statement, visitor::Visitor};

#[derive(Debug)]
pub struct Ast {
	pub statementes: Vec<Statement>,
}

impl Ast {
	pub fn new()-> Self {
    Self { statementes: Vec::new() }
  }
  
  pub fn add(&mut self, statement: Statement)  {
    self.statementes.push(statement);
  }
  
  pub fn visit(&mut self, visitor: &mut dyn Visitor<EvaluatorValue>) {
    for statement in &self.statementes {
      match statement {
         Statement::Expression(expression) => {
          let value = visitor.visit_expression_statement(expression);
          match value {
            EvaluatorValue::Int(v) => println!("{}", v),
            EvaluatorValue::Double(v) => println!("{}", v),
            EvaluatorValue::Boolean(v) => println!("{}", v),
            EvaluatorValue::String(v) => println!("{}", v),
            _ => ()
          }
         }
         _ => ()
        //  Statement::Variable(_) => visitor.visit_variable_statement(statement)
      };
    }
  }
}
