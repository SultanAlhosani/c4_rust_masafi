/// Abstract Syntax Tree (AST) for a simple programming language.
/// This module defines the structure of the AST nodes

#[derive(Debug, Clone)]
/// represents the different types of expressions in the language.
pub enum Expr {
    Number(i32),
    Variable(String),
    BinaryOp {
        op: BinOp,        // Binary operator
        left: Box<Expr>,  // Left operand
        right: Box<Expr>, // Right operand
    },
    Boolean(bool),
    Char(char),
    FunctionCall {
        // added function call
        name: String,    // Name of the function
        args: Vec<Expr>, // Arguments for the function call
    },
}
//// Represents the different binary operators in the language.
/// This includes arithmetic, comparison, and logical operators.
#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Equal,       // ==
    NotEqual,    // !=
    LessThan,    // <
    GreaterThan, // >
}
/// Represents the different types of statements in the language.
/// This includes control flow statements, variable declarations, and function definitions.
#[derive(Debug, Clone)]
pub enum Stmt {
    Return(Expr),
    If {
        condition: Expr,                // Condition for the if statement
        then_branch: Box<Stmt>,         // Branch to execute if condition is true
        else_branch: Option<Box<Stmt>>, // Optional else branch
    },
    While {
        condition: Expr, // Condition for the while loop
        body: Box<Stmt>, // Body of the loop
    },
    Let {
        name: String, // Name of the variable
        value: Expr,  // Initial value of the variable
    },
    Assign {
        name: String, // Name of the variable
        value: Expr,  // Value to assign to the variable
    },
    Block(Vec<Stmt>),
    Function {
        //added function declaration
        name: String,        // Name of the function
        params: Vec<String>, // Parameters of the function
        body: Box<Stmt>,     // Body of the function
    },
}
