pub mod function;
pub mod print;

use std::fmt::Debug;

use super::{
  data_type::DataType,
  evaluator::{EvaluatorValue, EvaluatorResult, Evaluator},
};

pub trait Callable: Debug {
  fn arity(&self) -> usize;
  fn call(
    &self,
    arguments: Vec<EvaluatorValue>,
    evaluator: &mut Box<Evaluator>,
  ) -> EvaluatorResult<EvaluatorValue>;
  fn get_type(&self) -> Option<DataType>;
  fn clone_box(&self) -> Box<dyn Callable>;
}
