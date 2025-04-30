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
  #[allow(dead_code)]
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

  fn add_syscall(&mut self, name: &str, code: i32) {
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
    for (i, sym) in self.symbols.iter().enumerate() {
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
        self.line += 1;
        if self.src {
          // Print source line and assembly
          let line_end = self.source[self.lp..self.p].find('\n')
            .map_or(self.p, |pos| self.lp + pos + 1);
          print!("{}: {}", self.line - 1, &self.source[self.lp..line_end]);
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
    if ch.is_alphabetic() || ch=='_'{
      let start = self.p;
      let mut hash: i32 = ch as i32;
      self.p +=1;

      //Collect identifiers characters
      while self.p < self.source.len() {
        let ch = self.current_char();
        if ch.is_alphabetic() || ch=='_'{
          hash = hash.wrapping_mul(147).wrapping_add(ch as i32);
          self.p +=1;
        } else {
          break;
        }
      }
      
      //Calculating Hash
      hash = (hash<<6).wrapping_add((self.p - start) as i32);
      let name = &self.source[start..self.p];
      if let Some(idx) = self.find_symbol(hash, name) {
        self.token = self.symbols[idx].token;
        self.id = idx;
      } else {
        self.id = self.symbols.len();
        self.symbols.push(Symbol {
          token: TokenType::Id as i32,
          hash,
          name: name.to_string(),
          class: 0,
          type_: 0,
          value: 0,
          h_class: 0,
          h_type: 0,
          h_val: 0,
        });
        self.token = TokenType::Id as i32;
      }
      
      println!("Parsed identifier: '{}', token = {}, id={}", name, self.token, self.id);
      return;
    }
    
    //Parse numbers
    if ch.is_digit(10) {
      let is_zero = ch == '0';
      self.token_val = (ch as u8 - b'0') as Int;
      self.p +=1;
      
      if is_zero && self.p < self.source.len() {
        let next_ch = self.current_char();
        
        if next_ch == 'x' || next_ch == 'X' {
          self.p += 1;
          self.token_val = 0;
          while self.p < self.source.len() {
            let ch = self.current_char();
            if ch.is_digit(16) {
              let digit_val = if ch.is_digit(10) {
                ch as u8 - b'0'
              } else if ch >= 'a' && ch <= 'f' {
                (ch as u8 - b'a') + 10
              } else {
                (ch as u8 - b'A') + 10
              };
              self.token_val = self.token_val * 16 + digit_val as Int;
              self.p += 1;
            } else {
              break;
            }
          }
        }
        else if next_ch.is_digit(8) {
          while self.p < self.source.len(){
            let ch = self.current_char();
            if ch.is_digit(8) {
              self.token_val = self.token_val * 8 + (ch as u8 - b'0') as Int;
              self.p += 1;
            } else {
              break;
            }
          }
        }
      }
      // Handle decimal numbers
      else if !is_zero {
        while self.p < self.source.len() {
          let ch = self.current_char();
          if ch.is_digit(10) {
            self.token_val = self.token_val * 10 + (ch as u8 - b'0') as Int;
            self.p += 1;
          } else {
            break;
          }
        }
      }
      self.token = TokenType::Num as i32;
      return;
    }
    
    //Handle string and character literals
    if ch == '"' || ch == '\'' {
      let string_type = ch;
      let data_start = self.data_index;
      self.p += 1;
      
      while self.p < self.source.len() && self.current_char() != string_type {
        let mut val = self.current_char() as i32;
        self.p += 1;
        if val == '\\' as i32 && self.p < self.source.len() {
          val = self.current_char() as i32;
          self.p += 1;
          if val == 'n' as i32 {
            val = '\n' as i32;
          }
        }
        
        if string_type == '"' {
          self.data[self.data_index] = val as u8;
          self.data_index += 1;
        }
      }
      
      if self.p < self.source.len() {
        self.p += 1;
      }
      
      if string_type == '"' {
        self.token_val = data_start as Int;
        // Align data pointer
        self.data_index = (self.data_index + std::mem::size_of::<Int>() - 1) & !(std::mem::size_of::<Int>() - 1);
      } else {
        self.token = TokenType::Num as i32;
      }
      return;
    }
    
    // Handle operators and other tokens
    match ch {
      '/' => {
        self.p += 1;
        if self.current_char() == '/' {
          // Line comment
          self.p += 1;
          while self.p < self.source.len() && self.current_char() != '\n' {
            self.p += 1;
          }
          self.next(); 
          return;
        }
        self.token = TokenType::Div as i32;
      },
      '=' => {
        self.p += 1;
        if self.current_char() == '=' {
          self.p += 1;
          self.token = TokenType::Eq as i32;
        } else {
          self.token = TokenType::Assign as i32;
        }
      },
      '+' => {
        self.p += 1;
        if self.current_char() == '+' {
          self.p += 1;
          self.token = TokenType::Inc as i32;
        } else {
          self.token = TokenType::Add as i32;
        }
      },
      '-' => {
        self.p += 1;
        if self.current_char() == '-' {
          self.p += 1;
          self.token = TokenType::Dec as i32;
        } else {
          self.token = TokenType::Sub as i32;
        }
      },
      '!' => {
        self.p += 1;
        if self.current_char() == '=' {
          self.p += 1;
          self.token = TokenType::Ne as i32;
        } else {
          self.token = '!' as i32;
        }
      },
      '<' => {
        self.p += 1;
        if self.current_char() == '=' {
          self.p += 1;
          self.token = TokenType::Le as i32;
        } else if self.current_char() == '<' {
          self.p += 1;
          self.token = TokenType::Shl as i32;
        } else {
          self.token = TokenType::Lt as i32;
        }
      },
      '>' => {
        self.p += 1;
        if self.current_char() == '=' {
          self.p += 1;
          self.token = TokenType::Ge as i32;
        } else if self.current_char() == '>' {
          self.p += 1;
          self.token = TokenType::Shr as i32;
        } else {
          self.token = TokenType::Gt as i32;
        }
      },
      '|' => {
        self.p += 1;
        if self.current_char() == '|' {
          self.p += 1;
          self.token = TokenType::Lor as i32;
        } else {
          self.token = TokenType::Or as i32;
        }
      },
      '&' => {
        self.p += 1;
        if self.current_char() == '&' {
          self.p += 1;
          self.token = TokenType::Lan as i32;
        } else {
          self.token = TokenType::And as i32;
        }
      },
      '^' => {
        self.p += 1;
        self.token = TokenType::Xor as i32;
      },
      '%' => {
        self.p += 1;
        self.token = TokenType::Mod as i32;
      },
      '*' => {
        self.p += 1;
        self.token = TokenType::Mul as i32;
      },
      '[' => {
        self.p += 1;
        self.token = TokenType::Brak as i32;
      },
      '?' => {
        self.p += 1;
        self.token = TokenType::Cond as i32;
      },
      '#' => {
        self.p += 1;
        while self.p < self.source.len() && self.current_char() != '\n' {
          self.p += 1;
        }
        self.next(); // next token
        return;
      },
      '~' | ';' | '{' | '}' | '(' | ')' | ']' | ',' | ':' => {
        self.token = ch as i32;
        self.p += 1;
      },
      _ => {
        self.token = ch as i32;
        self.p += 1;
      }
    }
  }
}
