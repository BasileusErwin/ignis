pub mod analyzer_error;
pub mod analyzer_value;
pub mod ir;

use std::{ collections::HashMap,
  vec, fs,
  fmt::{Display, Formatter},
};

use analyzer_error::AnalyzerDiagnosticError;
use analyzer_value::AnalyzerValue;
use ast::{
  visitor::Visitor,
  expression::{
    binary::Binary, Expression, literal::Literal, unary::Unary, grouping::Grouping,
    logical::Logical, assign::Assign, variable::VariableExpression, ternary::Ternary, call::Call,
    array::Array, new::NewExpression, get::Get,
  },
  statement::{
    Statement,
    expression::ExpressionStatement,
    block::Block,
    variable::Variable,
    if_statement::IfStatement,
    while_statement::WhileStatement,
    function::{FunctionStatement, FunctionDecorator},
    return_statement::Return,
    class::Class,
    for_in::ForIn,
    import::Import,
    break_statement::BreakStatement,
    continue_statement::Continue,
    method::MethodStatement,
    property::PropertyStatement,
  },
};
use enums::{data_type::DataType, token_type::TokenType};
use ir::{
  instruction::{
    IRInstruction,
    binary::IRBinary,
    logical::IRLogical,
    literal::IRLiteral,
    unary::IRUnary,
    function::{IRFunction, IRFunctionMetadata},
    variable::{IRVariable, IRVariableMetadata},
    block::IRBlock,
    assign::IRAssign,
    ternary::IRTernary,
    call::IRCall,
    ir_if::IRIf,
    ir_while::IRWhile,
    ir_return::IRReturn,
    ir_for_in::IRForIn,
    ir_array::IRArray,
    import::IRImport,
    ir_break::IRBreak,
    ir_continue::IRContinue,
    ir_get::IRGet,
    class::IRClass, class_instance::IRClassInstance,
  },
  instruction_type::IRInstructionType,
};
use lexer::{Lexer, token::Token};
use parser::Parser;

pub type AnalyzerResult = Result<IRInstruction, AnalyzerDiagnosticError>;
type CheckCompatibility<T> = (bool, T);

#[derive(Debug, Clone, PartialEq)]
enum AnalyzerContext {
  Function,
  Method,
  Class,
  Loop,
  Switch,
  Match,
}

impl Display for AnalyzerContext {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      AnalyzerContext::Function => write!(f, "Function"),
      AnalyzerContext::Method => write!(f, "Method"),
      AnalyzerContext::Class => write!(f, "Class"),
      AnalyzerContext::Loop => write!(f, "Loop"),
      AnalyzerContext::Switch => write!(f, "Switch"),
      AnalyzerContext::Match => write!(f, "Match"),
    }
  }
}

pub struct Analyzer {
  pub irs: HashMap<String, Vec<IRInstruction>>,
  pub block_stack: Vec<HashMap<String, bool>>,
  pub diagnostics: Vec<AnalyzerDiagnosticError>,
  pub scopes_variables: Vec<IRVariable>,
  pub current_function: Option<IRFunction>,
  pub current_file: String,
  pub current_class: Option<IRClass>,
  context: Vec<AnalyzerContext>,
}

impl Visitor<AnalyzerResult> for Analyzer {
  fn visit_binary_expression(&mut self, expression: &Binary) -> AnalyzerResult {
    let left = self.analyzer(&expression.left)?;
    let right = self.analyzer(&expression.right)?;
    let operator = expression.operator.clone();
    let instruction_type = if operator.kind == TokenType::Plus {
      if self.extract_data_type(&left) == DataType::String
        && self.extract_data_type(&right) == DataType::String
      {
        IRInstructionType::Concatenate
      } else {
        IRInstructionType::Add
      }
    } else {
      IRInstructionType::from_token_kind(&operator.kind)
    };

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
    let right = self.analyzer(&expression.right)?;
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
    if self.block_stack.is_empty() {
      return Err(AnalyzerDiagnosticError::UndeclaredVariable(
        variable.clone(),
      ));
    }

    let irs = &self.irs.get(&self.current_file).unwrap();
    let is_function = irs.iter().find(|ir| match ir {
      IRInstruction::Function(f) => f.name == variable.name.span.literal,
      _ => false,
    });

    if let Some(IRInstruction::Function(f)) = is_function {
      let function = f.clone();

      let instruction = IRInstruction::Function(function);

      return Ok(instruction);
    }

    if let Some(f) = &mut self.current_function {
      if f.name == variable.name.span.literal {
        f.metadata.is_recursive = true;

        let instruction = IRInstruction::Function(f.clone());

        return Ok(instruction);
      }
    }

    let env = self.block_stack.last();

    if let Some(block) = env {
      if block.get(&variable.name.span.literal).is_none() {
        return Err(AnalyzerDiagnosticError::UndeclaredVariable(
          variable.clone(),
        ));
      }

      let is_declared = *block.get(variable.name.span.literal.as_str()).unwrap();

      if !is_declared {
        return Err(AnalyzerDiagnosticError::UndeclaredVariable(
          variable.clone(),
        ));
      }

      let mut variable = self
        .scopes_variables
        .iter()
        .find(|v| v.name == variable.name.span.literal)
        .unwrap()
        .clone();

      variable.metadata.is_declaration = false;

      let instruction = IRInstruction::Variable(variable.clone());

      return Ok(instruction);
    }

    Err(AnalyzerDiagnosticError::UndeclaredVariable(
      variable.clone(),
    ))
  }

