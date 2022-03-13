use std::rc::Rc;

use crate::ir;

#[derive(Clone, Debug)]
pub struct SourceLocation {
    pub file: Rc<String>,
    pub line: usize,
    pub col: usize,
}

#[derive(Clone, Debug)]
pub enum Number {
    Int(i64),
    Float(f64),
}
impl ToString for Number {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}
#[derive(Clone, Debug)]
pub enum Token {
    Number { value: Number, loc: SourceLocation },
    Symbol { value: String, loc: SourceLocation },
    String { value: String, loc: SourceLocation },
    Identifier { value: String, loc: SourceLocation },
    Keyword { value: String, loc: SourceLocation },
    EOF { loc: SourceLocation },
}
impl Token {
    pub fn is_eof(&self) -> bool {
        match self {
            Token::EOF { .. } => true,
            _ => false,
        }
    }
    pub fn str(&self) -> String {
        match self {
            Token::Symbol { value, .. } => value.clone(),
            Token::Keyword { value, .. } => value.clone(),
            Token::Number { value, .. } => value.to_string(),
            Token::String { value, .. } => value.clone(),
            Token::Identifier { value, .. } => value.clone(),
            Token::EOF { .. } => "".into(),
        }
    }

    pub fn as_identifier(&self) -> Option<&String> {
        match self {
            Token::Identifier { value, .. } => Some(value),
            _ => None,
        }
    }
    #[allow(dead_code)]
    pub fn loc(&self) -> &SourceLocation {
        match self {
            Token::Number { value: _, loc } => loc,
            Token::Symbol { value: _, loc } => loc,
            Token::String { value: _, loc } => loc,
            Token::Identifier { value: _, loc } => loc,
            Token::Keyword { value: _, loc } => loc,
            Token::EOF { loc } => loc,
        }
    }
}
#[derive(Clone, Debug)]
pub enum Stmt {
    Assign { var: Token, expr: Expr },
    Return(Option<Expr>),
    Expr(Expr),
}
#[derive(Clone, Debug)]
pub enum Expr {
    Literal(Token),
    Unary {
        op: Token,
        expr: Box<Expr>,
    },
    Binary {
        op: Token,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Call {
        func: Token,
        args: Vec<Expr>,
    },
    MethodCall {
        object: Vec<Expr>,
        func: Token,
        args: Vec<Expr>,
    },
    Block {
        stms: Vec<Stmt>,
        ret: Option<Box<Expr>>,
    },
    IfThenElse {
        cond: Box<Expr>,
        then: Box<Expr>,
        else_: Box<Expr>,
    },
}
#[derive(Clone, Debug)]
pub enum Type {
    Atom(Token),
}
#[derive(Clone, Debug)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
    pub requires_gradient: bool,
}
#[derive(Clone, Debug)]
pub struct Function {
    pub name: Token,
    pub parameters: Vec<Parameter>,
    pub body: Stmt,
}

// fn parse_str(s:&str)->Function {}