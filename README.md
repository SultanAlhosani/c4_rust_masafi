# C4 Rust Compiler

## Project Overview

This project involves rewriting the C4 compiler in Rust. The goal is to reimplement the C4 compiler, which processes a specific subset of C code, using Rust’s features. The Rust version of the compiler must compile the same subset of C code as the original C4 compiler, maintaining its self-hosting capability and core functionality while improving design where possible.

## Objective

The objective of this project is to:

- **Rewriting the C4 Codebase in Rust**: Reimplement the lexer, parser, virtual machine, and all other components of the C4 compiler in Rust.
- **Self-Hosting**: The Rust version must compile the same C code as the original C4, including its own source code (self-hosting).
- **Use Rust Features**: Leverage Rust’s safety, modern features, and performance improvements, while ensuring compatibility with the original C4 functionality.

## Features

### C4 Features (Implemented in Both C and Rust Versions)

#### 1. **Primitive Types**:
   - `int`, `char`, `bool`, `void`
   - `str` (as a pointer to `char`)

#### 2. **Expressions & Operators**:
   - **Arithmetic operators**: `+`, `-`, `*`, `/`, `%`
   - **Comparison operators**: `==`, `!=`, `<`, `>`, `<=`, `>=`
   - **Logical operators**: `&&`, `||`, `!`
   - **Bitwise operators**: `&`, `|`, `^`, `<<`, `>>`, `~`
   - **Unary operators**: `++`, `--`, `!`, `*`, `&`
   - **Ternary conditional** (`? :`)

#### 3. **Variable Declarations and Assignment**:
   - `let` and typed declarations (e.g., `int x = 5;`)
   - Comma-separated variable declarations (e.g., `let x = 1, y = 2;`)
   - Implicit declarations by assignment (e.g., `x = 7;`)
   - Shadowing and scoping with nested blocks

#### 4. **Data Structures**:
   - **Arrays**:
     - Array literals (e.g., `[1, 2, 3]`)
     - Indexing (e.g., `arr[1]`)
     - Assignment (e.g., `arr[2] = 42`)
     - Size querying with `sizeof(int[3])`
   - **Strings**:
     - String literals (e.g., `"hello"`)
     - String concatenation with `+`
     - Printing and returning strings

#### 5. **Control Flow**:
   - `if`, `else if`, `else`
   - `while` loops (including nested loops)
   - Blocks `{}` with scoped variables
   - Early return with `return`

#### 6. **Functions**:
   - Function declarations with types (e.g., `int f(int x) { ... }`)
   - Multiple parameters, recursion, overwriting function definitions
   - Returning strings and numbers
   - `main()` function with support for `return main();`

#### 7. **Memory Simulation**:
   - **Pointer support**:
     - `&x` gives an address (multiplied by 1000)
     - `*p` dereferences (divided by 1000)

#### 8. **Type Casting**:
   - Supported for types like `(int)`, `(char)`, and `(Pointer)`

#### 9. **Enumerations (Enums)**:
   - Enum syntax (e.g., `enum { A = 1, B, C = 10, D };`)
   - Auto-increment of enum values
   - Local and global enums

#### 10. **Built-in Functions**:
   - `print(...)` function supporting integers, strings, and arrays (e.g., `[1, 2, 3]`)

#### 11. **Utilities**:
   - `sizeof(...)` operator supporting basic types (`int`, `char`, `bool`, `str`) and arrays

#### 12. **Error Reporting**:
   - Syntax errors with line and column numbers
   - Parser panics on invalid code with detailed messages
   - Runtime panics for division by zero, undefined variables, invalid pointer usage, and out-of-bounds access

#### 13. **Comment Support**:
   - Both single-line (`//`) and multi-line (`/* ... */`) comments supported

---

### Bonus Features (Implemented in Rust but not in C4)

#### 1. **String Concatenation**:
   - Direct concatenation of strings using the `+` operator, which is not available in C4.

#### 2. **Sizeof Operator Enhanced**:
   - Enhanced `sizeof()` operator that handles more types, including support for arrays, which was simplified in the original C4.

#### 3. **Pointer Arithmetic Simulation**:
   - Pointer arithmetic supported, with addresses being simulated using multiplication/division.

#### 4. **Enhanced Error Reporting**:
   - Improved error reporting for syntax errors, runtime issues like division by zero, undefined variables, and pointer errors.

#### 5. **Recursion Support**:
   - Support for recursion in function definitions (e.g., `factorial(n)`), which C4's implementation was less focused on.

#### 6. **Recursive Function Overwriting**:
   - The Rust version allows function overwriting, which was not part of the original C4 but enhances flexibility and testing.

---

## Setup Instructions

### Cloning the Repository

Clone the repository and navigate into the project directory:

```bash
git clone https://github.com/your-team-name/c4_rust_masafi.git
cd c4_rust_masafi
```

### Running the Code
cd src
cargo run

### Testing the Code
cargo test

Generated Documentation
The project’s documentation is generated using cargo doc. It is available in the following location after building the project:

target/doc/compiler/index.html

You can also generate and view the documentation yourself by running:

cargo doc --open
