use ast::{
  statement::{
    class::Class,
    variable::VariableMetadata,
    for_in::ForIn,
    import::{Import, ImportSource, ImportSymbol},
    function::FunctionDecorator,
    break_statement::BreakStatement,
    continue_statement::Continue,
    method::{MethodStatement, MethodMetadata},
    property::PropertyStatement,
  },
  expression::{array::Array, get::Get, new::NewExpression, set::Set, method_call::MethodCall},
};
use enums::{data_type::DataType, token_type::TokenType};
use lexer::text_span::TextSpan;

use {
  lexer::token::Token,
  ast::expression::{
    Expression, binary::Binary, unary::Unary, literal::Literal, grouping::Grouping,
    logical::Logical, assign::Assign, variable::VariableExpression, ternary, call::Call,
  },
  enums::literal_value::LiteralValue,
  ast::statement::{
    Statement,
    variable::Variable,
    expression::ExpressionStatement,
    if_statement::IfStatement,
    block::Block,
    while_statement::WhileStatement,
    function::{FunctionStatement, FunctionParameter},
    return_statement::Return,
  },
};

#[derive(Debug)]
pub enum ParserDiagnosticError {
  ExpectedExpression(Token),
  ExpectedToken(TokenType, Token),
  ExpectedVariableName(Token),
  ExpectedReturnTypeAfterFunction(Token),
  ExpectedAfterExpression(TokenType, Token, Token),
  ExpectedExpressionAfter(Token),
  UnexpectedToken(TokenType, Token),
  InvalidAssignmentTarget(TextSpan),
  ExpectedTypeAfterVariable(Token),
  InvalidNumberOfArguments(usize, usize, Token),
}

type ParserResult<T> = Result<T, Box<ParserDiagnosticError>>;

enum ParserContext {
  Function,
  Class,
}

pub struct Parser {
  pub tokens: Vec<Token>,
  current: usize,
  pub diagnostics: Vec<ParserDiagnosticError>,
  context: Vec<ParserContext>,
  class_declarations: Vec<String>,
}

impl Parser {
  pub fn new(tokens: Vec<Token>) -> Self {
    Self {
      tokens,
      current: 0,
      diagnostics: Vec::new(),
      context: Vec::new(),
      class_declarations: Vec::new(),
    }
  }

  fn report_error(&mut self, error: ParserDiagnosticError) {
    self.diagnostics.push(error);
  }

  pub fn parse(&mut self) -> Result<Vec<Statement>, ()> {
    let mut statements: Vec<Statement> = vec![];
    while !self.is_at_end() {
      match self.declaration() {
        Ok(result) => statements.push(result),
        Err(error) => {
          self.report_error(*error);
        }
      };
    }

    if !self.diagnostics.is_empty() {
      return Err(());
    }

    Ok(statements)
  }

  fn expression(&mut self) -> ParserResult<Expression> {
    self.assignment()
  }

  // equelity -> comparison (("!=" | "==") comparison)*;
  fn equality(&mut self) -> ParserResult<Expression> {
    let mut expression = self.comparison()?;

    while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
      let operator: Token = self.previous();
      let right = self.comparison()?;

      let data_type: DataType = DataType::Boolean;

      expression = Expression::Binary(Binary::new(
        Box::new(expression),
        operator,
        Box::new(right),
        data_type,
      ));
    }

