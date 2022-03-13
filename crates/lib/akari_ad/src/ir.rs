use std::{collections::HashMap, rc::Rc};

use crate::parse::Number;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Primitive {
    I32,
    F32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vector(Primitive, usize);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Matrix(Primitive, usize, usize);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<(String, Rc<Type>)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Array(Rc<Type>, usize);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slice(Rc<Type>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tuple(Vec<Rc<Type>>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Primitive(Primitive),
    Vector(Vector),
    Matrix(Matrix),
    Struct(Struct),
    Tuple(Tuple),
    // Array(Array),
    Slice(Slice),
}
#[derive(Clone, Debug)]
pub enum Atom {
    Number(Number, Primitive),
    String(String),
    Identifier(String),
}

#[derive(Clone, Debug)]
pub enum PrimFunc {
    Add,
    Sub,
    Mul,
    Div,
    Sin,
    Cos,
    Load,
    Extract,
    Insert,
}

#[derive(Clone, Debug)]
pub struct UserFunc {
    pub name: String,
    pub ty: (Vec<Type>, Type),
    pub is_method: bool,
}
#[derive(Clone, Debug)]
pub enum Func {
    PrimFunc(PrimFunc),
}

#[derive(Clone, Debug)]
pub enum Node {
    Atom(Atom),
    Func(Func),
    IfThenElse {
        cond: Rc<Node>,
        then: Rc<Node>,
        else_: Rc<Node>,
    },
    Call {
        f: Func,
        args: Vec<Rc<Node>>,
    },
}

pub trait Transform {
    type Output: Clone;
    fn transform(&mut self, node: &Rc<Node>) -> Self::Output;
}
pub struct Transformer<T: Transform> {
    op: T,
    cache: HashMap<usize, T::Output>,
}
impl<T: Transform> Transformer<T> {
    pub fn transform(&mut self, node: &Rc<Node>) -> T::Output {
        let ptr = node.as_ref() as *const Node as usize;
        if let Some(r) = self.cache.get(&ptr) {
            return r.clone();
        }
        let r = self.op.transform(node);
        self.cache.insert(ptr, r.clone());
        r
    }
}
