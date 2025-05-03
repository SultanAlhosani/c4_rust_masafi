/// Abstract Syntax Tree (AST) for a simple programming language.
/// This module defines the structure of the AST nodes

#[derive(Debug, Clone)]
/// Represents the different types of expressions in the language.
pub enum Expr {
    Number(i32),
    Variable(String),
    Boolean(bool),
    Char(char),
    BinaryOp {
        op: BinOp,        // Binary operator
        left: Box<Expr>,  // Left operand
        right: Box<Expr>, // Right operand
    },
    UnaryOp {
        op: UnOp,         // Unary operator
        expr: Box<Expr>,  // Operand
    },
    FunctionCall {
        name: String,    // Name of the function
        args: Vec<Expr>, // Arguments for the function call
    },
}

/// Represents the different binary operators in the language.
/// Includes arithmetic, comparison, and logical operators.
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
    LessEqual,      // ✅ add this
    GreaterEqual,   // ✅ and this
    And,         // &&
    Or,          // ||
    Assign, // =
}

/// Represents the different unary operators in the language.
#[derive(Debug, Clone)]
pub enum UnOp {
    Not, // !
}

/// Represents the different types of statements in the language.
/// This includes control flow, variable declarations, and functions.
#[derive(Debug, Clone)]
pub enum Stmt {
    Return(Expr),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Let {
        name: String,
        value: Expr,
    },
    Assign {
        name: String,
        value: Expr,
    },
    Block(Vec<Stmt>),
    Function {
        name: String,
        params: Vec<String>,
        body: Box<Stmt>,
    },
    Print(Expr), // ✅ NEW: print statement
}
