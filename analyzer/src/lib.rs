pub mod analyzer_error;
pub mod analyzer_value;
pub mod debug;
pub mod ir;

use analyzer_error::AnalyzerDiagnosticError;
use analyzer_value::AnalyzerValue;
use ast::{
  visitor::Visitor,
  expression::{
    binary::Binary,
    Expression,
    literal::Literal,
    unary::Unary,
    grouping::Grouping,
    logical::Logical,
    assign::Assign,
    variable::VariableExpression,
    ternary::Ternary,
    call::{Call, self},
  },
  statement::{
    Statement, expression::ExpressionStatement, block::Block, variable::Variable,
    if_statement::IfStatement, while_statement::WhileStatement, function::FunctionStatement,
    return_statement::Return,
  },
};
use enums::data_type::{DataType, self};
use ir::{
  instruction::{
    IRInstruction, binary::IRBinary, logical::IRLogical, literal::IRLiteral, unary::IRUnary,
    function::IRFunction, variable::IRVariable, block::IRBlock, assign::IRAssign,
    ternary::IRTernary, call::IRCall, ir_if::IRIf, ir_while::IRWhile, ir_return::IRReturn,
  },
  instruction_type::IRInstructionType,
};

pub type AnalyzerResult = Result<IRInstruction, AnalyzerDiagnosticError>;
type CheckCompatibility<T> = (bool, T);

pub struct Analyzer {
  pub irs: Vec<IRInstruction>,
  pub block_stack: Vec<IRBlock>,
  pub diagnostics: Vec<AnalyzerDiagnosticError>,
}

impl Visitor<AnalyzerResult> for Analyzer {
  fn visit_binary_expression(&mut self, expression: &Binary) -> AnalyzerResult {
    let left = self.analyzer(&*expression.left)?;
    let right = self.analyzer(&*expression.right)?;
    let operator = expression.operator.clone();
    let instruction_type = IRInstructionType::from_token_kind(&operator.kind);

    let (result, data_type) = self.are_types_compatible(&left, &right, &instruction_type);
    let left_type = self.extract_data_type(&left);
    let right_type = self.extract_data_type(&right);

    if !result {
      return Err(AnalyzerDiagnosticError::TypeMismatch(
        left_type,
        right_type,
        operator.clone(),
      ));
    }

    let instruction = IRInstruction::Binary(IRBinary::new(
      instruction_type,
      Box::new(left),
      Box::new(right),
      data_type,
    ));

    Ok(instruction)
  }

  fn visit_grouping_expression(&mut self, expression: &Grouping) -> AnalyzerResult {
    self.analyzer(&expression.expression)
  }

  fn visit_literal_expression(&mut self, expression: &Literal) -> AnalyzerResult {
    let instruction = IRInstruction::Literal(IRLiteral::new(AnalyzerValue::from_literation_value(
      expression.value.clone(),
    )));

    Ok(instruction)
  }

  fn visit_unary_expression(&mut self, expression: &Unary) -> AnalyzerResult {
    let right = self.analyzer(&*expression.right)?;
    let instruction_type = IRInstructionType::from_token_kind(&expression.operator.kind);

    if !self.are_types_unary_compatible(&right, &instruction_type) {
      let right_type = self.extract_data_type(&right);
      return Err(AnalyzerDiagnosticError::TypeMismatchUnary(
        right_type,
        expression.operator.clone(),
      ));
    }

    let instruction = IRInstruction::Unary(IRUnary::new(
      instruction_type,
      Box::new(right),
      DataType::Int,
    ));

    Ok(instruction)
  }

  fn visit_variable_expression(&mut self, variable: &VariableExpression) -> AnalyzerResult {
    let irs = &self.irs;
    let is_function = irs.into_iter().find(|ir| match ir {
      IRInstruction::Function(f) => f.name == variable.name.span.literal,
      _ => false,
    });

    if is_function.is_some() {
      let function = is_function.unwrap();

      let instruction = IRInstruction::Function(match function {
        IRInstruction::Function(f) => f.clone(),
        _ => unreachable!(),
      });

      return Ok(instruction);
    }

    let current_block = &self.block_stack;

    let env = current_block.into_iter().find(|block| {
      block
        .scopes_variables
        .iter()
        .any(|var| var.name == variable.name.span.literal)
    });

    if let Some(block) = env {
      let variable = block
        .scopes_variables
        .iter()
        .find(|var| var.name == variable.name.span.literal)
        .unwrap();

      let instruction = IRInstruction::Variable(variable.clone());

      Ok(instruction)
    } else {
      Err(AnalyzerDiagnosticError::UndeclaredVariable(
        variable.clone(),
      ))
    }
  }

