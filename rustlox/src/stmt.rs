use crate::expr::Expr;
use crate::scanner::Token;

#[derive(Debug)]
pub struct Break<'a> {
    pub keyword: &'a Token<'a>,
}

#[derive(Debug)]
pub struct Block<'a> {
    pub statements: Vec<Stmt<'a>>,
}

#[derive(Debug)]
pub struct Continue<'a> {
    pub keyword: &'a Token<'a>,
}

#[derive(Debug)]
pub struct Expression<'a> {
    pub expression: Expr<'a>,
}

#[derive(Debug)]
pub struct For<'a> {
    pub initializer: Option<Box<Stmt<'a>>>,
    pub condition: Option<Expr<'a>>,
    pub increment: Option<Expr<'a>>,
    pub body: Box<Stmt<'a>>,
}

#[derive(Copy, Clone, Debug)]
pub enum FunctionKind {
    Script,
    Function,
}

#[derive(Debug)]
pub struct Function<'a> {
    pub name: &'a Token<'a>,
    pub params: Vec<&'a Token<'a>>,
    pub body: Vec<Stmt<'a>>,
    pub kind: FunctionKind,
}

#[derive(Debug)]
pub struct If<'a> {
    pub condition: Expr<'a>,
    pub then_branch: Box<Stmt<'a>>,
    pub else_branch: Option<Box<Stmt<'a>>>,
}

#[derive(Debug)]
pub struct Print<'a> {
    pub expression: Expr<'a>,
}

#[derive(Debug)]
pub struct Return<'a> {
    pub keyword: &'a Token<'a>,
    pub value: Option<Expr<'a>>,
}

#[derive(Debug)]
pub struct Var<'a> {
    pub name: &'a Token<'a>,
    pub initializer: Option<Expr<'a>>,
}

#[derive(Debug)]
pub struct While<'a> {
    pub condition: Expr<'a>,
    pub body: Box<Stmt<'a>>,
}

#[derive(Debug)]
pub enum Stmt<'a> {
    Break(Break<'a>),
    Block(Block<'a>),
    Continue(Continue<'a>),
    Expression(Expression<'a>),
    For(For<'a>),
    Function(Function<'a>),
    If(If<'a>),
    Print(Print<'a>),
    Return(Return<'a>),
    Var(Var<'a>),
    While(While<'a>),
}
