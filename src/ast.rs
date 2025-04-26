#[derive(Debug, Clone)]
pub enum Expr {
    Number(i32),
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Variable(String), // <-- New! To store x, y, z
}

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
    Block(Vec<Stmt>), // <-- NEW!! Block of statements
}




