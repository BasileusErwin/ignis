use std::{
  io::{self, Write, BufRead},
  env,
  process::exit,
  fs,
};

use analyzer::{
  Analyzer,
  debug::{display_ir, display_block},
  ir::instruction::{block::IRBlock, IRInstruction},
  AnalyzerResult,
  analyzer_error::AnalyzerDiagnosticError,
  analyzer_value::AnalyzerValue,
};
use parser::Parser;
use lexer::Lexer;
use ast::{Ast, statement::Statement, visitor::Visitor};
use diagnostic::{DiagnosticList, error::DiagnosticError};
use evaluator::{
  Evaluator, EvaluatorResult, evaluator_value::EvaluatorValue, execution_error::ExecutionError,
};

fn display_diagnostic(diagnostics: &DiagnosticList, relp: bool) {
  for diagnostic in diagnostics.diagnostics.iter() {
    println!("{}: {}", diagnostic.code, diagnostic.hint.as_ref().unwrap());
    if relp {
      println!("{} | {}", diagnostic.span.line, diagnostic.span.literal);
      println!("Column: {}", diagnostic.span.end - diagnostic.span.start);
    }
  }
}

fn run_file(path: &str) -> Result<(), ()> {
  let mut evaluator = Evaluator::new();

  match fs::read_to_string(path) {
    Ok(content) => run(content, &mut evaluator, true),
    Err(e) => {
      println!("{:?}", e);
      Err(())
    }
  }
}

fn run(source: String, evaluator: &mut Evaluator, relp: bool) -> Result<(), ()> {
  let mut lexer: Lexer<'_> = Lexer::new(&source);
  lexer.scan_tokens();

  // for token in &lexer.tokens {
  //   println!("{:?}", token);
  // }

  let mut parser = Parser::new(lexer.tokens);
  let parser_result = parser.parse();

  let mut diagnostics = DiagnosticList::new();

  let mut ast: Ast = match parser_result {
    Ok(statements) => Ast::new(statements),
    Err(_) => {
      DiagnosticError::from_parser_diagnostic(parser.diagnostics)
        .iter()
        .for_each(|error| {
          error.report(&mut diagnostics);
        });
      display_diagnostic(&diagnostics, relp);
      return Err(());
    }
  };

  let mut analyzer = Analyzer::new();

  // for result in &ast.statements {
  //   println!("{:?}", result);
  // }

  analyzer.analyze(&mut ast.statements);

  for diagnostic in &analyzer.diagnostics {
    DiagnosticError::report(
      &DiagnosticError::from_evaluator_error(diagnostic.clone()),
      &mut diagnostics,
    );
  }

  for block in &analyzer.block_stack {
    display_block(&block.clone(), "Block", 1);
  }

  for ir in &analyzer.irs {
    display_ir(ir, 1);
  }

  // visit(ast.statements, &mut diagnostics, evaluator);

  if diagnostics.diagnostics.len() > 0 {
    display_diagnostic(&diagnostics, relp);
  }

  diagnostics.clean_diagnostic();

  Ok(())
}

fn run_prompt() -> Result<(), String> {
  let mut evaluator = Evaluator::new();

  loop {
    print!("(ignis) > ");

    match io::stdout().flush() {
      Ok(_) => (),
      Err(_) => return Err("Could not flush stdout".to_string()),
    }
    let mut buffer = String::new();
    let mut handler = io::stdin().lock();

    match handler.read_line(&mut buffer) {
      Ok(n) => {
        if n == 0 {
          println!("");
          return Ok(());
        } else if n == 1 {
          continue;
        }
      }
      Err(_) => return Err("Clound not read line".to_string()),
    }

    if buffer.trim() == String::from("exit") {
      println!("Bye!");
      exit(0);
    }

    if buffer.contains("load") == true {
      let path = buffer.split("load").collect::<Vec<&str>>()[1]
        .trim()
        .to_string();
      match run_file(&path) {
        Ok(_) => (),
        Err(_) => println!("Could not import file"),
      }
      continue;
    }

    match run(buffer, &mut evaluator, false) {
      Ok(_) => (),
      Err(()) => (),
    }
  }
}

fn main() {
  let args: Vec<String> = env::args().collect();

  if args.len() == 2 {
    let _ = run_file(&args[1]);
  } else {
    match run_prompt() {
      Ok(_) => exit(0),
      Err(msg) => {
        println!("ERROR\n{}", msg);
        exit(1);
      }
    }
  }
}