  fn visit_assign_expression(&mut self, expression: &Assign) -> AnalyzerResult {
    let value = self.analyzer(&expression.value)?;
    let current_block = &self.block_stack;

    let env = current_block.into_iter().find(|block| {
      block
        .scopes_variables
        .iter()
        .any(|var| var.name == expression.name.span.literal)
    });

    if let Some(block) = env {
      let variable = block
        .scopes_variables
        .iter()
        .find(|var| var.name == expression.name.span.literal)
        .unwrap();

      if variable.is_mutable {
        let instruction = IRInstruction::Assign(IRAssign::new(
          expression.name.span.literal.clone(),
          Box::new(value),
        ));

        Ok(instruction)
      } else {
        Err(AnalyzerDiagnosticError::InvalidReassignedVariable(
          expression.name.span.clone(),
        ))
      }
    } else {
      Err(AnalyzerDiagnosticError::UndefinedVariable(
        expression.name.clone(),
      ))
    }
  }

  fn visit_logical_expression(&mut self, expression: &Logical) -> AnalyzerResult {
    let left = self.analyzer(&expression.left)?;
    let right = self.analyzer(&expression.right)?;

    let instruction_type = IRInstructionType::from_token_kind(&expression.operator.kind);

    match instruction_type {
      IRInstructionType::And | IRInstructionType::Or => {
        if !self.are_types_logical_compatibel(&left, &right, &instruction_type) {
          return Err(AnalyzerDiagnosticError::TypeMismatch(
            self.extract_data_type(&left),
            self.extract_data_type(&right),
            expression.operator.clone(),
          ));
        }

        let instruction = IRInstruction::Logical(IRLogical::new(
          instruction_type,
          Box::new(left),
          Box::new(right),
        ));

        Ok(instruction)
      }
      _ => Err(AnalyzerDiagnosticError::InvalidOperator(
        expression.operator.clone(),
      )),
    }
  }

  fn visit_ternary_expression(&mut self, expression: &Ternary) -> AnalyzerResult {
    let condition = self.analyzer(&*expression.condition)?;
    let then_branch = self.analyzer(&*expression.then_branch)?;
    let else_branch = self.analyzer(&*expression.else_branch)?;

    let instruction = IRInstruction::Ternary(IRTernary::new(
      Box::new(condition),
      Box::new(then_branch),
      Box::new(else_branch),
    ));

    Ok(instruction)
  }

  fn visit_call_expression(&mut self, expression: &Call) -> AnalyzerResult {
    let calle = self.analyzer(&expression.callee)?;

    let function = match calle {
      IRInstruction::Function(f) => Some(f),
      _ => {
        return Err(AnalyzerDiagnosticError::NotCallable(
          expression.paren.clone(),
        ))
      }
    };

    let function = function.unwrap();

    if function.parameters.len() != expression.arguments.len() {
      return Err(AnalyzerDiagnosticError::InvalidNumberOfArguments(
        function.parameters.len(),
        expression.arguments.len(),
        expression.paren.clone(),
      ));
    }

    let mut arguments = Vec::<IRInstruction>::new();

    for (i, arg) in expression.arguments.iter().enumerate() {
      let arg_type = self.analyzer(&arg)?;

      let kind = match arg_type {
        IRInstruction::Literal(l) => l.value.to_data_type(),
        IRInstruction::Variable(v) => v.data_type,
        IRInstruction::Function(f) => f.return_type,
        _ => DataType::None,
      };

      if kind != function.parameters[i].data_type
        && function.parameters[i].data_type != DataType::None
      {
        return Err(AnalyzerDiagnosticError::ArgumentTypeMismatch(
          function.parameters[i].data_type.clone(),
          kind,
          expression.paren.clone(),
        ));
      }
    }

    let instruction = IRInstruction::Call(IRCall::new(
      Box::new(IRInstruction::Function(function)),
      arguments,
    ));

    Ok(instruction)
  }

