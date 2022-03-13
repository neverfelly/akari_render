use std::{collections::HashMap, rc::Rc};

use crate::parse::Number;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Primitive {
    I32,
    F32,
    Bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Array(Rc<Type>, usize);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slice(Rc<Type>);
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Path(pub Vec<String>);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tuple(Vec<Rc<Type>>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    // Primitive(Primitive),
    Path(Path),
    // Vector(Vector),
    // Matrix(Matrix),
    Tuple(Tuple),
    // Array(Array),
    Slice(Slice),
    Inferred,
}
#[derive(Clone, Debug)]
pub enum Atom {
    Literal(String, Primitive),
    String(String),
    Identifier { path: Path, constant: bool },
}

#[derive(Clone, Debug)]
pub enum PrimFunc {
    Neg,
    Not,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    BitAnd,
    BitOr,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Load,
    Extract,
    Insert,
}

#[derive(Clone, Debug)]
pub enum Func {
    PrimFunc(PrimFunc),
    Named(Path),
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Var {
    Id(usize),
    Named(Path),
}
#[derive(Clone, Debug)]
pub enum Expr {
    Atom(Atom),
    Var(Var),
    Call(Func, Vec<Var>),
    IfThenElse {
        cond: Var,
        then: Box<Block>,
        else_: Box<Block>,
    },
}
#[derive(Clone, Debug)]
pub struct Let {
    pub var: Var,
    pub val: Expr,
}
#[derive(Clone, Debug)]
pub struct Block {
    pub bindings: Vec<Let>,
    pub ret: Option<Var>,
}
impl Block {
    pub fn new() -> Self {
        Self {
            bindings: vec![],
            ret: None,
        }
    }
}

// #[derive(Clone, Debug)]
// pub enum Node {
//     Atom(Atom),
//     Func(Func),
//     IfThenElse {
//         cond: Rc<Node>,
//         then: Rc<Node>,
//         else_: Rc<Node>,
//     },
//     Call {
//         f: Func,
//         args: Vec<Rc<Node>>,
//     },
// }

// pub trait Transform {
//     type Output: Clone;
//     fn transform(&mut self, node: &Rc<Node>) -> Self::Output;
// }
// pub struct Transformer<T: Transform> {
//     op: T,
//     cache: HashMap<usize, T::Output>,
// }
// impl<T: Transform> Transformer<T> {
//     pub fn transform(&mut self, node: &Rc<Node>) -> T::Output {
//         let ptr = node.as_ref() as *const Node as usize;
//         if let Some(r) = self.cache.get(&ptr) {
//             return r.clone();
//         }
//         let r = self.op.transform(node);
//         self.cache.insert(ptr, r.clone());
//         r
//     }
// }
