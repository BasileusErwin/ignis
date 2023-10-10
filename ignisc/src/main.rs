use std::{
  io::{self, Write, BufRead},
  env,
  process::exit,
  fs,
};

use analyzer::Analyzer;
use parser::Parser;
use lexer::Lexer;
use ast::Ast;
use to_lua::TranspilerToLua;
use diagnostic::{DiagnosticList, error::DiagnosticError};

struct CodeResult {
  pub code: String,
  pub file_name: String,
}

impl CodeResult {
  pub fn new(code: String, file_name: String) -> Self {
    Self { code, file_name }
  }
}

fn display_diagnostic(diagnostics: &DiagnosticList, relp: bool) {
  for diagnostic in diagnostics.diagnostics.iter() {
    println!("- {}", diagnostic.module_path.as_ref().unwrap());
    println!("{}: {}", diagnostic.code, diagnostic.hint.as_ref().unwrap());
    if !relp {
      println!("{} | {}", diagnostic.span.line, diagnostic.span.literal);
      println!("Column: {}", diagnostic.span.end - diagnostic.span.start);
    }
  }
}

fn run_file(path: &str) -> Result<(), ()> {
  match fs::read_to_string(path) {
    Ok(content) => match run(content, path.to_string(), false) {
      Ok(result) => {
        for code_result in result {
          let mut path = code_result.file_name.split("/").collect::<Vec<&str>>();
          let code = code_result.code.clone();

          let mut name = path.last().unwrap().replace(r".ign", "");

          name.push_str(".lua");
          path.pop();

          let mut build_path = "build/".to_string() + path.join("/").as_str();

          fs::create_dir_all(build_path.clone()).unwrap();

          build_path.push_str(format!("/{}", &name).as_str());

          fs::write(build_path, code).unwrap();
        }

        return Ok(());
      }
      Err(_) => {
        return Err(());
      }
    },
    Err(e) => {
      println!("{:?}", e);
      Err(())
    }
  }
}

fn run(
  source: String,
  module_path: String,
  relp: bool,
) -> Result<Vec<CodeResult>, ()> {
  let mut lexer: Lexer<'_> = Lexer::new(&source, module_path.clone());
  lexer.scan_tokens();

  // lexer.display_lexer();

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

  // let pretty_string = serde_json::to_string_pretty(&ast.to_json()).unwrap();
  // println!("{}", pretty_string);

  let mut analyzer = Analyzer::new(module_path.clone());

  analyzer.analyze(&mut ast.statements);

  for diagnostic in &analyzer.diagnostics {
    DiagnosticError::report(
      &DiagnosticError::from_evaluator_error(diagnostic.clone()),
      &mut diagnostics,
    );
  }

  // for block in &analyzer.block_stack {
  //   display_block(&block.clone(), "Block", 1);
  // }

  // for result in &analyzer.irs {
  //   println!("IR for {}", result.0);
  //   for ir in result.1 {
  //     display_ir(ir, 1);
  //   }
  // }

  let mut generator = bytecode_generator::BytecodeGenerator::new();
  
  generator.generate(analyzer.irs.clone());
  
  let debug = bytecode_generator::debug::BytecodeDebug::new(generator.bytecodes);
  debug.print_bytecode();

  let mut transpiler = TranspilerToLua::new();
  let mut code_results: Vec<CodeResult> = vec![];

  for result in analyzer.irs.iter() {
    transpiler.transpile(result.1);

    code_results.push(CodeResult::new(transpiler.code.clone(), result.0.clone()));
  }

  // visit(ast.statements, &mut diagnostics, evaluator);

  if diagnostics.diagnostics.len() > 0 {
    display_diagnostic(&diagnostics, relp);

    if !relp {
      exit(1);
    }
  }

  diagnostics.clean_diagnostic();

  return Ok(code_results);
}

fn run_prompt() -> Result<(), String> {
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

    match run(buffer, "".to_string(), false) {
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
