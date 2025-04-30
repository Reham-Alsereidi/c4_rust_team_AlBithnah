# c4_rust_team_AlBithnah
A Rust implementation of the c4 tiny C compiler.

## Description

c4_rust is a port of the original c4 C compiler to Rust. It follows the same principles and behavior as the original c4 implementation:
- Compiles a subset of C (char, int, pointers, if, while, return statements)
- Includes a built-in virtual machine to execute the compiled code
- Accepts the same command-line arguments as the original c4
- Self-hosting: capable of compiling the original c4.c source code

## Requirements

- Rust compiler (rustc) and Cargo
- For Windows users: You'll need MSYS2 or MinGW for libc compatibility

## Building the project
### Windows
```powershell
cd c4_rust
cargo build --release
```

Note: On Windows, you must ensure you have the appropriate MSYS2/MinGW environment set up for libc support.

The compiled binary will be available at:
- Windows: `target\release\c4_rust.exe`

## Usage

```
c4_rust [-s] [-d] file.c
```

Where:
- `-s`: Shows source code and assembly output during compilation
- `-d`: Enables debug mode that prints executed instructions
- `file.c`: Path to the C source file you want to compile and execute

## Examples

1. Compile and run a simple C program:
first go to the c4_rust folder
```
cd c4_rust
```
Run The Code
```

cargo run -- test.c
```
OR
```
/c4_rust/cargo run -- test.c
```

2. Show source and assembly during compilation:
```
./c4_rust -s example.c
```

3. Run with debugging information:
```
./c4_rust -d example.c
```

4. Compile the original c4.c (self-hosting):
```
./c4_rust c4.c
```


## Project Structure

- `src/main.rs`: The main compiler and VM implementation
- `c4_rust_comparison.md`: A comparison report between C and Rust implementations
- `test.c`: A simple example C program for testing
- `Cargo.toml`: Project configuration and dependencies

## Supported C Features

The c4_rust compiler supports the same subset of C as the original c4:
- Basic data types: char, int, and pointers
- Control structures: if, while, return
- Expressions and basic operators
- Function definitions and calls
- Simple I/O through system calls

## Implementation Details

Our implementation uses Rust's safety features while maintaining compatibility with the original:
- Memory safety through Rust's ownership system
- Strong typing with enums for tokens and opcodes
- Proper error handling
- Comprehensive test coverage
- Documented code with Rust doc comments

## Known Limitations

The same limitations as the original c4 apply:
- Limited C language subset
- No standard library beyond the provided syscalls
- No preprocessor beyond simple # line skipping
- Limited error reporting

## License

This project is available under the same license as the original c4 project.
