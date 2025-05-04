/// Abstract Syntax Tree (AST) for a simple programming language.
/// This module defines the structure of the AST nodes.
#[derive(Debug, Clone)]
/// Represents the different types of expressions in the language.
pub enum Expr {
    /// A numeric literal (e.g., 42)
    Number(i32),
    /// A variable reference (e.g., x)
    Variable(String),
    /// A boolean literal (e.g., true or false)
    Boolean(bool),
    /// A character literal (e.g., 'a')
    Char(char),
    /// An array literal (e.g., {1, 2, 3})
    ArrayLiteral(Vec<Expr>),
    /// An array index expression (e.g., arr[0])
    ArrayIndex(Box<Expr>, Box<Expr>),
    /// A string literal (e.g., "Hello")
    StringLiteral(String),
    /// Pre-increment operation (e.g., ++x)
    PreInc(Box<Expr>),
    /// Pre-decrement operation (e.g., --x)
    PreDec(Box<Expr>),
    /// Post-increment operation (e.g., x++)
    PostInc(Box<Expr>),
    /// Post-decrement operation (e.g., x--)
    PostDec(Box<Expr>),
    /// Ternary conditional expression (e.g., condition ? then : else)
    Ternary {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    /// Binary operation (e.g., a + b)
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// Unary operation (e.g., !x)
    UnaryOp {
        op: UnOp,
        expr: Box<Expr>,
    },
    /// Function call (e.g., my_function(arg1, arg2))
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
    /// Enum value (e.g., EnumName.Variant)
    #[allow(dead_code)]
    EnumValue(String, String),
    /// SizeOf operator to get the size of a type
    SizeOf(Type),
    /// Type casting (e.g., (int)x)
    Cast(Type, Box<Expr>),
    /// Address-of operator (e.g., &x)
    AddressOf(Box<Expr>),
    /// Dereference operator (e.g., *x)
    Deref(Box<Expr>),
}

/// Represents the different binary operators in the language.
/// Includes arithmetic, comparison, and logical operators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinOp {
    /// Addition operator (e.g., a + b)
    Add,
    /// Subtraction operator (e.g., a - b)
    Sub,
    /// Multiplication operator (e.g., a * b)
    Mul,
    /// Division operator (e.g., a / b)
    Div,
    /// Equality operator (e.g., a == b)
    Equal,
    /// Inequality operator (e.g., a != b)
    NotEqual,
    /// Less than operator (e.g., a < b)
    LessThan,
    /// Greater than operator (e.g., a > b)
    GreaterThan,
    /// Less than or equal operator (e.g., a <= b)
    LessEqual,
    /// Greater than or equal operator (e.g., a >= b)
    GreaterEqual,
    /// Logical AND operator (e.g., a && b)
    And,
    /// Logical OR operator (e.g., a || b)
    Or,
    /// Assignment operator (e.g., a = b)
    Assign,
    /// Modulus operator (e.g., a % b)
    Mod,
    /// Bitwise AND operator (e.g., a & b)
    BitAnd,
    /// Bitwise OR operator (e.g., a | b)
    BitOr,
    /// Bitwise XOR operator (e.g., a ^ b)
    BitXor,
    /// Bitwise left shift operator (e.g., a << b)
    Shl,
    /// Bitwise right shift operator (e.g., a >> b)
    Shr,
}

/// Represents the different unary operators in the language.
#[derive(Debug, Clone)]
pub enum UnOp {
    /// Logical NOT operator (e.g., !x)
    Not,
}

/// Represents runtime values (integers and strings).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Value {
    /// Integer value
    Int(i32),
    /// String value
    Str(String),
}

/// Represents the different types in the language.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Integer type
    Int,
    /// Character type
    Char,
    /// Pointer type (e.g., int* or char*)
    Pointer(Box<Type>),
    /// Void type (for functions that do not return a value)
    Void,
    /// Array type (e.g., int[3])
    Array(Box<Type>, usize),
}

/// Represents the different types of statements in the language.
/// This includes control flow, variable declarations, and functions.
#[derive(Debug, Clone)]
pub enum Stmt {
    /// Return statement (e.g., return 42;)
    Return(Expr),
    /// Print statement (e.g., print("Hello");)
    Print(Expr),
    /// Expression statement (e.g., x = 42;)
    ExprStmt(Expr),
    /// Block of statements (e.g., { ... })
    Block(Vec<Stmt>),
    /// Variable declaration (e.g., let x = 42;)
    #[allow(dead_code)]
    Let { name: String, value: Expr, var_type: Option<Type> },
    /// Assignment statement (e.g., x = 42;)
    #[allow(dead_code)]
    Assign {
        name: String,
        value: Expr,
    },
    /// If statement (e.g., if (x > 0) { ... } else { ... })
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    /// While loop (e.g., while (x > 0) { ... })
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    /// Function definition (e.g., function foo() { ... })
    Function {
        name: String,
        params: Vec<String>,
        body: Box<Stmt>,
        return_type: Option<Type>,
    },
}
