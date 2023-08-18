use crate::diagnostic::{DiagnosticList, Diagnostic};

use super::{
  lexer::{token::Token, token_type::TokenType},
  expression::{
    Expression, binary::Binary, unary::Unary, literal::Literal, LiteralValue, grouping::Grouping,
    logical::Logical, assign::Assign, variable::VariableExpression, ternary, call::Call,
  },
  statement::{
    Statement, variable::Variable, expression::ExpressionStatement, if_statement::IfStatement,
    block::Block, while_statement::WhileStatement, function::FunctionStatement, return_statement,
  },
  data_type::DataType,
};

#[derive(Debug)]
pub enum ParserResult {
  Expression(Expression),
  Statement(Statement),
  Token(Token),
  Error,
}

#[derive(Debug)]
pub enum FunctionKind {
  Function,
  Method,
  Initializer,
  Lambda,
}

pub struct Parser {
  pub tokens: Vec<Token>,
  current: usize,
  pub diagnostics: DiagnosticList,
}

impl Parser {
  pub fn new(tokens: Vec<Token>) -> Self {
    Self {
      tokens,
      current: 0,
      diagnostics: DiagnosticList::new(),
    }
  }

  pub fn parse(&mut self) -> Vec<ParserResult> {
    let mut statements: Vec<ParserResult> = vec![];
    while !self.is_at_end() {
      statements.push(match self.declaration() {
        ParserResult::Statement(s) => ParserResult::Statement(s),
        _ => ParserResult::Error,
      });
    }

    statements
  }

  fn expression(&mut self) -> ParserResult {
    self.assignment()
  }

  // equelity -> comparison (("!=" | "==") comparison)*;
  fn equality(&mut self) -> ParserResult {
    let mut expression = match self.comparison() {
      ParserResult::Expression(e) => e,
      _ => return ParserResult::Error,
    };

    while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
      let operator: Token = self.previous();
      let right = match self.comparison() {
        ParserResult::Expression(e) => e,
        _ => return ParserResult::Error,
      };

      let data_type: DataType = DataType::Boolean;

      expression = Expression::Binary(Binary::new(
        Box::new(expression),
        operator,
        Box::new(right),
        data_type,
      ));
    }

