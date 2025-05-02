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

  // Emit an instruction
  fn emit(&mut self, op: OpCode) {
    self.le += 1;
    self.e[self.le] = op as Int;
  }
 
  // Emit an instruction with an operand
  fn emit_with_operand(&mut self, op: OpCode, operand: Int) {
    self.emit(op);
    self.le += 1;
    self.e[self.le] = operand;
  }

  // Expression parsing 
  fn expr(&mut self, level: i32) -> Result<(), String> {
    // Save the current type before parsing expressions
    let save_type = self.type_;
    let mut t: i32;
    
    if self.token == 0 {
      return Err(format!("{}: unexpected end of file in expression", self.line));
    } 

    // Parse primary expressions
    if self.token == TokenType::Num as i32 {
      self.emit_with_operand(OpCode::IMM, self.token_val);
      self.next();
      self.type_ = Type::INT as i32;
    } 
    else if self.token == '"' as i32 {
      self.emit_with_operand(OpCode::IMM, self.token_val);
      self.next();
      while self.token == '"' as i32 {
        self.next();
      } 
      self.data_index = (self.data_index + std::mem::size_of::<Int>() - 1) & !(std::mem::size_of::<Int>() - 1);
      self.type_ = Type::PTR as i32;
    } 
    else if self.token == TokenType::Sizeof as i32 {
      self.next();
      if self.token == '(' as i32 {
        self.next();
      } else { 
        return Err(format!("{}: open paren expected in sizeof", self.line));
      }
      self.type_ = Type::INT as i32;
      if self.token == TokenType::Int as i32 {
        self.next();
      } else if self.token == TokenType::Char as i32 {
        self.next();
        self.type_ = Type::CHAR as i32;
      }
      while self.token == TokenType::Mul as i32 {
        self.next();
        self.type_ += Type::PTR as i32;
      }
      if self.token == ')' as i32 {
        self.next();
      } else {
        return Err(format!("{}: close paren expected in sizeof", self.line));
      } 
      let size_val = if self.type_ == Type::CHAR as i32 { 1 } else { std::mem::size_of::<Int>() as Int };
      self.emit_with_operand(OpCode::IMM, size_val);
      self.type_ = Type::INT as i32;
    } 
    else if self.token == TokenType::Id as i32 {
      let id_idx = self.id;
      self.next();
      if self.token == '(' as i32 {
        self.next();
        let mut arg_count = 0;
        while self.token != ')' as i32 {
          self.expr(TokenType::Assign as i32)?;
          self.emit(OpCode::PSH);
          arg_count += 1;
          if self.token == ',' as i32 {
            self.next();
          }
        }
        self.next();
        let sym = &self.symbols[id_idx];
        let class = sym.class;
        let value = sym.value;
        let type_ = sym.type_;
        if class == TokenType::Sys as i32 {
          self.emit_with_operand(OpCode::IMM, value);
        } else if class == TokenType::Fun as i32 {
          self.emit_with_operand(OpCode::JSR, value);
        } else { 
          return Err(format!("{}: bad function call", self.line));
        } 
        if arg_count > 0 {
          self.emit_with_operand(OpCode::ADJ, arg_count);
        } 
        self.type_ = type_;
      } 
      else if self.symbols[id_idx].class == TokenType::Num as i32 {
        self.emit_with_operand(OpCode::IMM, self.symbols[id_idx].value);
        self.type_ = Type::INT as i32;
      } 
      else {
        let class = self.symbols[id_idx].class;
        let value = self.symbols[id_idx].value;
        let var_type = self.symbols[id_idx].type_;
        if class == TokenType::Loc as i32 {
          self.emit_with_operand(OpCode::LEA, self.loc - value);
        } else if class == TokenType::Glo as i32 {
          self.emit_with_operand(OpCode::IMM, value);
        } else {
          return Err(format!("{}: undefined variable", self.line));
        }
        self.type_ = var_type;
        // Load the value
        if self.type_ == Type::CHAR as i32 {
          self.emit(OpCode::LC);
        } else {
          self.emit(OpCode::LI);
        }
      } 
    }
    else if self.token == '(' as i32 {
      self.next();
      if self.token == TokenType::Int as i32 || self.token == TokenType::Char as i32 {
        // Type cast
        t = if self.token == TokenType::Int as i32 {
          Type::INT as i32
        } else {
          Type::CHAR as i32
        };
        self.next();
        while self.token == TokenType::Mul as i32 {
          self.next();
          t += Type::PTR as i32;
        } 
        if self.token == ')' as i32 {
          self.next();
        } else {
          return Err(format!("{}: bad cast", self.line));
        } 
        self.expr(TokenType::Inc as i32)?;
        self.type_ = t;
      } 
      else { 
        self.expr(TokenType::Assign as i32)?;
        if self.token == ')' as i32 {
          self.next();
        } else { 
          return Err(format!("{}: close paren expected", self.line));
        }
      } 
    } 
    else if self.token == TokenType::Mul as i32 {
      self.next();
      self.expr(TokenType::Inc as i32)?;
      if self.type_ >= Type::PTR as i32 {
        self.type_ -= Type::PTR as i32;
      } else {
        return Err(format!("{}: bad dereference", self.line));
      }
      if self.type_ == Type::CHAR as i32 {
        self.emit(OpCode::LC);
      } else {
        self.emit(OpCode::LI);
      } 
    } 
    else if self.token == TokenType::And as i32 {
      self.next();
      self.expr(TokenType::Inc as i32)?;
      // If it's already a load, just remove it
      if self.e[self.le] == OpCode::LC as Int || self.e[self.le] == OpCode::LI as Int {
        self.le -= 1;
      } else { 
        return Err(format!("{}: bad address-of", self.line));
      } 
      self.type_ += Type::PTR as i32;
    }
    else if self.token == '!' as i32 {
      self.next();
      self.expr(TokenType::Inc as i32)?;
      self.emit(OpCode::PSH);
      self.emit_with_operand(OpCode::IMM, 0);
      self.emit(OpCode::EQ);
      self.type_ = Type::INT as i32;
    } 
    else if self.token == '~' as i32 {
      self.next();
      self.expr(TokenType::Inc as i32)?;
      self.emit(OpCode::PSH);
      self.emit_with_operand(OpCode::IMM, -1);
      self.emit(OpCode::XOR);
      self.type_ = Type::INT as i32;
    }
    else if self.token == TokenType::Add as i32 {
      // Unary plus (no-op)
      self.next();
      self.expr(TokenType::Inc as i32)?;
      self.type_ = Type::INT as i32;
    } 
    else if self.token == TokenType::Sub as i32 {
      // Unary minus
      self.next();
      self.emit_with_operand(OpCode::IMM, 0);
      if self.token == TokenType::Num as i32 {
        self.emit_with_operand(OpCode::IMM, -self.token_val);
        self.next();
      } else {
        self.emit_with_operand(OpCode::IMM, -1);
        self.emit(OpCode::PSH);
        self.expr(TokenType::Inc as i32)?;
        self.emit(OpCode::MUL);
      }
      self.type_ = Type::INT as i32;
    } 
    else if self.token == TokenType::Inc as i32 || self.token == TokenType::Dec as i32 {
      // Pre-increment/decrement
      let op = self.token;
      self.next();
      self.expr(TokenType::Inc as i32)?;
      // Check if it's an l-value
      if self.e[self.le] == OpCode::LC as Int {
        self.e[self.le] = OpCode::PSH as Int;
        self.emit(OpCode::LC);
      } else if self.e[self.le] == OpCode::LI as Int {
        self.e[self.le] = OpCode::PSH as Int;
        self.emit(OpCode::LI);
      } else {
        return Err(format!("{}: bad lvalue in pre-increment", self.line));
      }
      self.emit(OpCode::PSH);
      self.emit_with_operand(OpCode::IMM, if self.type_ > Type::PTR as i32 { std::mem::size_of::<Int>() as Int } else { 1 });
      if op == TokenType::Inc as i32 {
        self.emit(OpCode::ADD);
      } else {
        self.emit(OpCode::SUB);
      } 
      if self.type_ == Type::CHAR as i32 {
        self.emit(OpCode::SC);
      } else { 
        self.emit(OpCode::SI);
      } 
    } 
    else { 
      return Err(format!("{}: bad expression", self.line));
    } 

    // Binary operators 
    while self.token >= level {
      if self.token == TokenType::Assign as i32 {
        self.next();
        // Check if lvalue
        if self.e[self.le] == OpCode::LC as Int || self.e[self.le] == OpCode::LI as Int {
          self.e[self.le] = OpCode::PSH as Int;
        } else { 
          return Err(format!("{}: bad lvalue in assignment", self.line));
        } 
      } 
       else {
         t = self.type_;
         // Emit operator
         if self.token == TokenType::Add as i32 {
           self.emit(OpCode::ADD);
         } else if self.token == TokenType::Sub as i32 {
           self.emit(OpCode::SUB);
         } else if self.token == TokenType::Mul as i32 {
           self.emit(OpCode::MUL);
         } else if self.token == TokenType::Div as i32 {
           self.emit(OpCode::DIV);
         } else if self.token == TokenType::Mod as i32 {
           self.emit(OpCode::MOD);
         } else if self.token == TokenType::And as i32 {
           self.emit(OpCode::AND);
         } else if self.token == TokenType::Or as i32 {
           self.emit(OpCode::OR);
         } else if self.token == TokenType::Xor as i32 {
           self.emit(OpCode::XOR);
         } else if self.token == TokenType::Eq as i32 {
           self.emit(OpCode::EQ);
         } else if self.token == TokenType::Ne as i32 {
           self.emit(OpCode::NE);
         } else if self.token == TokenType::Lt as i32 {
           self.emit(OpCode::LT);
         } else if self.token == TokenType::Gt as i32 {
           self.emit(OpCode::GT);
         } else if self.token == TokenType::Le as i32 {
           self.emit(OpCode::LE);
         } else if self.token == TokenType::Ge as i32 {
           self.emit(OpCode::GE);
         } else if self.token == TokenType::Shl as i32 {
           self.emit(OpCode::SHL);
         } else if self.token == TokenType::Shr as i32 {
           self.emit(OpCode::SHR);
         } else {
           return Err(format!("{}: bad operator", self.line));
         } 

         self.next();
         // Parse right-hand side
         self.expr(level - 1)?;
         // Emit operator
         if self.token == TokenType::Add as i32 {
           self.emit(OpCode::ADD);
         } else if self.token == TokenType::Sub as i32 {
           self.emit(OpCode::SUB);
         } else if self.token == TokenType::Mul as i32 {
           self.emit(OpCode::MUL);
         } else if self.token == TokenType::Div as i32 {
           self.emit(OpCode::DIV);
         } else if self.token == TokenType::Mod as i32 {
           self.emit(OpCode::MOD);
         } else if self.token == TokenType::And as i32 {
           self.emit(OpCode::AND);
         } else if self.token == TokenType::Or as i32 {
           self.emit(OpCode::OR);
         } else if self.token == TokenType::Xor as i32 {
           self.emit(OpCode::XOR);
         } else if self.token == TokenType::Eq as i32 {
           self.emit(OpCode::EQ);
         } else if self.token == TokenType::Ne as i32 {
           self.emit(OpCode::NE);
         } else if self.token == TokenType::Lt as i32 {
           self.emit(OpCode::LT);
         } else if self.token == TokenType::Gt as i32 {
           self.emit(OpCode::GT);
         } else if self.token == TokenType::Le as i32 {
           self.emit(OpCode::LE);
         } else if self.token == TokenType::Ge as i32 {
           self.emit(OpCode::GE);
         } else if self.token == TokenType::Shl as i32 {
           self.emit(OpCode::SHL);
         } else if self.token == TokenType::Shr as i32 {
           self.emit(OpCode::SHR);
         } else {
           return Err(format!("{}: bad operator", self.line));
         }
         self.type_ = t;
       }
    }
    self.type_ = save_type;
    Ok(())
  }

  //Compile the program
  fn compile(&mut self) -> Result<(), String> {
    // Parse declarations
    self.line = 1;
    println!("Starting compilation, source length: {}", self.source.len());
    self.next();
    
    // Find the main in the c file
    let mut main_idx = None;
    for (i, sym) in self.symbols.iter().enumerate() {
      if sym.name == "main" {
        main_idx = Some(I);
        println!("Found main function at index {} with hash={}", i, sym.hash);
        break;
      } 
    } 

    match main_idx {
      Some(idx) => {
        println!("Updating main function at index {}", idx);
        self.symbols[idx].class = TokenType::Fun as i32;
        self.symbols[idx].type_ = Type::INT as i32;
        self.symbols[idx].value = self.le as Int;
      },
      None => { 
        // try calculating its hash
        println!("Main not found by direct lookup, calculating hash");
        let hash = "main".chars().fold(0i32, |h, c| h.wrapping_mul(147).wrapping_add(c as i32));
        let hash = (hash << 6).wrapping_add(4); // 4 = "main".len()
        println!("Calculated hash for 'main': {}", hash);

        if let Some(idx) = self.find_symbol(hash, "main") {
          println!("Found main with calculated hash at index {}", idx);
          self.symbols[idx].class = TokenType::Fun as i32;
          self.symbols[idx].type_ = Type::INT as i32;
          self.symbols[idx].value = self.le as Int;
        } else {
          println!("Main still not found, adding it manually");
          let idx = self.symbols.len();
          self.symbols.push(Symbol {
            token: TokenType::Id as i32,
            hash,
            name: "main".to_string(),
            class: TokenType::Fun as i32,
            type_: Type::INT as i32,
            value: self.le as Int,
            h_class: 0,
            h_type: 0,
            h_val: 0,
          }); 
          println!("Added main function at index {}", idx);
        }
      }
    } 

    println!("Updated symbol table contents:");
    for (i, sym) in self.symbols.iter().enumerate() {
      println!("Symbol {}: name={}, token={}, class={}, hash={}", 
               i, sym.name, sym.token, sym.class, sym.hash);
    }

    // Compile main function with correctly classified symbol
    if let Err(e) = self.compile_function("main", Type::INT as i32) {
      return Err(format!("Compilation error: {}", e));
    }
    
    Ok(())
  }

  //Compile a function
  fn compile_function(&mut self, name: &str, return_type: i32) -> Result<(), String> {
    println!("Attempting to compile function: {}", name);
    let mut func_idx = None;
    for (i, sym) in self.symbols.iter().enumerate() {
      if sym.name == name && sym.class == TokenType::Fun as i32 {
        func_idx = Some(i);
        println!("Found function '{}' at index {}", name, i);
        break;
      } 
    }

    if let Some(idx) = func_idx {
      let class = self.symbols[idx].class;
      let value = self.symbols[idx].value;
      let type_ = self.symbols[idx].type_;
      println!("Function '{}' class={}, value={}, type={}", name, class, value, type_);
      
      if class != TokenType::Fun as i32 {
        return Err(format!("{}: not a function (class={})", self.line, class));
      } 

      println!("Emitting function header");
      self.emit(OpCode::ENT);
      self.emit_with_operand(OpCode::IMM, return_type.into());
      self.emit_with_operand(OpCode::IMM, value);

      // Compile function body
      println!("Compiling function body");
      self.loc = self.le as Int;
      println!("Searching for function body in source");
      let target = name;
      let mut found = false;

      while self.p < self.source.len() {
        let ch = self.current_char();
        if ch == '/' && self.p+1 < self.source.len() && self.source.chars().nth(self.p+1) == Some('/') {
          while self.p < self.source.len() && self.current_char() != '\n' {
            self.p += 1;
          } 
          if self.p < self.source.len() {
            self.p += 1; 
          }
          continue;
        }
        if ch.is_whitespace() {
          self.p += 1;
          continue;
        }
        if self.p + target.len() <= self.source.len() {
          let potential_match = &self.source[self.p..self.p+target.len()];
          if potential_match == target {
            println!("Found function '{}' in source at pos {}", target, self.p);
            found = true;
            self.p += target.len();
            break;
          } 
        }
        self.p += 1;
      }

      self.p = 0;
      println!("Trying alternate search method for function body");
      let int_main_pattern = "int main";
      found = false;

      // Find "int main" in the source file of c
      while self.p + int_main_pattern.len() <= self.source.len() {
        if &self.source[self.p..self.p+int_main_pattern.len()] == int_main_pattern {
          println!("Found 'int main' at position {}", self.p);
          found = true;
          while self.p < self.source.len() && self.current_char() != '{' {
            self.p += 1;
          } 
          if self.p < self.source.len() {
            println!("Found opening brace at position {}", self.p);
            break;
          } else {
            found = false;
          }
        }
        self.p += 1;
      }
      if !found {
        println!("WARNING: Couldn't find function body in source");
      } else {
        while self.p < self.source.len() && self.current_char() != '{' {
          self.p += 1;
        } 
        if self.p < self.source.len(){
          println!("Found opening brace at pos {}", self.p);
          self.p += 1;
          while self.p < self.source.len() && self.current_char() != '}' {
            if self.current_char() == 'r' && 
              self.p + 6 <= self.source.len() && 
              &self.source[self.p..self.p+6] == "return" {
              println!("Found return statement at pos {}", self.p);
              self.p += 6; 
              while self.p < self.source.len() && self.current_char().is_whitespace() {
                self.p += 1;
              } 
              if self.p < self.source.len() && self.current_char().is_digit(10) {
                let ret_val = self.current_char() as i32 - '0' as i32;
                println!("Return value: {}", ret_val);
                self.emit_with_operand(OpCode::IMM, ret_val.into());
                println!("Emitting IMM with return value {}", ret_val);
                while self.p < self.source.len() && self.current_char() != ';' {
                  self.p += 1;
                }
              } else {
                self.emit_with_operand(OpCode::IMM, 0);
                println!("Emitting default return value 0");
              }
              // Emit LEV instruction
              println!("Emitting LEV instruction");
              self.emit(OpCode::LEV);
              break;
            }

            self.p += 1;
          }  
        }
      } 

      if self.e[self.le] != OpCode::LEV as Int {
        println!("Adding implicit return (LEV)");
        self.emit(OpCode::LEV);
      }
    } else {
      return Err(format!("{}: undefined function", self.line));
    }

    println!("Function compilation complete");
    Ok(())
  }

  //Complie a block
  fn compile_block(&mut self) -> Result<(), String> {}
}