  fn visit_expression_statement(&mut self, statement: &ExpressionStatement) -> AnalyzerResult {
    self.analyzer(&statement.expression)
  }

  fn visit_variable_statement(&mut self, variable: &Variable) -> AnalyzerResult {
    let mut value = IRInstruction::Literal(IRLiteral::new(AnalyzerValue::Null));
    let data_type = variable.type_annotation.clone();

    if let Some(initializer) = &variable.initializer {
      let expression = self.analyzer(&initializer)?;
      match expression {
        IRInstruction::Literal(literal) => {
          value = IRInstruction::Literal(literal);
        }
        IRInstruction::Binary(binary) => {
          value = IRInstruction::Binary(binary);
        }
        IRInstruction::Unary(unary) => {
          value = IRInstruction::Unary(unary);
        }
        IRInstruction::Variable(variable) => {
          value = IRInstruction::Variable(variable);
        }
        _ => (),
      }
    }

    let current_block = self.block_stack.last_mut();

    if current_block.is_none() {
      self.block_stack = vec![IRBlock::new(Vec::new(), Vec::new())];
    }

    let current_block = self.block_stack.last_mut().unwrap();

    current_block.scopes_variables.push(IRVariable::new(
      variable.name.span.literal.clone(),
      data_type.clone(),
      variable.is_mutable,
      false,
      Some(Box::new(value.clone())),
    ));

    Ok(IRInstruction::Variable(IRVariable::new(
      variable.name.span.literal.clone(),
      data_type,
      false,
      false,
      Some(Box::new(value)),
    )))
  }

  fn visit_block(&mut self, block: &Block) -> AnalyzerResult {
    self.block_stack.push(IRBlock::new(Vec::new(), Vec::new()));

    for statement in &block.statements {
      self.analyze_statement(statement)?;
    }

    let block = self.block_stack.pop().unwrap();

    Ok(IRInstruction::Block(IRBlock::new(
      self.irs.clone(),
      block.scopes_variables,
    )))
  }

  fn visit_if_statement(&mut self, statement: &IfStatement) -> AnalyzerResult {
    let condition = self.analyzer(&statement.condition)?;
    let then_branch = self.analyze_statement(&statement.then_branch)?;

    let else_branch: Option<Box<IRInstruction>> = if statement.else_branch.is_some() {
      Some(Box::new(
        self.analyze_statement(statement.else_branch.as_ref().unwrap())?,
      ))
    } else {
      None
    };

    let instruction = IRInstruction::If(IRIf::new(
      Box::new(condition),
      Box::new(then_branch),
      else_branch,
    ));

    Ok(instruction)
  }

  fn visit_while_statement(&mut self, statement: &WhileStatement) -> AnalyzerResult {
    let condition = self.analyzer(&statement.condition)?;
    let body = self.analyze_statement(&statement.body)?;

    let instruction = IRInstruction::While(IRWhile::new(Box::new(condition), Box::new(body)));

    Ok(instruction)
  }

  fn visit_function_statement(&mut self, statement: &FunctionStatement) -> AnalyzerResult {
    let mut parameters = Vec::<IRVariable>::new();

    for param in &statement.parameters {
      parameters.push(IRVariable::new(
        param.name.span.literal.clone(),
        param.data_type.clone(),
        false,
        false,
        None,
      ));
    }

    let previus_block = self.block_stack.clone();

    self
      .block_stack
      .push(IRBlock::new(Vec::new(), parameters.clone()));

    let mut current_block = self.block_stack.last().unwrap().clone();

    for body in &statement.body {
      let result = self.analyze_statement(body)?;

      match result {
        IRInstruction::Variable(v) => {
          current_block.scopes_variables.push(v);
        }
        _ => {
          current_block.instructions.push(result);
        }
      };
    }

    self.block_stack.pop();

    self.block_stack = previus_block.clone();

    let instruction = IRInstruction::Function(IRFunction::new(
      statement.name.span.literal.clone(),
      parameters,
      statement.return_type.clone().unwrap_or(DataType::Void),
      Some(Box::new(current_block.clone())),
    ));

    Ok(instruction)
  }