    ParserResult::Expression(expression)
  }

  // comparison -> term ((">" | ">=" | "<" | "<=") term)*;
  fn comparison(&mut self) -> ParserResult {
    let mut expression = match self.term() {
      ParserResult::Expression(e) => e,
      _ => return ParserResult::Error,
    };

    while self.match_token(&[
      TokenType::Greater,
      TokenType::GreaterEqual,
      TokenType::Less,
      TokenType::LessEqual,
    ]) {
      let operator: Token = self.previous();
      let right = match self.term() {
        ParserResult::Expression(e) => e,
        _ => return ParserResult::Error,
      };

      let data_type: DataType = DataType::Boolean;

      expression = Expression::Binary(Binary::new(
        Box::new(expression),
        operator,
        Box::new(right),
        data_type,
      ));
    }

    ParserResult::Expression(expression)
  }

  // term -> factor (("-" | "+") factor)*;
  fn term(&mut self) -> ParserResult {
    let mut expression = match self.factor() {
      ParserResult::Expression(e) => e,
      _ => return ParserResult::Error,
    };

    while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
      let operator: Token = self.previous();
      let right = match self.factor() {
        ParserResult::Expression(e) => e,
        _ => return ParserResult::Error,
      };

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

    ParserResult::Expression(expression)
  }

  // factor -> ("!" | "-") unary | primary;
  fn factor(&mut self) -> ParserResult {
    let mut expression: Expression = match self.unary() {
      ParserResult::Expression(e) => e,
      _ => return ParserResult::Error,
    };

    while self.match_token(&[TokenType::Slash, TokenType::Asterisk]) {
      let operator: Token = self.previous();
      let right: Expression = match self.unary() {
        ParserResult::Expression(e) => e,
        _ => return ParserResult::Error,
      };

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

    ParserResult::Expression(expression)
  }

  // unary -> ("!" | "-") unary | call;
  fn unary(&mut self) -> ParserResult {
    if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
      let operator = self.previous();
      let right: Expression = match self.unary() {
        ParserResult::Expression(e) => e,
        _ => return ParserResult::Error,
      };

      let right_type = self.get_expression_type(&right);
      let operator_kind = operator.kind.clone();

      return ParserResult::Expression(Expression::Unary(Unary::new(
        operator,
        Box::new(right),
        self.get_data_type_by_operator(None, right_type, operator_kind),
      )));
    }

    self.call()
  }

  fn call(&mut self) -> ParserResult {
    let mut expression: Expression = match self.primary() {
      ParserResult::Expression(e) => e,
      _ => return ParserResult::Error,
    };

    loop {
      if self.match_token(&[TokenType::LeftParen]) {
        expression = match self.finish_call(expression) {
          ParserResult::Expression(e) => e,
          _ => return ParserResult::Error,
        };
      } else {
        break;
      }
    }

    ParserResult::Expression(expression)
  }

  fn primary(&mut self) -> ParserResult {
    let token = self.peek();

    match token.kind {
      TokenType::True
      | TokenType::False
      | TokenType::Null
      | TokenType::Int
      | TokenType::Double
      | TokenType::String => {
        self.advance();
        ParserResult::Expression(Expression::Literal(Literal::new(LiteralValue::from_token(
          token.clone(),
        ))))
      }
      TokenType::LeftParen => {
        self.advance();
        match self.expression() {
          ParserResult::Expression(e) => {
            match self.consume(TokenType::RightParen) {
              ParserResult::Token(_) => (),
              _ => {
                return ParserResult::Error;
              }
            }

            return ParserResult::Expression(Expression::Grouping(Grouping::new(Box::new(e))));
          }
          _ => return ParserResult::Error,
        };
      }
      TokenType::Let | TokenType::Const => {
        self.advance();
        return self.variable_declaration();
      }
      TokenType::Identifier => {
        self.advance();
        let kind = token.kind.clone();
        return ParserResult::Expression(Expression::Variable(VariableExpression::new(
          token,
          DataType::from_token_type(kind),
        )));
      }
      _ => {
        self.diagnostics.report_expected_expression(&token);
        return ParserResult::Error;
      }
    }
  }

  fn finish_call(&mut self, callee: Expression) -> ParserResult {
    let mut arguments: Vec<Expression> = Vec::new();

    if !self.check(TokenType::RightParen) {
      loop {
        if arguments.len() >= 255 {
          let token = &self.peek();

          self
            .diagnostics
            .report_invalid_number_of_arguments(255, arguments.len(), token);
        }

        arguments.push(match self.expression() {
          ParserResult::Expression(e) => e,
          _ => return ParserResult::Error,
        });

        if !self.match_token(&[TokenType::Comma]) {
          break;
        }
      }
    }

    let token = match self.consume(TokenType::RightParen) {
      ParserResult::Token(t) => t,
      _ => return ParserResult::Error,
    };

    ParserResult::Expression(Expression::Call(Call::new(
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
      (Some(DataType::Double), DataType::Double, TokenType::Plus)
      | (Some(DataType::Double), DataType::Double, TokenType::Minus)
      | (Some(DataType::Double), DataType::Double, TokenType::Slash)
      | (Some(DataType::Double), DataType::Double, TokenType::Asterisk)
      | (None, DataType::Double, TokenType::Minus) => DataType::Double,
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
        LiteralValue::Double(_) => DataType::Double,
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

  fn declaration(&mut self) -> ParserResult {
    if self.match_token(&[TokenType::Let, TokenType::Const]) {
      return self.variable_declaration();
    }

    if self.match_token(&[TokenType::Function]) {
      return self.function(FunctionKind::Function);
    }

    if self.match_token(&[TokenType::Return]) {
      // return self.return_statement();
    }

    if self.match_token(&[TokenType::While]) {
      return self.while_statement();
    }

    match self.statement() {
      ParserResult::Statement(s) => ParserResult::Statement(s),
      _ => {
        self.synchronize();
        ParserResult::Error
      }
    }
  }

  fn function(&mut self, kind: FunctionKind) -> ParserResult {
    match kind {
      FunctionKind::Function => {
        let name: Token = match self.consume(TokenType::Identifier) {
          ParserResult::Token(t) => t,
          _ => return ParserResult::Error,
        };

        match self.consume(TokenType::LeftParen) {
          ParserResult::Token(_) => (),
          _ => return ParserResult::Error,
        }

        let mut parameters: Vec<Token> = Vec::new();

        if !self.check(TokenType::RightParen) {
          loop {
            if parameters.len() >= 255 {
              // TODO: report error
            }

            let param = match self.consume(TokenType::Identifier) {
              ParserResult::Token(t) => t,
              _ => return ParserResult::Error,
            };

            parameters.push(param);

            if !self.match_token(&[TokenType::Comma]) {
              break;
            }
          }
        }

        match self.consume(TokenType::RightParen) {
          ParserResult::Token(_) => (),
          _ => return ParserResult::Error,
        };

        match self.consume(TokenType::Colon) {
          ParserResult::Token(_) => (),
          _ => return ParserResult::Error,
        };

        let mut return_type: Option<DataType> = None;
        if self.match_token(&[
          TokenType::Void,
          TokenType::Int,
          TokenType::Double,
          TokenType::String,
          TokenType::BooleanType,
          TokenType::Char,
        ]) {
          return_type = Some(DataType::from_token_type(self.peek().kind));
        } else {
          let token = &self.peek();

          self
            .diagnostics
            .report_expected_return_type_after_function(token);

          return ParserResult::Error;
        }

        match self.consume(TokenType::LeftBrace) {
          ParserResult::Token(_) => (),
          _ => return ParserResult::Error,
        };

        let body: Vec<Statement> = match self.block() {
          ParserResult::Statement(s) => match s {
            Statement::Block(b) => b.statements,
            _ => return ParserResult::Error,
          },
          _ => return ParserResult::Error,
        };

        ParserResult::Statement(Statement::FunctionStatement(FunctionStatement::new(
          name,
          parameters,
          body,
          return_type,
        )))
      }
      FunctionKind::Method => todo!(),
      FunctionKind::Initializer => todo!(),
      FunctionKind::Lambda => todo!(),
    }
  }

  fn block(&mut self) -> ParserResult {
    let mut statements: Vec<Statement> = Vec::new();

    while !self.check(TokenType::RightBrace) && !self.is_at_end() {
      statements.push(match self.declaration() {
        ParserResult::Statement(s) => s,
        _ => return ParserResult::Error,
      });
    }

    match self.consume(TokenType::RightBrace) {
      ParserResult::Token(_) => ParserResult::Statement(Statement::Block(Block::new(statements))),
      _ => ParserResult::Error,
    }
  }

  fn variable_declaration(&mut self) -> ParserResult {
    let mutable: bool = if self.peek().kind == TokenType::Mut {
      self.advance();
      true
    } else {
      false
    };

    let name: Token = match self.consume(TokenType::Identifier) {
      ParserResult::Token(t) => t,
      _ => return ParserResult::Error,
    };

    let mut initializer: Option<Expression> = None;

    let type_annotation: DataType = match self.consume(TokenType::Colon) {
      ParserResult::Token(_) => {
        let token = self.peek();

        let kind = DataType::from_token_type(token.kind.clone());

        if kind == DataType::None {
          return ParserResult::Error;
        }

        kind
      }
      _ => return ParserResult::Error,
    };

    self.advance();

    if self.match_token(&[TokenType::Equal]) {
      initializer = match self.expression() {
        ParserResult::Expression(e) => Some(e),
        _ => return ParserResult::Error,
      };
    }

    match self.consume(TokenType::SemiColon) {
      ParserResult::Token(_) => (),
      _ => return ParserResult::Error,
    };

    if let Some(ini) = initializer {
      ParserResult::Statement(Statement::Variable(Variable::new(
        Box::new(name),
        Some(Box::new(ini)),
        Box::new(type_annotation),
        mutable,
      )))
    } else {
      let token = self.peek();
      self.diagnostics.report_expected_expression(&token);

      return ParserResult::Error;
    }
  }

  // statement -> expressionStatement | ifStatement;
  fn statement(&mut self) -> ParserResult {
    if self.match_token(&[TokenType::LeftBrace]) {
      return self.block();
    }

    if self.match_token(&[TokenType::If]) {
      return self.if_statement();
    }

    self.expression_statement()
  }

  // expressionStatement -> expression ";";
  fn expression_statement(&mut self) -> ParserResult {
    let resutl = self.expression();
    let expression: Expression;

    match resutl {
      ParserResult::Expression(e) => expression = e,
      _ => return ParserResult::Error,
    }

    match self.consume(TokenType::SemiColon) {
      ParserResult::Token(_) => ParserResult::Statement(Statement::Expression(
        ExpressionStatement::new(Box::new(expression)),
      )),
      _ => ParserResult::Error,
    }
  }

  fn assignment(&mut self) -> ParserResult {
    let mut expression: Expression = match self.ternary() {
      ParserResult::Expression(e) => e,
      _ => return ParserResult::Error,
    };

    if self.match_token(&[TokenType::Equal]) {
      let equals: Token = self.previous();
      let value: Expression = match self.assignment() {
        ParserResult::Expression(e) => e,
        _ => return ParserResult::Error,
      };

      if let Expression::Variable(variable) = expression {
        expression = Expression::Assign(Assign::new(
          variable.name,
          Box::new(value),
          variable.data_type,
        ));
      } else {
        self
          .diagnostics
          .report_invalid_assignment_target(&equals.span);

        return ParserResult::Error;
      }
    }

    return ParserResult::Expression(expression);
  }

  fn ternary(&mut self) -> ParserResult {
    let mut children: Vec<Expression> = match self.or_expression() {
      ParserResult::Expression(e) => vec![e],
      _ => return ParserResult::Error,
    };

    while self.match_token(&[TokenType::QuestionMark]) {
      match self.expression() {
        ParserResult::Expression(e) => children.push(e),
        _ => return ParserResult::Error,
      }

      match self.consume(TokenType::Colon) {
        ParserResult::Token(_) => (),
        _ => return ParserResult::Error,
      }

      match self.expression() {
        ParserResult::Expression(e) => children.push(e),
        _ => return ParserResult::Error,
      }
    }

    if children.len() == 1 {
      return ParserResult::Expression(children.pop().unwrap());
    }

    let else_branch = children.pop().unwrap();
    let then_branch = children.pop().unwrap();
    let condition = children.pop().unwrap();

    let mut expression: Expression = Expression::Ternary(ternary::Ternary::new(
      Box::new(condition),
      Box::new(then_branch),
      Box::new(else_branch),
      DataType::Pending,
    ));

    while !children.is_empty() {
      expression = Expression::Ternary(ternary::Ternary::new(
        Box::new(children.pop().unwrap()),
        Box::new(expression),
        Box::new(children.pop().unwrap()),
        DataType::Pending,
      ));
    }

    ParserResult::Expression(expression)
  }

  fn or_expression(&mut self) -> ParserResult {
    let result = self.and_expression();
    let mut expression: Expression;

    match result {
      ParserResult::Expression(e) => expression = e,
      _ => return ParserResult::Error,
    }

    while self.match_token(&[TokenType::Or]) {
      let operator: Token = self.previous();
      let right_result = self.and_expression();

      match right_result {
        ParserResult::Expression(e) => {
          expression =
            Expression::Logical(Logical::new(Box::new(expression), operator, Box::new(e)));
        }
        _ => return ParserResult::Error,
      }
    }

    ParserResult::Expression(expression)
  }

  fn and_expression(&mut self) -> ParserResult {
    let result = self.equality();
    let mut expression: Expression;

    match result {
      ParserResult::Expression(e) => expression = e,
      _ => return ParserResult::Error,
    }

    while self.match_token(&[TokenType::And]) {
      let operator: Token = self.previous();
      let right_result = self.equality();

      match right_result {
        ParserResult::Expression(e) => {
          expression =
            Expression::Logical(Logical::new(Box::new(expression), operator, Box::new(e)));
        }
        _ => return ParserResult::Error,
      }
    }

    ParserResult::Expression(expression)
  }

  fn while_statement(&mut self) -> ParserResult {
    match self.consume(TokenType::LeftParen) {
      ParserResult::Token(_) => (),
      _ => return ParserResult::Error,
    }

    let condition: Expression = match self.expression() {
      ParserResult::Expression(e) => e,
      _ => return ParserResult::Error,
    };

    match self.consume(TokenType::RightParen) {
      ParserResult::Token(_) => (),
      _ => return ParserResult::Error,
    }

    let body: Statement = match self.statement() {
      ParserResult::Statement(s) => s,
      _ => return ParserResult::Error,
    };

    ParserResult::Statement(Statement::WhileStatement(WhileStatement::new(
      Box::new(condition),
      Box::new(body),
    )))
  }

  fn if_statement(&mut self) -> ParserResult {
    match self.consume(TokenType::LeftParen) {
      ParserResult::Token(_) => (),
      _ => return ParserResult::Error,
    }

    let condition: Expression = match self.expression() {
      ParserResult::Expression(e) => e,
      _ => return ParserResult::Error,
    };

    match self.consume(TokenType::RightParen) {
      ParserResult::Token(_) => (),
      _ => return ParserResult::Error,
    }

    let then_branch: Statement = match self.statement() {
      ParserResult::Statement(s) => s,
      _ => return ParserResult::Error,
    };

    let else_branch: Option<Statement> = if self.match_token(&[TokenType::Else]) {
      let else_statement: Statement = match self.statement() {
        ParserResult::Statement(s) => s,
        _ => return ParserResult::Error,
      };

      Some(else_statement)
    } else {
      None
    };

    ParserResult::Statement(Statement::IfStatement(IfStatement::new(
      Box::new(condition),
      Box::new(then_branch),
      else_branch.map(|s| Box::new(s)),
    )))
  }

  fn consume(&mut self, kind: TokenType) -> ParserResult {
    let token: Token = self.peek();
    if token.kind == kind {
      return ParserResult::Token(self.advance());
    }

    match kind {
      TokenType::SemiColon => {
        self
          .diagnostics
          .report_unexpected_token(&TokenType::SemiColon, &token);
      }
      TokenType::Colon => {
        self
          .diagnostics
          .report_unexpected_token(&TokenType::Colon, &token);
      }
      TokenType::Identifier => {
        self.diagnostics.report_expected_variable_name(&token);
      }
      TokenType::QuestionMark => {
        self
          .diagnostics
          .report_expected_token(&TokenType::QuestionMark, &token);
      }
      TokenType::LeftParen | TokenType::RightParen => {
        let expression = self.previous();

        self
          .diagnostics
          .report_expected_after_expression(&kind, &expression, &token);
      }
      _ => {
        self.diagnostics.report_unexpected_token(&kind, &token);
      }
    }

    return ParserResult::Error;
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
}
