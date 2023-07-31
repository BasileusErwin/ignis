mod ast;

fn main() {
  let source = "let helloWorld = \"Hello World\";";

  println!("Source: {}", &source);
  let mut lexer = ast::lexer::Lexer::new(source);

  lexer.scan_tokens();

  for token in lexer.tokens {
    println!("{:?}", token);
  }
}