  fn visit_assign_expression(&mut self, expression: &Assign) -> AnalyzerResult {
    if self.block_stack.is_empty()
      || self
        .block_stack
        .last()
        .unwrap()
        .get(&expression.name.span.literal)
        .is_none()
    {
      return Err(AnalyzerDiagnosticError::UndefinedVariable(
        expression.name.clone(),
      ));
    }

    let value = self.analyzer(&expression.value)?;
    let current_block = self.block_stack.last().unwrap();

    let env = current_block.iter().find(|(name, is_declared)| {
      name.as_str() == expression.name.span.literal.as_str() && **is_declared
    });

    if let Some((name, _)) = env {
      let variable = self
        .scopes_variables
        .iter()
        .find(|v| v.name == *name)
        .unwrap();

      if variable.metadata.is_mutable {
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
    let condition = self.analyzer(&expression.condition)?;
    let then_branch = self.analyzer(&expression.then_branch)?;
    let else_branch = self.analyzer(&expression.else_branch)?;

    if self.extract_data_type(&condition) != DataType::Boolean {
      return Err(AnalyzerDiagnosticError::InvalidCondition(
        *expression.token.clone(),
      ));
    }

    let then_type = self.extract_data_type(&then_branch);
    let else_type = self.extract_data_type(&else_branch);

    if then_type != else_type {
      return Err(AnalyzerDiagnosticError::TypeMismatch(
        then_type,
        else_type,
        *expression.token.clone(),
      ));
    }

    Ok(IRInstruction::Ternary(IRTernary::new(
      Box::new(condition),
      Box::new(then_branch),
      Box::new(else_branch),
      then_type,
    )))
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
      let arg_type = self.analyzer(arg)?;

      let kind = match &arg_type {
        IRInstruction::Literal(l) => l.value.to_data_type(),
        IRInstruction::Variable(v) => v.data_type.clone(),
        IRInstruction::Function(f) => f.return_type.clone(),
        IRInstruction::Call(c) => c.return_type.clone(),
        IRInstruction::Return(r) => r.data_type.clone(),
        IRInstruction::Binary(b) => b.data_type.clone(),
        IRInstruction::Unary(u) => u.data_type.clone(),
        IRInstruction::Logical(_) => DataType::Boolean,
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

      if let IRInstruction::Variable(v) = &arg_type {
        if !v.metadata.is_mutable && function.parameters[i].metadata.is_mutable {
          return Err(
            AnalyzerDiagnosticError::ImmutableVariableAsMutableParameter(
              function.parameters[i].name.clone(),
              v.name.clone(),
              expression.paren.clone(),
            ),
          );
        }
      };

      arguments.push(arg_type);
    }

    let instruction = IRInstruction::Call(IRCall::new(
      function.name,
      arguments,
      function.return_type,
    ));

    Ok(instruction)
  }

  fn visit_expression_statement(&mut self, statement: &ExpressionStatement) -> AnalyzerResult {
    self.analyzer(&statement.expression)
  }

  fn visit_variable_statement(&mut self, variable: &Variable) -> AnalyzerResult {
    self.declare(variable.name.span.literal.clone());

    let mut value = IRInstruction::Literal(IRLiteral::new(AnalyzerValue::Null));
    let data_type = variable.type_annotation.clone();

    if let Some(initializer) = &variable.initializer {
      let expression = self.analyzer(initializer)?;
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
        IRInstruction::Ternary(ternary) => {
          value = IRInstruction::Ternary(ternary);
        }
        IRInstruction::Call(call) => {
          value = IRInstruction::Call(call);
        }
        IRInstruction::Class(class) => {
          value = IRInstruction::Class(class);
        }
        IRInstruction::Logical(logical) => {
          value = IRInstruction::Logical(logical);
        }
        IRInstruction::Array(array) => {
          value = IRInstruction::Array(array);
        }
        IRInstruction::ClassInstance(class) => {
          value = IRInstruction::ClassInstance(class);
        }
        _ => (),
      }
    }

    let variable = IRVariable::new(
      variable.name.span.literal.clone(),
      data_type.clone(),
      Some(Box::new(value.clone())),
      IRVariableMetadata::new(
        variable.metadata.is_mutable,
        variable.metadata.is_reference,
        false,
        false,
        false,
        true,
        false,
        false,
        false,
      ),
    );

    self.define(variable.name.clone());

    self.scopes_variables.push(variable.clone());

    Ok(IRInstruction::Variable(variable.clone()))
  }

  fn visit_block(&mut self, block: &Block) -> AnalyzerResult {
    let scopes_variables = self.scopes_variables.clone();

    self.begin_scope();

    let mut ir_block = IRBlock::new(Vec::new(), Vec::new());

    for statement in &block.statements {
      let result = self.analyze_statement(statement)?;
      ir_block.instructions.push(result);
    }

    self.end_scope();

    self.scopes_variables = scopes_variables;

    Ok(IRInstruction::Block(ir_block))
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
    self.context.push(AnalyzerContext::Loop);
    let condition = self.analyzer(&statement.condition)?;
    let body = self.analyze_statement(&statement.body)?;

    let instruction = IRInstruction::While(IRWhile::new(Box::new(condition), Box::new(body)));

    self.context.pop();

    Ok(instruction)
  }

  fn visit_function_statement(&mut self, statement: &FunctionStatement) -> AnalyzerResult {
    self.begin_scope();
    self.context.push(AnalyzerContext::Function);
    let mut parameters = Vec::<IRVariable>::new();

    if self.is_allready_declared(&statement.name.span.literal) {
      return Err(AnalyzerDiagnosticError::FunctionAlreadyDefined(
        statement.name.span.literal.clone(),
        statement.name.clone(),
      ));
    }

    self.declare(statement.name.span.literal.clone());
    self.define(statement.name.span.literal.clone());

    for param in &statement.parameters {
      self.define_parameter(param.name.span.literal.clone());
      let parameter = IRVariable::new(
        param.name.span.literal.clone(),
        param.data_type.clone(),
        None,
        IRVariableMetadata::new(
          param.is_mutable,
          param.is_reference,
          true,
          false,
          false,
          false,
          false,
          false,
          false,
        ),
      );

      self.scopes_variables.push(parameter.clone());

      parameters.push(parameter);
    }

    let mut ir: IRBlock = IRBlock::new(Vec::new(), Vec::new());

    let is_extern = statement
      .annotations
      .clone()
      .into_iter()
      .any(|a| matches!(&a, FunctionDecorator::Extern(_)));

    let mut current_function = IRFunction::new(
      statement.name.span.literal.clone(),
      parameters.clone(),
      statement.return_type.clone().unwrap_or(DataType::Void),
      None,
      IRFunctionMetadata::new(
        false,
        statement.is_exported,
        false,
        is_extern,
        false,
        false,
        false,
      ),
    );

    self.current_function = Some(current_function.clone());

    for body in &statement.body {
      let result = self.analyze_statement(body)?;

      match result {
        IRInstruction::Variable(v) => {
          self.scopes_variables.push(v.clone());
          ir.scopes_variables.push(v);
        }
        _ => {
          ir.instructions.push(result);
        }
      };
    }

    self.end_scope();

    current_function = self.current_function.as_ref().unwrap().clone();

    current_function.body = Some(Box::new(ir.clone()));

    let instruction = IRInstruction::Function(current_function);

    self.context.pop();
    self.current_function = None;

    Ok(instruction)
  }

  fn visit_return_statement(&mut self, statement: &Return) -> AnalyzerResult {
    if !self.context.iter().any(|context| {
      matches!(
        &context,
        AnalyzerContext::Function | AnalyzerContext::Method
      )
    }) {
      return Err(AnalyzerDiagnosticError::ReturnOutsideFunction(
        *statement.keyword.clone(),
      ));
    }

    let value = &statement.value;
    if value.is_none() {
      let instruction = IRInstruction::Return(IRReturn::new(
        Box::new(IRInstruction::Literal(IRLiteral::new(AnalyzerValue::Null))),
        DataType::Void,
      ));

      return Ok(instruction);
    }

    let value = self.analyzer(value.as_ref().unwrap())?;
    let data_type = self.extract_data_type(&value);

    let instruction = IRInstruction::Return(IRReturn::new(Box::new(value), data_type));

    Ok(instruction)
  }

  fn visit_class_statement(&mut self, statement: &Class) -> AnalyzerResult {
    self.context.push(AnalyzerContext::Class);
    let mut properties = Vec::<IRVariable>::new();

    if self.is_allready_declared(&statement.name.span.literal) {
      return Err(AnalyzerDiagnosticError::ClassAlreadyDefined(
        statement.name.clone(),
      ));
    }

    self.begin_scope();

    self.declare(statement.name.span.literal.clone());
    self.define(statement.name.span.literal.clone());

    for property in &statement.properties {
      let result = self.analyze_statement(property);

      match result {
        Ok(IRInstruction::Variable(v)) => {
          self.scopes_variables.push(v.clone());
          properties.push(v);
        }
        _ => {
          todo!()
        }
      };
    }

    let mut ir: Vec<IRFunction> = Vec::new();

    let mut current_class =
      IRClass::new(statement.name.span.literal.clone(), Vec::new(), properties);

    self.current_class = Some(current_class.clone());

    for method in &statement.methods {
      let result = self.analyze_statement(method)?;

      match result {
        IRInstruction::Function(f) => {
          self.scopes_variables.push(IRVariable::new(
            f.name.clone(),
            DataType::None,
            None,
            IRVariableMetadata::new(
              false,
              false,
              false,
              false,
              false,
              false,
              false,
              false,
              f.metadata.is_constructor,
            ),
          ));

          ir.push(f);
        }
        _ => {
          return Err(AnalyzerDiagnosticError::ClassAlreadyDefined(
            statement.name.clone(),
          ));
        }
      };
    }

    self.end_scope();
    self.declare(statement.name.span.literal.clone());
    self.define(statement.name.span.literal.clone());

    current_class = self.current_class.as_ref().unwrap().clone();

    current_class.methods = ir.clone();

    let instruction = IRInstruction::Class(current_class.clone());

    self.context.pop();
    self.current_class = None;

    Ok(instruction)
  }

  fn visit_array_expression(&mut self, expression: &Array) -> AnalyzerResult {
    let mut elements = Vec::new();
    let mut element_types = Vec::new();

    for elem in &expression.elements {
      let analyzed_elem = self.analyzer(elem)?;
      let elem_type = self.extract_data_type(&analyzed_elem);

      elements.push(analyzed_elem);
      element_types.push(elem_type);
    }

    let first_type = element_types.first().unwrap_or(&DataType::None);

    if !element_types.iter().all(|t| t == first_type) {
      return Err(AnalyzerDiagnosticError::ArrayElementTypeMismatch(
        expression.token.clone(),
      ));
    }

    let instruction = IRInstruction::Array(IRArray::new(
      elements,
      DataType::Array(Box::new(first_type.clone())),
    ));

    Ok(instruction)
  }

  fn visit_for_in_statement(&mut self, statement: &ForIn) -> AnalyzerResult {
    self.declare(statement.variable.name.span.literal.clone());

    let iterable = self.analyzer(&statement.iterable)?;
    let data_type = self.extract_data_type(&iterable);

    if !self.is_iterable(&iterable) {
      return Err(AnalyzerDiagnosticError::NotIterable(
        statement.token.clone(),
      ));
    }

    self.begin_scope();

    self.define(statement.variable.name.span.literal.clone());

    let variable = IRVariable::new(
      statement.variable.name.span.literal.clone(),
      data_type,
      None,
      IRVariableMetadata::new(
        statement.variable.metadata.is_mutable,
        statement.variable.metadata.is_reference,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
      ),
    );

    self.scopes_variables.push(variable.clone());

    let body = self.analyze_statement(&statement.body)?;

    self.end_scope();

    let instruction = IRInstruction::ForIn(IRForIn::new(
      variable,
      Box::new(iterable),
      Box::new(body),
      statement.token.clone(),
    ));

    Ok(instruction)
  }

  fn visit_import_statement(&mut self, statement: &Import) -> AnalyzerResult {
    let mut block_stack: HashMap<String, bool> = self.block_stack.last_mut().unwrap().clone();

    if !statement.is_std {
      self.resolve_module_import(statement, &mut block_stack)?;
    } else {
      self.resolve_std_import(statement.module_path.span.literal.clone(), &mut block_stack);
    }

    Ok(IRInstruction::Import(IRImport::new(
      statement
        .symbols
        .clone()
        .into_iter()
        .map(|i| (i.name, i.alias))
        .collect::<Vec<(Token, Option<Token>)>>(),
      statement.module_path.span.literal.clone(),
    )))
  }

  fn visit_break_statement(&mut self, statement: &BreakStatement) -> AnalyzerResult {
    let is_loop = self
      .context
      .iter()
      .find(|context| matches!(context, AnalyzerContext::Loop | AnalyzerContext::Switch));

    if is_loop.is_none() {
      return Err(AnalyzerDiagnosticError::BreakOutsideLoop(
        statement.token.clone(),
      ));
    }

    Ok(IRInstruction::Break(IRBreak::new(statement.token.clone())))
  }

  fn visit_continue_statement(&mut self, statement: &Continue) -> AnalyzerResult {
    let is_loop = self
      .context
      .iter()
      .find(|context| matches!(context, AnalyzerContext::Loop));

    if is_loop.is_none() {
      return Err(AnalyzerDiagnosticError::ContinueOutsideLoop(
        statement.token.clone(),
      ));
    }

    Ok(IRInstruction::Continue(IRContinue::new(
      statement.token.clone(),
    )))
  }

  fn visit_new_expression(&mut self, expression: &NewExpression) -> AnalyzerResult {
    let class_info = self.find_class_in_ir(expression.name.span.literal.clone());

    if class_info.is_none() {
      return Err(AnalyzerDiagnosticError::UndefinedClass(
        expression.name.clone(),
      ));
    }

    let class = class_info.unwrap();

    if class.methods.is_empty() {
      return Err(AnalyzerDiagnosticError::UndefinedMethods(
        expression.name.clone(),
      ));
    }

    let mut arguments = Vec::<IRInstruction>::new();

    for arg in &expression.arguments {
      let arg_type = self.analyzer(arg)?;

      arguments.push(arg_type);
    }

    let constructor = self.find_matching_constructor(&class, &arguments);

    if constructor.is_none() {
      return Err(AnalyzerDiagnosticError::UndefinedMethods(
        expression.name.clone(),
      ));
    }

    let constructor = constructor.unwrap();

    let instruction = IRInstruction::ClassInstance(IRClassInstance::new(
      Box::new(class),
      constructor.name,
      arguments,
    ));

    Ok(instruction)
  }

  fn visit_get_expression(&mut self, expression: &Get) -> AnalyzerResult {
    let ir_object = self.analyzer(&expression.object)?;

    let object = match ir_object {
      IRInstruction::Variable(c) => c,
      _ => return Err(AnalyzerDiagnosticError::NotAClass(expression.name.clone())),
    };

    let class = self.find_class_in_ir(
      match object.data_type {
        DataType::ClassType(name) => name,
        _ => return Err(AnalyzerDiagnosticError::NotAClass(expression.name.clone())),
      }
    );

    if class.is_none() {
      return Err(AnalyzerDiagnosticError::UndefinedClass(
      expression.name.clone(),
      ));
    }

    let class = class.unwrap();

    if class.properties.is_empty() {
      return Err(AnalyzerDiagnosticError::UndefinedProperty(
        expression.name.clone(),
      ));
    }

    let class_binding = class.clone();

    let property = class_binding
      .properties
      .iter()
      .find(|p| p.name == expression.name.span.literal);

    if property.is_none() {
      return Err(AnalyzerDiagnosticError::UndefinedProperty(
        expression.name.clone(),
      ));
    }

    let instruction = IRInstruction::Get(IRGet::new(
      expression.name.span.literal.clone(),
      Box::new(class),
      self.extract_data_type(&IRInstruction::Variable(property.unwrap().clone())),
    ));

    Ok(instruction)
  }

  fn visit_method_statement(&mut self, statement: &MethodStatement) -> AnalyzerResult {
    if self.is_allready_declared(&statement.name.span.literal) && !statement.metadata.is_contructor
    {
      return Err(AnalyzerDiagnosticError::MethodAlreadyDefined(
        statement.name.clone(),
      ));
    }

    if self
      .context
      .iter()
      .any(|c| !matches!(c, AnalyzerContext::Class))
    {
      return Err(AnalyzerDiagnosticError::MethodOutsideClass(
        statement.name.clone(),
      ));
    }

    self.begin_scope();
    self.context.push(AnalyzerContext::Method);
    let mut parameters = Vec::<IRVariable>::new();

    self.declare(statement.name.span.literal.clone());
    self.define(statement.name.span.literal.clone());

    for param in &statement.parameters {
      self.define_parameter(param.name.span.literal.clone());
      let parameter = IRVariable::new(
        param.name.span.literal.clone(),
        param.data_type.clone(),
        None,
        IRVariableMetadata::new(
          param.is_mutable,
          param.is_reference,
          true,
          false,
          false,
          false,
          false,
          false,
          false,
        ),
      );

      self.scopes_variables.push(parameter.clone());

      parameters.push(parameter);
    }

    let mut ir: IRBlock = IRBlock::new(Vec::new(), Vec::new());

    let mut current_function = IRFunction::new(
      statement.name.span.literal.clone(),
      parameters.clone(),
      statement.return_type.clone().unwrap_or(DataType::Void),
      None,
      IRFunctionMetadata::new(
        false,
        false,
        false,
        false,
        statement.metadata.is_static,
        statement.metadata.is_public,
        statement.metadata.is_contructor,
      ),
    );

    self.current_function = Some(current_function.clone());

    for body in &statement.body {
      let result = self.analyze_statement(body)?;

      match result {
        IRInstruction::Variable(v) => {
          self.scopes_variables.push(v.clone());
          ir.scopes_variables.push(v);
        }
        _ => {
          ir.instructions.push(result);
        }
      };
    }

    self.end_scope();

    current_function = self.current_function.as_ref().unwrap().clone();

    current_function.body = Some(Box::new(ir.clone()));

    let instruction = IRInstruction::Function(current_function);

    self.context.pop();
    self.current_function = None;

    Ok(instruction)
  }

  fn visit_property_statement(&mut self, statement: &PropertyStatement) -> AnalyzerResult {
    if self.is_allready_declared(&statement.name.span.literal) {
      return Err(AnalyzerDiagnosticError::PropertyAlreadyDefined(
        statement.name.clone(),
      ));
    }

    if self
      .context
      .iter()
      .any(|c| !matches!(c, AnalyzerContext::Class))
    {
      return Err(AnalyzerDiagnosticError::PropertyOutsideClass(
        statement.name.clone(),
      ));
    }

    self.declare(statement.name.span.literal.clone());

    let mut value = IRInstruction::Literal(IRLiteral::new(AnalyzerValue::Null));
    let data_type = statement.type_annotation.clone();

    if let Some(initializer) = &statement.initializer {
      let expression = self.analyzer(initializer)?;
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
        IRInstruction::Ternary(ternary) => {
          value = IRInstruction::Ternary(ternary);
        }
        IRInstruction::Call(call) => {
          value = IRInstruction::Call(call);
        }
        IRInstruction::Class(class) => {
          value = IRInstruction::Class(class);
        }
        IRInstruction::Logical(logical) => {
          value = IRInstruction::Logical(logical);
        }
        IRInstruction::Array(array) => {
          value = IRInstruction::Array(array);
        }
        _ => (),
      }
    }

    let variable = IRVariable::new(
      statement.name.span.literal.clone(),
      data_type.clone(),
      Some(Box::new(value.clone())),
      IRVariableMetadata::new(
        statement.metadata.is_mutable,
        statement.metadata.is_reference,
        false,
        false,
        false,
        true,
        false,
        false,
        false,
      ),
    );

    self.define(variable.name.clone());

    self.scopes_variables.push(variable.clone());

    Ok(IRInstruction::Variable(variable.clone()))
  }
}

