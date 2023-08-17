use crate::ast::expression::Expression;

use super::Statement;

#[derive(Debug)]
pub struct IfStatement {
	pub condition: Box<Expression>,
	pub then_branch: Box<Statement>,
	pub else_branch: Option<Box<Statement>>,
}

impl IfStatement {
	pub fn new(
		condition: Box<Expression>,
		then_branch: Box<Statement>,
		else_branch: Option<Box<Statement>>,
	) -> Self {
		Self {
			condition,
			then_branch,
			else_branch,
		}
	}
}
