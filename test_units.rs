use std::fs;

// Import from main crate
extern crate c4_rust;
use c4_rust::{C4, TokenType, OpCode, Type};

#[test]
fn test_init_symbol_table() {
    let mut c4 = C4::new();
    c4.init_symbol_table();
    
    // Verify that the symbol table has entries after initialization
    assert!(!c4.symbols.is_empty());
    
    // Check for specific keywords
    let has_char = c4.symbols.iter().any(|sym| sym.name == "char");
    let has_if = c4.symbols.iter().any(|sym| sym.name == "if");
    let has_printf = c4.symbols.iter().any(|sym| sym.name == "printf");
    
    assert!(has_char);
    assert!(has_if);
    assert!(has_printf);
}

#[test]
fn test_lexer() {
    let mut c4 = C4::new();
    c4.init_symbol_table();
    
    // Test identifier lexing
    c4.source = "main".to_string();
    c4.p = 0;
    c4.next();
    assert_eq!(c4.token, TokenType::Id as i32);
    
    // Test number lexing
    c4.source = "42".to_string();
    c4.p = 0;
    c4.next();
    assert_eq!(c4.token, TokenType::Num as i32);
    assert_eq!(c4.token_val, 42);
    
    // Test operator lexing
    c4.source = "+".to_string();
    c4.p = 0;
    c4.next();
    assert_eq!(c4.token, TokenType::Add as i32);
}

#[test]
fn test_expression_parsing() {
    let mut c4 = C4::new();
    c4.init_symbol_table();
    
    // Test simple expression
    c4.source = "42".to_string();
    c4.p = 0;
    c4.next();
    let result = c4.expr(TokenType::Assign as i32);
    assert!(result.is_ok());
    
    // Verify the generated code (should have IMM 42)
    assert_eq!(c4.e[1], OpCode::IMM as i64);
    assert_eq!(c4.e[2], 42);
}

#[test]
fn test_full_compilation() {
    let mut c4 = C4::new();
    c4.init_symbol_table();
    
    // Test compiling a minimal C program
    c4.source = "int main() { return 42; }".to_string();
    let result = c4.compile();
    assert!(result.is_ok());
    
    // Verify main was found
    assert!(c4.find_main().is_some());
}

#[test]
fn test_simple_program() {
    // Create a simple C program
    let program = "int main() { return 42; }";
    
    // Write it to a temporary file
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_program.c");
    fs::write(&test_file, program).expect("Failed to write test file");
    
    // Run the compiler on it
    let mut c4 = C4::new();
    c4.init_symbol_table();
    c4.source = program.to_string();
    let result = c4.compile();
    assert!(result.is_ok());
    
    // Clean up
    fs::remove_file(test_file).expect("Failed to remove test file");
}
