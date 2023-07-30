mod ast;

fn main() {
  let source = "(3+3.5);";

  let mut lexer = ast::lexer::Lexer::new(source);

  lexer.scan_tokens();

  for token in lexer.tokens {
    println!("{:?}", token);
  }
}