  fn visit_return_statement(&mut self, statement: &Return) -> AnalyzerResult {
    let value = &statement.value;
    if value.is_none() {
      let instruction = IRInstruction::Return(IRReturn::new(Box::new(IRInstruction::Literal(
        IRLiteral::new(AnalyzerValue::Null),
      ))));

      return Ok(instruction);
    }

    let value = self.analyzer(&value.as_ref().unwrap())?;

    let instruction = IRInstruction::Return(IRReturn::new(Box::new(value)));

    Ok(instruction)
  }
}

impl Analyzer {
  pub fn new() -> Self {
    let mut irs = Vec::<IRInstruction>::new();
    irs.push(IRInstruction::Function(IRFunction::new(
      "println".to_string(),
      vec![IRVariable::new(
        "message".to_string(),
        DataType::None,
        false,
        false,
        None,
      )],
      DataType::Void,
      None,
    )));

    Self {
      irs,
      diagnostics: Vec::new(),
      block_stack: Vec::new(),
    }
  }

  pub fn analyze(&mut self, statements: &Vec<Statement>) {
    for statement in statements {
      match self.analyze_statement(statement) {
        Ok(ir) => {
          self.irs.push(ir);
        }
        Err(e) => self.diagnostics.push(e),
      }
    }
  }

  fn analyzer(&mut self, expression: &Expression) -> AnalyzerResult {
    expression.accept(self)
  }

  fn analyze_statement(&mut self, statement: &Statement) -> AnalyzerResult {
    statement.accept(self)
  }

  fn find_function_in_ir(&self, name: String) -> Option<IRFunction> {
    let irs = &self.irs;

    let function = irs.into_iter().find(|ir| match ir {
      IRInstruction::Function(f) => f.name == name,
      _ => false,
    });

    match function {
      Some(IRInstruction::Function(f)) => Some(f.clone()),
      _ => None,
    }
  }

  fn are_types_logical_compatibel(
    &self,
    left: &IRInstruction,
    right: &IRInstruction,
    operator: &IRInstructionType,
  ) -> bool {
    match operator {
      IRInstructionType::And | IRInstructionType::Or => match (left, right) {
        (
          IRInstruction::Literal(IRLiteral {
            value: AnalyzerValue::Boolean(_),
          }),
          IRInstruction::Literal(IRLiteral {
            value: AnalyzerValue::Boolean(_),
          }),
        ) => true,
        _ => false,
      },
      _ => false,
    }
  }

  fn are_types_unary_compatible(
    &self,
    right: &IRInstruction,
    operator: &IRInstructionType,
  ) -> bool {
    match operator {
      IRInstructionType::Sub => match right {
        IRInstruction::Literal(IRLiteral {
          value: AnalyzerValue::Int(_),
        })
        | IRInstruction::Literal(IRLiteral {
          value: AnalyzerValue::Double(_),
        }) => true,
        _ => false,
      },
      IRInstructionType::Not => match right {
        IRInstruction::Literal(IRLiteral {
          value: AnalyzerValue::Boolean(_),
        })
        | IRInstruction::Literal(IRLiteral {
          value: AnalyzerValue::Int(_),
        })
        | IRInstruction::Literal(IRLiteral {
          value: AnalyzerValue::String(_),
        })
        | IRInstruction::Literal(IRLiteral {
          value: AnalyzerValue::Double(_),
        })
        | IRInstruction::Literal(IRLiteral {
          value: AnalyzerValue::Null,
        }) => true,
        _ => false,
      },
      _ => false,
    }
  }

  fn extract_data_type(&self, instruction: &IRInstruction) -> DataType {
    match instruction {
      IRInstruction::Literal(l) => l.value.to_data_type(),
      IRInstruction::Variable(v) => v.data_type.clone(),
      IRInstruction::Function(f) => f.return_type.clone(),
      IRInstruction::Binary(b) => b.data_type.clone(),
      IRInstruction::Unary(u) => u.data_type.clone(),
      IRInstruction::Logical(_) => DataType::Boolean,
      IRInstruction::Assign(a) => self.extract_data_type(&*a.value.clone()),
      IRInstruction::Call(c) => self.extract_data_type(&*c.callee.clone()),
      _ => DataType::None,
    }
  }

