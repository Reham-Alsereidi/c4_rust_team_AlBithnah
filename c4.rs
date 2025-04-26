use std::env;
use std::fs;
use std::process;

type Int=i64;

//Token types
#[allow(dead_code)]
enum TokenType {
  Num=128,
  Fun,
  Sys,
  Glo,
  Loc,
  Id,
  Char,
  Else,
  Enum,
  If,
  Int,
  Return,
  Sizeof,
  While,
  Assign,
  Cond,
  Lor,
  Lan,
  Or,
  Xor,
  And,
  Eq,
  Ne,
  Lt,
  Gt,
  Le,
  Ge,
  Shl,
  Shr,
  Add,
  Sub,
  Mul,
  Div,
  Mod,
  Inc,
  Dec,
  Brak,
}

//VM instruction opcodes
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum OpCode {
  LEA, IMN, JMP, JSR, BZMBNZ,ENT, ADJ, LEV, LI, LC, SI, SC, PSH,OR, XOR, AND, EQ, NE, LT, GT, LE, GE,
  SHL, SHR, ADD, SUB, MUL, DIV, MOD, OPEN, READ, CLOS, PRTF, MALC, FREE, MSET, MCMP, EXIT, FUN
}

//Types
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum Type {
  CHAR = 0,
  INT = 1,
  PTR = 2,
}

#[derive(Debug, Clone)]
struct Symbol {
  token: i32,              // Token type
  hash: i32,               // Hash value
  name: String,            // Symbol name
  class: i32,              // Storage class (Glo, Loc, etc)
  type_: i32,              // Data type
  value: Int,              // Value
  // Fields for local symbol handling
  #[allow(dead_code)]
  h_class: i32,
  #[allow(dead_code)]
  h_type: i32,
  #[allow(dead_code)]
  h_val: Int,
}

#[allow(dead_code)]
struct C4 {
  p: usize,
  lp: usize,
  source: String,
  e: Vec<Int>,
  le: usize,
  symbols: Vec<Symbol>,
  token: i32,
  token_val: Int,
  #[allow(deaad_code)]
  type_: i32,
  loc: Int,
  line: i32,
  src: bool,
  debug: bool,
  data: Vec<u8>,
  data_index: usize,
  id: usize,
  cycle: i32,
}

//Implementation of the compiler
#[allow(dead_code)]
impl C4 {
  fn new() -> Self {
    C4 {
      p: 0,
      lp: 0,
      source: String::new(),
      e: vec![0; 256*1024],
      le: 0,
      symbols: Vec::new(),
      token: 0,
      token_val: 0,
      type_: 0,
      loc: 0,
      line: 1,
      src: false,
      debug: false,
      data: vec![0; 256*1024],
      data_index: 0,
      id: 0,
      cycle: 0,
    }
  }

  //Get current character
  fn current_char(&self) -> char{
    if self.p < self.source.len(){
      self.source.chars().nth(self.p).unwrap_or('\0')
    } else {
      '\0'
    }
  }

  //Advance to next character
  #[allow(dead_code)]
  fn next_char(&mut self) -> char{
    self.p +=1;
    self.current_char()
  }

  //Symbol table with keywords and system calls
  fn init_symbol_table(&mut self){
    //Add keywords
    let keywords = [
      ("char", TokenType::Char as i32),
      ("else", TokenType::Else as i32),
      ("enum", TokenType::Enum as i32),
      ("if", TokenType::If as i32),
      ("int", TokenType::Int as i32),
      ("return", TokenType::Return as i32),
      ("sizeof", TokenType::Sizeof as i32),
      ("while", TokenType::While as i32),
    ];

    for (word, token) in keywords {
      self.add_keyword(word, token);
    }

    //Add system calls
    let syscalls = [
      ("open", OpCode::OPEN as i32),
      ("read", OpCode::READ as i32),
      ("close", OpCode::CLOS as i32),
      ("printf", OpCode::PRTF as i32),
      ("malloc", OpCode::MALC as i32),
      ("free", OpCode::FREE as i32),
      ("memset", OpCode::MSET as i32),
      ("memcmp", OpCode::MCMP as i32),
      ("exit", OpCode::EXIT as i32),
    ];

    for (name, code) in syscalls {
      self.add_syscall(name, code);
    }

    self.add_keyword("void", TokenType::Char as i32);
  }

  fn add_keyword(&mut self, name: &str, token: i32){
    let mut hash: i32 = 0;

    for c in name.chars() {
      hash = hash.wrapping_mul(147).wrapping_add(c as i32);
    }
    hash = (hash << 6).wrapping_add(name.len() as i32);

    self.symbols.push(Symbol {
      token,
      hash,
      name: name.to_string(),
      class: 0,
      type_: 0,
      value: 0,
      h_class: 0,
      h_type: 0,
      h_val: 0,
    });
  }

  fn add_syscall(&mut self, name: &str, token: i32) {
    let mut hash: i32 = 0;

    for c in name.chars() {
      hash = hash.wrapping_mul(147).wrapping_add(c as i32);
    }
    hash = (hash << 6).wrapping_add(name.len() as i32);
    
    self.symbols.push(Symbol {
      token: TokenType::Id as i32,
      hash,
      name: name.to_string(),
      class: TokenType::Sys as i32,
      type_: Type::INT as i32,
      value: code as Int,
      h_class: 0,
      h_type: 0,
      h_val: 0,
    });
  }

  fn find_symbol(&self, hash: i32, name: &str) -> Option<usize> {
    for (i, sym) in self.symbol.iter().enumerate() {
      if sym.hash == hash && sym.name == name {
        return Some(i);
      }
    }
    None
  }

  //Next token lexer function
  fn next(&mut self) {
    self.token = 0;

    while self.p < self.source.len() {
      let ch = self.current_char();

      if ch == '\n' {
        self.line += 1
        if self.src {
          //Print source line and assembly
          let line_end = self.source[self.lp..self.p].find('\n').map_or(self.source, |pos| self.lp + pos + 1);
          print!("{}: {}", self.line-1, &self.source[self.lp..line_end]);
        }
        self.lp = self.p +1;
        self.p += 1;
        continue;
      } else if ch.is_whitespace() {
        self.p += 1;
        continue;
      }
      break;
    }
    
    if self.p < self.source.len(){
      println!("Next token starts with character: '{}' at position {}", self.current_char(), self.p);
    } else {
      println!("Reached end of source");
      return;
    }

    let ch = self.current_char();

    //Parse identifiers
  }
}
