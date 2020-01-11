#[derive(Clone, Debug)]
pub enum Node {
    Empty,
    BinOp(Box<Node>, Op, Box<Node>),
    Number(i64),
    StrLiteral(String),
    Ident(String),
    Call(Vec<Node>),
    VarDef(String, Box<Node>),
    FuncDef(String, Vec<(String, String)>, String, Vec<Node>), // funcname, (argname, argtype), rettype, body
    If(Box<Node>, Box<Node>, Box<Node>),
    Ret(Box<Node>)
}

#[derive(Clone, Copy, Debug)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Eql,
    And,
    Or,
}