  fn check_add_compatibility(
    &self,
    left: &DataType,
    right: &DataType,
  ) -> CheckCompatibility<DataType> {
    match (left, right) {
      (DataType::Int, DataType::Int) => (true, DataType::Int),
      (DataType::Double, DataType::Double) => (true, DataType::Double),
      (DataType::String, DataType::String) => (true, DataType::String),
      (_, DataType::Null) => (true, left.clone()),
      (DataType::Null, _) => (true, right.clone()),
      _ => (false, DataType::None),
    }
  }

  fn check_arithmetic_compatibility(
    &self,
    left: &DataType,
    right: &DataType,
  ) -> CheckCompatibility<DataType> {
    match (left, right) {
      (DataType::Int, DataType::Int) => (true, DataType::Int),
      (DataType::Double, DataType::Double) => (true, DataType::Double),
      (DataType::Int, DataType::Double) => (true, DataType::Double),
      (DataType::Double, DataType::Int) => (true, DataType::Double),
      (_, DataType::Null) => (true, left.clone()),
      (DataType::Null, _) => (true, right.clone()),
      _ => (false, DataType::None),
    }
  }

  fn check_comparation_compatibility(
    &self,
    left: &DataType,
    right: &DataType,
  ) -> CheckCompatibility<DataType> {
    match (left, right) {
      (DataType::Int, DataType::Int) => (true, DataType::Boolean),
      (DataType::Double, DataType::Double) => (true, DataType::Boolean),
      (DataType::Int, DataType::Double) => (true, DataType::Boolean),
      (DataType::Double, DataType::Int) => (true, DataType::Boolean),
      (_, DataType::Null) => (true, left.clone()),
      (DataType::Null, _) => (true, right.clone()),
      _ => (false, DataType::None),
    }
  }

  fn check_equal_compatibility(
    &self,
    left: &DataType,
    right: &DataType,
  ) -> CheckCompatibility<DataType> {
    match (left, right) {
      (DataType::Int, DataType::Int) => (true, DataType::Boolean),
      (DataType::Double, DataType::Double) => (true, DataType::Boolean),
      (DataType::String, DataType::String) => (true, DataType::Boolean),
      (DataType::Boolean, DataType::Boolean) => (true, DataType::Boolean),
      (_, DataType::Null) => (true, left.clone()),
      (DataType::Null, _) => (true, right.clone()),
      _ => (false, DataType::None),
    }
  }

  fn check_logical_compatibility(
    &self,
    left: &DataType,
    right: &DataType,
  ) -> CheckCompatibility<DataType> {
    match (left, right) {
      (DataType::Boolean, DataType::Boolean) => (true, DataType::Boolean),
      _ => (false, DataType::None),
    }
  }

  fn are_types_compatible(
    &self,
    left: &IRInstruction,
    right: &IRInstruction,
    operator: &IRInstructionType,
  ) -> CheckCompatibility<DataType> {
    let left_type = self.extract_data_type(left);
    let right_type = self.extract_data_type(right);

    match operator {
      IRInstructionType::Add => self.check_add_compatibility(&left_type, &right_type),
      IRInstructionType::Sub | IRInstructionType::Mul | IRInstructionType::Div => {
        self.check_arithmetic_compatibility(&left_type, &right_type)
      }
      IRInstructionType::GreaterEqual
      | IRInstructionType::Greater
      | IRInstructionType::LessEqual
      | IRInstructionType::Less => self.check_comparation_compatibility(&left_type, &right_type),
      IRInstructionType::Equal | IRInstructionType::NotEqual => {
        self.check_equal_compatibility(&left_type, &right_type)
      }
      IRInstructionType::And | IRInstructionType::Or => {
        self.check_logical_compatibility(&left_type, &right_type)
      }
      _ => (false, DataType::None),
    }
  }
}
