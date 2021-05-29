use crate::scanner::*;

pub fn compile(source: &String) {
  let mut scanner = Scanner::new(source);

  let mut line = -1;
  while let Some(token) = scanner.scan_token() {
    if token.line != line {
      line = token.line;
      print!("{:4} ", line);
    } else {
      print!("   | ");
    }
    println!("{:2} '{}'", token.kind as u8, token.lexeme);
  }
}