impl Analyzer {
  pub fn new(current_file: String) -> Self {
    let mut irs = HashMap::new();
    let block_stack: HashMap<String, bool> = HashMap::new();

    irs.insert(current_file.clone(), Vec::new());

    Self {
      irs,
      diagnostics: Vec::new(),
      block_stack: vec![block_stack],
      scopes_variables: Vec::new(),
      current_function: None,
      current_file,
      current_class: None,
      context: Vec::new(),
    }
  }

  pub fn analyze(&mut self, statements: &Vec<Statement>) {
    for statement in statements {
      match self.analyze_statement(statement) {
        Ok(ir) => {
          let current_ir = self.irs.get_mut(&self.current_file).unwrap();
          current_ir.push(ir.clone());
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

  fn begin_scope(&mut self) {
    self
      .block_stack
      .push(self.block_stack.clone().last().unwrap().clone());
  }

  fn end_scope(&mut self) {
    self.block_stack.pop().unwrap();
  }

  fn declare(&mut self, name: String) {
    if self.block_stack.is_empty() {
      return;
    }

    let current_block = self.block_stack.last_mut().unwrap();

    current_block.insert(name.clone(), false);
  }

  fn resolve_std_import(&mut self, lib: String, block_stack: &mut HashMap<String, bool>) {
    let current_ir = self.irs.get_mut(&self.current_file).unwrap();
    match lib.clone().as_str() {
      "std:io" => {
        current_ir.push(IRInstruction::Function(IRFunction::new(
          "println".to_string(),
          vec![IRVariable::new(
            "message".to_string(),
            DataType::None,
            None,
            IRVariableMetadata::new(false, false, true, false, false, false, false, false, false),
          )],
          DataType::Void,
          None,
          IRFunctionMetadata::new(false, true, true, true, false, false, false),
        )));

        block_stack.insert("println".to_string(), true);
      }
      "std:string" => {
        current_ir.push(IRInstruction::Function(IRFunction::new(
          "toString".to_string(),
          vec![IRVariable::new(
            "value".to_string(),
            DataType::None,
            None,
            IRVariableMetadata::new(false, false, true, false, false, false, false, false, false),
          )],
          DataType::String,
          None,
          IRFunctionMetadata::new(false, true, true, true, false, false, false),
        )));

        block_stack.insert("toString".to_string(), true);
      }
      &_ => {}
    }
  }

  fn resolve_module_import(
    &mut self,
    statement: &Import,
    block_stack: &mut HashMap<String, bool>,
  ) -> Result<(), AnalyzerDiagnosticError> {
    let mut analyzer = Analyzer::new(statement.module_path.span.literal.clone());
    match fs::read_to_string(format!("{}.{}", statement.module_path.span.literal, "ign")) {
      Ok(source) => {
        let mut lexer: Lexer<'_> = Lexer::new(&source, statement.module_path.span.literal.clone());
        lexer.scan_tokens();

        let mut parser: Parser = Parser::new(lexer.tokens);
        let statements = parser.parse();

        if let Ok(parser_reult) = statements {
          analyzer.analyze(&parser_reult);
        }
      }
      Err(_) => {
        return Err(AnalyzerDiagnosticError::ModuleNotFound(
          statement.module_path.clone(),
        ))
      }
    };

    analyzer.diagnostics.iter().for_each(|d| {
      self.diagnostics.push(d.clone());
    });

    let current_ir = analyzer
      .irs
      .get(&statement.module_path.span.literal)
      .unwrap()
      .clone();

    for ir in &current_ir {
      self.define_import(statement, ir.clone(), block_stack)?;
    }

    self.irs.insert(
      statement.module_path.span.literal.clone(),
      current_ir.clone(),
    );

    Ok(())
  }

  fn define_import(
    &mut self,
    statement: &Import,
    ir: IRInstruction,
    block_stack: &mut HashMap<String, bool>,
  ) -> Result<(), AnalyzerDiagnosticError> {
    let current_ir = self.irs.get_mut(&self.current_file).unwrap();

    if let IRInstruction::Function(f) = ir {
      for symbol in &statement.symbols {
        if symbol.name.span.literal == f.name && !f.metadata.is_exported {
          return Err(AnalyzerDiagnosticError::ImportedFunctionIsNotExported(
            symbol.name.clone(),
          ));
        }

        if symbol.name.span.literal == f.name && f.metadata.is_exported {
          let mut metadata = f.metadata.clone();
          metadata.is_imported = true;
          if symbol.alias.is_some() {
            block_stack.insert(symbol.alias.as_ref().unwrap().span.literal.clone(), true);
            current_ir.push(
              IRInstruction::Function(IRFunction::new(
                symbol.alias.as_ref().unwrap().span.literal.clone(),
                f.parameters.clone(),
                f.return_type.clone(),
                f.body.clone(),
                metadata,
              ))
              .clone(),
            );
          } else {
            block_stack.insert(symbol.name.span.literal.clone(), true);
            metadata.is_exported = false;
            current_ir.push(
              IRInstruction::Function(IRFunction::new(
                symbol.name.span.literal.clone(),
                f.parameters.clone(),
                f.return_type.clone(),
                f.body.clone(),
                metadata,
              ))
              .clone(),
            );
          }
        }
      }
    };

    Ok(())
  }

  fn is_allready_declared(&self, name: &String) -> bool {
    if self.block_stack.is_empty() {
      return false;
    }

    let current_block = self.block_stack.last().unwrap();

    current_block.get(name).is_some()
  }

  fn define(&mut self, name: String) {
    if self.block_stack.is_empty() {
      return;
    }

    let current_block = self.block_stack.last_mut().unwrap();

    current_block.insert(name.clone(), true);
  }

  fn define_parameter(&mut self, name: String) {
    if self.block_stack.is_empty() {
      return;
    }

    let current_block = self.block_stack.last_mut().unwrap();

    current_block.insert(name.clone(), true);
  }

  fn _find_function_in_ir(&self, name: String) -> Option<IRFunction> {
    let irs = self.irs.get(&self.current_file).unwrap();

    let function = irs.iter().find(|ir| match ir {
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
      IRInstructionType::And | IRInstructionType::Or => matches!(
        (left, right),
        (
          IRInstruction::Literal(IRLiteral {
            value: AnalyzerValue::Boolean(_),
          }),
          IRInstruction::Literal(IRLiteral {
            value: AnalyzerValue::Boolean(_),
          }),
        )
      ),
      _ => false,
    }
  }

  fn are_types_unary_compatible(
    &self,
    right: &IRInstruction,
    operator: &IRInstructionType,
  ) -> bool {
    match operator {
      IRInstructionType::Sub => matches!(
        right,
        IRInstruction::Literal(IRLiteral {
          value: AnalyzerValue::Int(_),
        }) | IRInstruction::Literal(IRLiteral {
          value: AnalyzerValue::Float(_),
        })
      ),
      IRInstructionType::Not => matches!(
        right,
        IRInstruction::Literal(IRLiteral {
          value: AnalyzerValue::Boolean(_),
        }) | IRInstruction::Literal(IRLiteral {
          value: AnalyzerValue::Int(_),
        }) | IRInstruction::Literal(IRLiteral {
          value: AnalyzerValue::String(_),
        }) | IRInstruction::Literal(IRLiteral {
          value: AnalyzerValue::Float(_),
        }) | IRInstruction::Literal(IRLiteral {
          value: AnalyzerValue::Null,
        })
      ),
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
      IRInstruction::Assign(a) => self.extract_data_type(&a.value.clone()),
      IRInstruction::Call(c) => c.return_type.clone(),
      IRInstruction::Return(r) => r.data_type.clone(),
      IRInstruction::Array(array) => array.data_type.clone(),
      IRInstruction::Class(c) => DataType::ClassType(c.name.clone()),
      IRInstruction::ClassInstance(c) => DataType::ClassType(c.class.name.clone()),
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
      (DataType::Float, DataType::Float) => (true, DataType::Float),
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
      (DataType::Float, DataType::Float) => (true, DataType::Float),
      (DataType::Int, DataType::Float) => (true, DataType::Float),
      (DataType::Float, DataType::Int) => (true, DataType::Float),
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
      (DataType::Float, DataType::Float) => (true, DataType::Boolean),
      (DataType::Int, DataType::Float) => (true, DataType::Boolean),
      (DataType::Float, DataType::Int) => (true, DataType::Boolean),
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
      (DataType::Float, DataType::Float) => (true, DataType::Boolean),
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
      IRInstructionType::Concatenate => {
        if left_type == DataType::String && right_type == DataType::String {
          (true, DataType::String)
        } else {
          (false, DataType::None)
        }
      }
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
      IRInstructionType::Mod => {
        if left_type == DataType::Int && right_type == DataType::Int {
          (true, DataType::Int)
        } else {
          (false, DataType::None)
        }
      }
      _ => (false, DataType::None),
    }
  }

  fn is_iterable(&self, iterable: &IRInstruction) -> bool {
    match iterable {
      IRInstruction::Variable(var) => matches!(var.data_type, DataType::Array(_)),
      _ => false,
    }
  }

  fn find_class_in_ir(&self, name: String) -> Option<IRClass> {
    let irs = self.irs.get(&self.current_file).unwrap();

    let class = irs.iter().find(|ir| match ir {
      IRInstruction::Class(c) => c.name == name,
      _ => false,
    });

    match class {
      Some(IRInstruction::Class(c)) => Some(c.clone()),
      _ => None,
    }
  }

  fn find_matching_constructor(
    &self,
    class: &IRClass,
    arguments: &Vec<IRInstruction>,
  ) -> Option<IRFunction> {
    let constructors = class.methods.iter().filter(|m| m.metadata.is_constructor);
    for constructor in constructors {
      if constructor.parameters.len() != arguments.len() {
        continue;
      }

      let mut is_matching = true;

      for (i, arg) in arguments.iter().enumerate() {
        let arg_type = self.extract_data_type(arg);
        let param_type = constructor.parameters[i].data_type.clone();
        if arg_type != param_type {
          is_matching = false;
          break;
        }
      }
      if is_matching {
        return Some(constructor.clone());
      }
    }

    None
  }

  fn verify_arguments(&self, arguments: &Vec<IRInstruction>, parameters: &Vec<IRVariable>) -> bool {
    if arguments.len() != parameters.len() {
      return false;
    }

    for (i, arg) in arguments.iter().enumerate() {
      let arg_type = self.extract_data_type(arg);
      let param_type = parameters[i].data_type.clone();
      if arg_type != param_type {
        return false;
      }
    }

    true
  }
}
