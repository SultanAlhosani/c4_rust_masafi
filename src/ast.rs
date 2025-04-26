#[derive(Debug, Clone)]
pub enum Expr {
    Number(i32),
    Variable(String),
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Boolean(bool), // <-- ADD THIS LINE
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
    If { condition: Expr, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>> },
    While { condition: Expr, body: Box<Stmt> },
    Let { name: String, value: Expr },    // <-- this line
    Assign { name: String, value: Expr },
    Block(Vec<Stmt>),
}





