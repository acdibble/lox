use crate::scanner::Token;

#[derive(Debug)]
pub struct Assign<'a> {
    pub name: &'a Token<'a>,
    pub value: Box<Expr<'a>>,
}

#[derive(Debug)]
pub struct Binary<'a> {
    pub left: Box<Expr<'a>>,
    pub operator: &'a Token<'a>,
    pub right: Box<Expr<'a>>,
}

#[derive(Debug)]
pub struct Call<'a> {
    pub callee: Box<Expr<'a>>,
    pub paren: &'a Token<'a>,
    pub args: Vec<Expr<'a>>,
}

#[derive(Debug)]
pub struct Literal<'a> {
    pub value: &'a Token<'a>,
}

#[derive(Debug)]
pub struct Logical<'a> {
    pub left: Box<Expr<'a>>,
    pub operator: &'a Token<'a>,
    pub right: Box<Expr<'a>>,
}

#[derive(Debug)]
pub struct Unary<'a> {
    pub operator: &'a Token<'a>,
    pub right: Box<Expr<'a>>,
}

#[derive(Debug)]
pub struct Variable<'a> {
    pub name: &'a Token<'a>,
}

#[derive(Debug)]
pub enum Expr<'a> {
    Assign(Assign<'a>),
    Binary(Binary<'a>),
    Call(Call<'a>),
    Literal(Literal<'a>),
    Logical(Logical<'a>),
    Unary(Unary<'a>),
    Variable(Variable<'a>),
}