    Ok(expression)
  }

  // comparison -> term ((">" | ">=" | "<" | "<=") term)*;
  fn comparison(&mut self) -> ParserResult<Expression> {
    let mut expression = self.term()?;

    while self.match_token(&[
      TokenType::Greater,
      TokenType::GreaterEqual,
      TokenType::Less,
      TokenType::LessEqual,
    ]) {
      let operator: Token = self.previous();
      let right = self.term()?;

      let data_type: DataType = DataType::Boolean;

      expression = Expression::Binary(Binary::new(
        Box::new(expression),
        operator,
        Box::new(right),
        data_type,
      ));
    }

    Ok(expression)
  }

  // term -> factor (("-" | "+") factor)*;
  fn term(&mut self) -> ParserResult<Expression> {
    let mut expression = self.factor()?;

    while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
      let operator: Token = self.previous();
      let right = self.factor()?;

      let left_type = self.get_expression_type(&expression);
      let right_type = self.get_expression_type(&right);
      let operator_kind = operator.kind.clone();

      expression = Expression::Binary(Binary::new(
        Box::new(expression),
        operator,
        Box::new(right),
        self.get_data_type_by_operator(Some(left_type), right_type, operator_kind),
      ));
    }

    Ok(expression)
  }

  // factor -> ("!" | "-") unary | primary;
  fn factor(&mut self) -> ParserResult<Expression> {
    let mut expression: Expression = self.unary()?;

    while self.match_token(&[TokenType::Slash, TokenType::Asterisk, TokenType::Mod]) {
      let operator: Token = self.previous();
      let right: Expression = self.unary()?;

      let left_type = self.get_expression_type(&expression);
      let right_type = self.get_expression_type(&right);

      let operator_kind = operator.kind.clone();

      expression = Expression::Binary(Binary::new(
        Box::new(expression),
        operator,
        Box::new(right),
        self.get_data_type_by_operator(Some(left_type), right_type, operator_kind),
      ));
    }

    Ok(expression)
  }

  // unary -> ("!" | "-") unary | call;
  fn unary(&mut self) -> ParserResult<Expression> {
    if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
      let operator = self.previous();
      let right: Expression = self.unary()?;

      let right_type = self.get_expression_type(&right);
      let operator_kind = operator.kind.clone();

      return Ok(Expression::Unary(Unary::new(
        operator,
        Box::new(right),
        self.get_data_type_by_operator(None, right_type, operator_kind),
      )));
    }

    self.call()
  }

  fn call(&mut self) -> ParserResult<Expression> {
    if self.match_token(&[TokenType::New]) {
      let name = self.consume(TokenType::Identifier)?;

      self.consume(TokenType::LeftParen)?;

      let mut arguments: Vec<Expression> = Vec::new();

      if !self.check(TokenType::RightParen) {
        loop {
          if arguments.len() >= 255 {
            let token = &self.peek();
            return Err(Box::new(ParserDiagnosticError::InvalidNumberOfArguments(
              255,
              arguments.len(),
              token.clone(),
            )));
          }
          arguments.push(self.expression()?);
          if !self.match_token(&[TokenType::Comma]) {
            break;
          }
        }
      }

      self.consume(TokenType::RightParen)?;

      return Ok(Expression::New(NewExpression::new(name, arguments)));
    }

    let mut expression: Expression = self.primary()?;

    loop {
      if self.match_token(&[TokenType::Dot]) {
        let name = self.consume(TokenType::Identifier)?;

        if self.match_token(&[TokenType::LeftParen]) {
          expression = Expression::Variable(VariableExpression::new(name.clone(), DataType::Pending));

          let calle = self.finish_call(expression.clone())?;

          expression = Expression::MethodCall(MethodCall::new(
            Box::new(name),
            Box::new(calle.clone()),
            self.get_expression_type(&calle),
            Box::new(expression),
          ));

          continue;
        }

        expression = Expression::Get(Get::new(Box::new(expression), name));

        continue;
      }

      if !self.match_token(&[TokenType::LeftParen]) {
        break;
      }

      expression = self.finish_call(expression)?;
    }

    Ok(expression)
  }

  fn primary(&mut self) -> ParserResult<Expression> {
    let token = self.peek();

    match token.kind {
      TokenType::True
      | TokenType::False
      | TokenType::Null
      | TokenType::Int
      | TokenType::Float
      | TokenType::String => {
        self.advance();
        Ok(Expression::Literal(Literal::new(
          LiteralValue::from_token_type(token.kind.clone(), token.span.literal.clone()),
        )))
      }
      TokenType::LeftBrack => {
        self.advance();

        let mut elements = Vec::new();
        if !self.check(TokenType::RightBrack) {
          loop {
            elements.push(self.expression()?);
            if !self.match_token(&[TokenType::Comma]) {
              break;
            }
          }
        }

        self.consume(TokenType::RightBrack)?;

        let data_type = DataType::Array(Box::new(DataType::Pending));
        Ok(Expression::Array(Array::new(token, elements, data_type)))
      }
      TokenType::LeftParen => {
        self.advance();
        let expression = self.expression()?;
        self.consume(TokenType::RightParen)?;

        Ok(Expression::Grouping(Grouping::new(Box::new(expression))))
      }
      TokenType::Identifier => {
        self.advance();
        let kind = token.kind.clone();
        Ok(Expression::Variable(VariableExpression::new(
          token,
          DataType::from_token_type(kind),
        )))
      }
      _ => Err(Box::new(ParserDiagnosticError::ExpectedExpression(
        token.clone(),
      ))),
    }
  }

  fn finish_call(&mut self, callee: Expression) -> ParserResult<Expression> {
    let mut arguments: Vec<Expression> = Vec::new();

    if !self.check(TokenType::RightParen) {
      loop {
        if arguments.len() >= 255 {
          let token = &self.peek();

          return Err(Box::new(ParserDiagnosticError::InvalidNumberOfArguments(
            255,
            arguments.len(),
            token.clone(),
          )));
        }

        arguments.push(self.expression()?);

        if !self.match_token(&[TokenType::Comma]) {
          break;
        }
      }
    }

    let token = self.consume(TokenType::RightParen)?;

    Ok(Expression::Call(Call::new(
      Box::new(callee),
      token,
      arguments,
      DataType::Pending,
    )))
  }

  fn get_data_type_by_operator(
    &mut self,
    left: Option<DataType>,
    right: DataType,
    operator: TokenType,
  ) -> DataType {
    match (left, right, operator) {
      (Some(DataType::Int), DataType::Int, TokenType::Plus)
      | (Some(DataType::Int), DataType::Int, TokenType::Minus)
      | (None, DataType::Int, TokenType::Minus)
      | (Some(DataType::Int), DataType::Int, TokenType::Asterisk)
      | (Some(DataType::Int), DataType::Int, TokenType::Slash) => DataType::Int,
      (Some(DataType::Float), DataType::Float, TokenType::Plus)
      | (Some(DataType::Float), DataType::Float, TokenType::Minus)
      | (Some(DataType::Float), DataType::Float, TokenType::Slash)
      | (Some(DataType::Float), DataType::Float, TokenType::Asterisk)
      | (None, DataType::Float, TokenType::Minus) => DataType::Float,
      (Some(DataType::String), DataType::String, TokenType::Plus) => DataType::String,
      (None, DataType::Boolean, TokenType::Bang) | (None, DataType::String, TokenType::Bang) => {
        DataType::Boolean
      }
      _ => DataType::Pending,
    }
  }

  fn get_expression_type(&self, expression: &Expression) -> DataType {
    match expression {
      Expression::Binary(binary) => binary.data_type.clone(),
      Expression::Unary(unary) => unary.data_type.clone(),
      Expression::Literal(literal) => match literal.value {
        LiteralValue::Boolean(_) => DataType::Boolean,
        LiteralValue::Char(_) => DataType::Char,
        LiteralValue::Float(_) => DataType::Float,
        LiteralValue::Int(_) => DataType::Int,
        LiteralValue::String(_) => DataType::String,
        _ => DataType::Int,
      },
      Expression::Grouping(grouping) => self.get_expression_type(&grouping.expression),
      Expression::Variable(variable) => DataType::Variable(variable.name.span.literal.clone()),
      Expression::Assign(assign) => assign.data_type.clone(),
      Expression::Logical(logical) => logical.data_type.clone(),
      Expression::Ternary(ternary) => ternary.data_type.clone(),
      Expression::Call(call) => call.return_type.clone(),
      Expression::Array(a) => a.data_type.clone(),
      Expression::Get(_) => DataType::Pending,
      Expression::New(new) => DataType::ClassType(new.name.span.literal.clone()),
      Expression::Set(set) => set.data_type.clone(),
      Expression::MethodCall(method) => method.data_type.clone(),
    }
  }

  fn synchronize(&mut self) {
    self.advance();

    while !self.is_at_end() {
      if self.previous().kind == TokenType::SemiColon {
        return;
      }

      match self.peek().kind {
        TokenType::Class
        | TokenType::Function
        | TokenType::Let
        | TokenType::Const
        | TokenType::For
        | TokenType::If
        | TokenType::Return => return,
        _ => (),
      };

      self.advance();
    }
  }

  fn declaration(&mut self) -> ParserResult<Statement> {
    if self.match_token(&[TokenType::Let]) {
      return self.variable_declaration();
    }

    if self.match_token(&[TokenType::Class]) {
      self.context.push(ParserContext::Class);
      let value = self.class_declaration();

      self.context.pop();

      return value;
    }

    if self.match_token(&[TokenType::Function]) {
      self.context.push(ParserContext::Function);

      let value = self.function_statement(false, None);

      self.context.pop();

      return value;
    }

    if self.match_token(&[TokenType::Return]) {
      return self.return_statement();
    }

    if self.match_token(&[TokenType::While]) {
      return self.while_statement();
    }

    if self.match_token(&[TokenType::For]) {
      return self.for_statement();
    }

    if self.match_token(&[TokenType::Import]) {
      return self.import_statement();
    }

    if self.match_token(&[TokenType::Export]) {
      return self.export_statement();
    }

    if self.match_token(&[TokenType::At]) {
      return self.decoration_statement();
    }

    if self.match_token(&[TokenType::Break]) {
      return self.break_statement();
    }

    if self.match_token(&[TokenType::Continue]) {
      return self.continue_statement();
    }

    match self.statement() {
      Ok(statement) => Ok(statement),
      Err(error) => {
        self.synchronize();
        Err(error)
      }
    }
  }

  fn continue_statement(&mut self) -> ParserResult<Statement> {
    let token = self.previous();

    self.consume(TokenType::SemiColon)?;

    Ok(Statement::Continue(Continue::new(token)))
  }

  fn break_statement(&mut self) -> ParserResult<Statement> {
    let token = self.previous();

    self.consume(TokenType::SemiColon)?;

    Ok(Statement::Break(BreakStatement::new(token)))
  }

  fn return_statement(&mut self) -> ParserResult<Statement> {
    let keyword = self.previous();

    if self.check(TokenType::SemiColon) {
      self.advance();
      return Ok(Statement::Return(Return::new(None, Box::new(keyword))));
    }

    let value = self.expression()?;

    self.consume(TokenType::SemiColon)?;

    let result = Return::new(Some(Box::new(value)), Box::new(keyword));
    Ok(Statement::Return(result))
  }

  fn function_statement(
    &mut self,
    is_public: bool,
    decorator: Option<FunctionDecorator>,
  ) -> ParserResult<Statement> {
    let name: Token = self.consume(TokenType::Identifier)?;

    self.consume(TokenType::LeftParen)?;

    let mut parameters: Vec<FunctionParameter> = Vec::new();

    if !self.check(TokenType::RightParen) {
      loop {
        if parameters.len() >= 255 {
          return Err(Box::new(ParserDiagnosticError::InvalidNumberOfArguments(
            255,
            parameters.len(),
            name.clone(),
          )));
        }

        let is_mut: bool = if self.peek().kind == TokenType::Mut {
          self.advance();
          true
        } else {
          false
        };

        let param = self.consume(TokenType::Identifier)?;

        self.consume(TokenType::Colon)?;
        let token = self.advance();

        let mut data_type = DataType::from_token_type(token.kind);

        if self.check(TokenType::LeftBrack) {
          self.advance();
          self.consume(TokenType::RightBrack)?;

          data_type = DataType::Array(Box::new(data_type));
        }

        parameters.push(FunctionParameter::new(param, data_type, is_mut));

        if !self.match_token(&[TokenType::Comma]) {
          break;
        }
      }
    }

    self.consume(TokenType::RightParen)?;

    self.consume(TokenType::Colon)?;

    let return_type: Option<DataType>;
    if self.match_token(&[
      TokenType::Void,
      TokenType::IntType,
      TokenType::FloatType,
      TokenType::StringType,
      TokenType::BooleanType,
      TokenType::CharType,
    ]) {
      let mut data_type: DataType = DataType::from_token_type(self.previous().kind);

      if self.check(TokenType::LeftBrack) {
        self.consume(TokenType::RightBrack)?;
        data_type = DataType::Array(Box::new(data_type));
      }

      return_type = Some(data_type);
    } else {
      let token = &self.peek();

      return Err(Box::new(
        ParserDiagnosticError::ExpectedReturnTypeAfterFunction(token.clone()),
      ));
    }

    let mut body: Vec<Statement> = Vec::new();

    if !self.match_token(&[TokenType::SemiColon]) {
      self.consume(TokenType::LeftBrace)?;

      body.push(self.block()?);
    }

    Ok(Statement::FunctionStatement(FunctionStatement::new(
      name,
      parameters,
      body,
      return_type,
      is_public,
      if decorator.is_some() {
        vec![decorator.unwrap()]
      } else {
        vec![]
      },
    )))
  }

  fn block(&mut self) -> ParserResult<Statement> {
    let mut statements: Vec<Statement> = Vec::new();

    while !self.check(TokenType::RightBrace) && !self.is_at_end() {
      statements.push(self.declaration()?);
    }

    self.consume(TokenType::RightBrace)?;

    Ok(Statement::Block(Block::new(statements)))
  }

  fn variable_declaration(&mut self) -> ParserResult<Statement> {
    let mutable: bool = if self.peek().kind == TokenType::Mut {
      self.advance();
      true
    } else {
      false
    };

    let name: Token = self.consume(TokenType::Identifier)?;

    let mut initializer: Option<Expression> = None;

    self.consume(TokenType::Colon)?;

    let token = self.peek();

    let mut type_annotation = DataType::from_token_type(token.kind.clone());

    if type_annotation == DataType::None && self.class_declarations.contains(&token.span.literal) {
      type_annotation = DataType::ClassType(token.span.literal.clone());
    }

    if type_annotation == DataType::None {
      return Err(Box::new(ParserDiagnosticError::ExpectedTypeAfterVariable(
        token,
      )));
    }

    self.advance();

    if self.match_token(&[TokenType::SemiColon]) {
      return Ok(Statement::Variable(Variable::new(
        Box::new(name),
        None,
        type_annotation,
        VariableMetadata::new(mutable, false, false, false, false),
      )));
    }

    if self.match_token(&[TokenType::LeftBrack]) {
      self.consume(TokenType::RightBrack)?;

      type_annotation = DataType::Array(Box::new(type_annotation));
    }

    if self.match_token(&[TokenType::Equal]) {
      let mut value = self.expression()?;

      if let Expression::Array(a) = &value {
        value = Expression::Array(Array::new(
          a.token.clone(),
          a.elements.clone(),
          type_annotation.clone(),
        ));
      }

      initializer = Some(value);
    }

    self.consume(TokenType::SemiColon)?;

    if let Some(ini) = initializer {
      Ok(Statement::Variable(Variable::new(
        Box::new(name),
        Some(Box::new(ini)),
        type_annotation,
        VariableMetadata::new(mutable, false, false, false, false),
      )))
    } else {
      let token = self.peek();
      Err(Box::new(ParserDiagnosticError::ExpectedExpression(
        token.clone(),
      )))
    }
  }

  // statement -> expressionStatement | ifStatement;
  fn statement(&mut self) -> ParserResult<Statement> {
    if self.match_token(&[TokenType::LeftBrace]) {
      return self.block();
    }

    if self.match_token(&[TokenType::If]) {
      return self.if_statement();
    }

    self.expression_statement()
  }

  // expressionStatement -> expression ";";
  fn expression_statement(&mut self) -> ParserResult<Statement> {
    let expression = self.expression()?;

    self.consume(TokenType::SemiColon)?;

    Ok(Statement::Expression(ExpressionStatement::new(Box::new(
      expression,
    ))))
  }

  fn assignment(&mut self) -> ParserResult<Expression> {
    let mut expression: Expression = self.ternary()?;

    if self.match_token(&[TokenType::Equal]) {
      let equals: Token = self.previous();
      let value: Expression = self.assignment()?;

      match expression {
        Expression::Variable(variable) => {
          expression = Expression::Assign(Assign::new(
            variable.name,
            Box::new(value),
            variable.data_type,
          ));
        }
        Expression::Get(get) => {
          expression = Expression::Set(Set::new(
            Box::new(get.name),
            Box::new(value.clone()),
            get.object,
            self.get_expression_type(&value),
          ));
        }
        _ => {
          return Err(Box::new(ParserDiagnosticError::InvalidAssignmentTarget(
            equals.span.clone(),
          )));
        }
      }
    }

    Ok(expression)
  }

  fn ternary(&mut self) -> ParserResult<Expression> {
    let mut children: Vec<Expression> = Vec::new();

    children.push(self.or_expression()?);

    while self.match_token(&[TokenType::QuestionMark]) {
      children.push(self.expression()?);

      self.consume(TokenType::Colon)?;

      children.push(self.expression()?);
    }

    if children.len() == 1 {
      return Ok(children.pop().unwrap());
    }

    let else_branch = children.pop().unwrap();
    let then_branch = children.pop().unwrap();
    let condition = children.pop().unwrap();

    let mut expression: Expression = Expression::Ternary(ternary::Ternary::new(
      Box::new(condition),
      Box::new(then_branch),
      Box::new(else_branch),
      Box::new(self.peek()),
      DataType::Pending,
    ));

    while !children.is_empty() {
      expression = Expression::Ternary(ternary::Ternary::new(
        Box::new(children.pop().unwrap()),
        Box::new(expression),
        Box::new(children.pop().unwrap()),
        Box::new(self.peek()),
        DataType::Pending,
      ));
    }

    Ok(expression)
  }

  fn or_expression(&mut self) -> ParserResult<Expression> {
    let mut expression = self.and_expression()?;

    while self.match_token(&[TokenType::Or]) {
      let operator: Token = self.previous();
      let right = self.and_expression()?;

      expression = Expression::Logical(Logical::new(
        Box::new(expression),
        operator,
        Box::new(right),
      ));
    }

    Ok(expression)
  }

  fn and_expression(&mut self) -> ParserResult<Expression> {
    let mut expression = self.equality()?;

    while self.match_token(&[TokenType::And]) {
      let operator: Token = self.previous();
      let right = self.equality()?;

      expression = Expression::Logical(Logical::new(
        Box::new(expression),
        operator,
        Box::new(right),
      ));
    }

    Ok(expression)
  }

  fn while_statement(&mut self) -> ParserResult<Statement> {
    self.consume(TokenType::LeftParen)?;

    let condition: Expression = self.expression()?;

    self.consume(TokenType::RightParen)?;

    let body: Statement = self.statement()?;

    Ok(Statement::WhileStatement(WhileStatement::new(
      Box::new(condition),
      Box::new(body),
    )))
  }

  fn for_statement(&mut self) -> ParserResult<Statement> {
    self.consume(TokenType::LeftParen)?;

    self.consume(TokenType::Let)?;
    let item: Token = self.consume(TokenType::Identifier)?;

    let variable = Variable::new(
      Box::new(item.clone()),
      None,
      DataType::Pending,
      VariableMetadata::new(true, false, false, false, false),
    );

    self.consume(TokenType::In)?;

    let iterable: Expression = self.expression()?;

    self.consume(TokenType::RightParen)?;

    let body: Statement = self.statement()?;

    let statement = ForIn::new(variable, iterable, body, item);

    Ok(Statement::ForIn(statement))
  }

  fn if_statement(&mut self) -> ParserResult<Statement> {
    self.consume(TokenType::LeftParen)?;

    let condition: Expression = self.expression()?;

    self.consume(TokenType::RightParen)?;

    let then_branch: Statement = self.statement()?;

    let else_branch: Option<Box<Statement>> = if self.match_token(&[TokenType::Else]) {
      Some(Box::new(self.statement()?))
    } else {
      None
    };

    Ok(Statement::IfStatement(IfStatement::new(
      Box::new(condition),
      Box::new(then_branch),
      else_branch,
    )))
  }

  fn consume(&mut self, kind: TokenType) -> ParserResult<Token> {
    let token: Token = self.peek();
    if token.kind == kind {
      return Ok(self.advance());
    }

    let error = match kind {
      TokenType::SemiColon => {
        ParserDiagnosticError::UnexpectedToken(TokenType::SemiColon, token.clone())
      }
      TokenType::Colon => ParserDiagnosticError::UnexpectedToken(TokenType::Colon, token.clone()),
      TokenType::Identifier => ParserDiagnosticError::ExpectedVariableName(token.clone()),
      TokenType::QuestionMark => {
        ParserDiagnosticError::ExpectedToken(TokenType::QuestionMark, token.clone())
      }
      TokenType::LeftParen | TokenType::RightParen => {
        let expression = self.previous();

        ParserDiagnosticError::ExpectedAfterExpression(
          kind.clone(),
          expression.clone(),
          token.clone(),
        )
      }
      _ => ParserDiagnosticError::ExpectedToken(kind.clone(), token.clone()),
    };

    Err(Box::new(error))
  }

  fn peek(&mut self) -> Token {
    self.tokens[self.current].clone()
  }

  fn is_at_end(&mut self) -> bool {
    self.peek().kind == TokenType::Eof
  }

  fn match_token(&mut self, kinds: &[TokenType]) -> bool {
    for kind in kinds {
      if self.check(kind.clone()) {
        self.advance();
        return true;
      }
    }

    false
  }

  fn check(&mut self, kind: TokenType) -> bool {
    if self.is_at_end() {
      return false;
    }

    self.peek().kind == kind
  }

  fn advance(&mut self) -> Token {
    if !self.is_at_end() {
      self.current += 1;
    }

    self.previous()
  }

  fn previous(&mut self) -> Token {
    self.tokens[self.current - 1].clone()
  }

  fn property_class_declaration(
    &mut self,
    name: Token,
    is_mutable: bool,
    is_public: bool,
  ) -> ParserResult<Statement> {
    self.consume(TokenType::Colon)?;

    let token = self.peek();

    let mut type_annotation = DataType::from_token_type(token.kind.clone());

    if type_annotation == DataType::None && self.class_declarations.contains(&token.span.literal) {
      type_annotation = DataType::ClassType(token.span.literal.clone());
    }

    if type_annotation == DataType::None {
      return Err(Box::new(ParserDiagnosticError::ExpectedTypeAfterVariable(
        self.peek(),
      )));
    }

    self.advance();

    if self.match_token(&[TokenType::LeftBrack]) {
      self.consume(TokenType::RightBrack)?;

      type_annotation = DataType::Array(Box::new(type_annotation));
    }

    let mut initializer: Option<Box<Expression>> = None;

    if self.match_token(&[TokenType::Equal]) {
      let mut value = self.expression()?;

      if let Expression::Array(a) = value {
        value = Expression::Array(Array::new(
          a.token.clone(),
          a.elements,
          type_annotation.clone(),
        ));
      };

      initializer = Some(Box::new(value));
    }

    self.consume(TokenType::SemiColon)?;

    Ok(Statement::Property(PropertyStatement::new(
      Box::new(name),
      initializer,
      type_annotation,
      VariableMetadata::new(is_mutable, false, false, is_public, false),
    )))
  }

  fn method_declaration(
    &mut self,
    name: Token,
    is_public: bool,
    decorator: Option<FunctionDecorator>,
  ) -> ParserResult<Statement> {
    let mut parameters: Vec<FunctionParameter> = Vec::new();

    if !self.check(TokenType::RightParen) {
      loop {
        if parameters.len() >= 255 {
          return Err(Box::new(ParserDiagnosticError::InvalidNumberOfArguments(
            255,
            parameters.len(),
            name.clone(),
          )));
        }

        let is_mut: bool = if self.peek().kind == TokenType::Mut {
          self.advance();
          true
        } else {
          false
        };

        let param = self.consume(TokenType::Identifier)?;

        self.consume(TokenType::Colon)?;
        let token = self.advance();

        let mut data_type = DataType::from_token_type(token.kind);

        if self.check(TokenType::LeftBrack) {
          self.advance();
          self.consume(TokenType::RightBrack)?;

          data_type = DataType::Array(Box::new(data_type));
        }

        parameters.push(FunctionParameter::new(param, data_type, is_mut));

        if !self.match_token(&[TokenType::Comma]) {
          break;
        }
      }
    }

    self.consume(TokenType::RightParen)?;

    let return_type: Option<DataType>;
    if self.class_declarations.contains(&name.span.literal) {
      return_type = Some(DataType::ClassType(name.span.literal.clone()));
    } else {
      self.consume(TokenType::Colon)?;

      let token = self.peek();

      let mut data_type: DataType = DataType::from_token_type(token.kind.clone());

      if data_type == DataType::None && self.class_declarations.contains(&token.span.literal) {
        data_type = DataType::ClassType(token.span.literal.clone());
      }

      if data_type == DataType::None {
        let token = &self.peek();

        return Err(Box::new(
          ParserDiagnosticError::ExpectedReturnTypeAfterFunction(token.clone()),
        ));
      }

      self.advance();

      if self.check(TokenType::LeftBrack) {
        self.consume(TokenType::RightBrack)?;
        data_type = DataType::Array(Box::new(data_type));
      }

      return_type = Some(data_type);
    }

    let mut body: Vec<Statement> = Vec::new();

    if !self.match_token(&[TokenType::SemiColon]) {
      self.consume(TokenType::LeftBrace)?;

      body.push(self.block()?);
    }

    let metadata = MethodMetadata::new(
      is_public,
      false,
      self.class_declarations.contains(&name.span.literal),
    );

    Ok(Statement::Method(MethodStatement::new(
      name,
      parameters,
      body,
      return_type,
      if decorator.is_some() {
        vec![decorator.unwrap()]
      } else {
        vec![]
      },
      metadata,
    )))
  }

  fn class_declaration(&mut self) -> ParserResult<Statement> {
    let name: Token = self.consume(TokenType::Identifier)?;

    self.class_declarations.push(name.span.literal.clone());

    let mut methods: Vec<Statement> = Vec::new();
    let mut properties: Vec<Statement> = Vec::new();

    self.consume(TokenType::LeftBrace)?;

    while !self.check(TokenType::RightBrace) && !self.is_at_end() {
      let is_public = self.check(TokenType::Public);

      self.match_token(&[TokenType::Public, TokenType::Private]);

      if self.match_token(&[TokenType::Mut]) {
        let name = self.consume(TokenType::Identifier)?;
        properties.push(self.property_class_declaration(name, true, is_public)?);
        continue;
      }

      let name = self.consume(TokenType::Identifier)?;

      if self.match_token(&[TokenType::LeftParen]) {
        methods.push(self.method_declaration(name, is_public, None)?);
      } else if self.match_token(&[TokenType::Colon]) {
        properties.push(self.property_class_declaration(name, false, is_public)?);
      } else {
        return Err(Box::new(ParserDiagnosticError::ExpectedToken(
          TokenType::SemiColon,
          self.peek(),
        )));
      }
    }

    self.consume(TokenType::RightBrace)?;

    Ok(Statement::Class(Class::new(name, methods, properties)))
  }

  fn import_statement(&mut self) -> ParserResult<Statement> {
    self.consume(TokenType::LeftBrace)?;

    let mut symbols: Vec<ImportSymbol> = Vec::new();
    loop {
      if self.check(TokenType::Comma) {
        self.advance();
        continue;
      }

      if self.check(TokenType::RightBrace) {
        break;
      }

      let symbol_name = self.consume(TokenType::Identifier)?;

      let symbol = if self.check(TokenType::As) {
        self.advance();
        let alias = self.consume(TokenType::Identifier)?;
        Some(alias)
      } else {
        None
      };

      symbols.push(ImportSymbol::new(symbol_name, symbol));
    }

    self.consume(TokenType::RightBrace)?;

    self.consume(TokenType::From)?;
    let module_path = self.consume(TokenType::String)?;

    self.consume(TokenType::SemiColon)?;

    let is_std = module_path.span.literal.contains("std");
    let source = if is_std {
      ImportSource::StandardLibrary
    } else {
      ImportSource::FileSystem
    };

    Ok(Statement::Import(Import::new(
      module_path,
      symbols,
      is_std,
      source,
    )))
  }

  /*
   *  export function sum(a: int, b: int): int {
   *    return a + b;
   * }
   */
  fn export_statement(&mut self) -> ParserResult<Statement> {
    if !self.match_token(&[TokenType::Function]) {
      return Err(Box::new(ParserDiagnosticError::ExpectedToken(
        TokenType::Function,
        self.peek(),
      )));
    }

    self.context.push(ParserContext::Function);

    let function = self.function_statement(true, None)?;

    self.context.pop();

    Ok(function)
  }

  fn decoration_statement(&mut self) -> ParserResult<Statement> {
    match self.peek().kind {
      TokenType::Function => {
        self.context.push(ParserContext::Function);

        let function = self.function_statement(true, None)?;

        self.context.pop();
        Ok(function)
      }
      TokenType::Extern => {
        self.advance();
        self.consume(TokenType::LeftParen)?;

        let path = self.consume(TokenType::String)?;

        self.consume(TokenType::RightParen)?;

        let is_public = self.match_token(&[TokenType::Export]);

        self.consume(TokenType::Function)?;

        self.context.push(ParserContext::Function);

        let func = self.function_statement(is_public, Some(FunctionDecorator::Extern(path)))?;

        self.context.pop();

        Ok(func)
      }
      TokenType::Identifier => {
        todo!()
      }
      _ => todo!(),
    }
  }
}
