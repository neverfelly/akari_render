use std::{collections::HashMap, rc::Rc};

use crate::ir::{self, Atom, Let, Path, PrimFunc, Type, Var};

#[derive(Clone, Debug)]
pub struct SourceLocation {
    // pub file: Rc<String>,
    pub line: usize,
    pub col: usize,
}

impl SourceLocation {
    pub fn from_span(span: proc_macro2::Span) -> Self {
        Self {
            line: span.start().line,
            col: span.end().column,
        }
    }
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
// #[derive(Clone, Debug)]
// pub enum Stmt {
//     Assign { var: Token, expr: Expr },
//     Return(Option<Expr>),
//     Expr(Expr),
// }
// #[derive(Clone, Debug)]
// pub enum Expr {
//     Literal(Token),
//     Unary {
//         op: Token,
//         expr: Box<Expr>,
//     },
//     Binary {
//         op: Token,
//         lhs: Box<Expr>,
//         rhs: Box<Expr>,
//     },
//     Call {
//         func: Token,
//         args: Vec<Expr>,
//     },
//     MethodCall {
//         object: Vec<Expr>,
//         func: Token,
//         args: Vec<Expr>,
//     },
//     Block {
//         stms: Vec<Stmt>,
//         ret: Option<Box<Expr>>,
//     },
//     IfThenElse {
//         cond: Box<Expr>,
//         then: Box<Expr>,
//         else_: Box<Expr>,
//     },
// }
// #[derive(Clone, Debug)]
// pub enum Type {
//     Atom(Token),
// }
#[derive(Clone, Debug)]
pub struct Parameter {
    pub name: ir::Var,
    pub ty: Type,
    pub requires_gradient: bool,
}
#[derive(Clone, Debug)]
pub struct Function {
    pub name: Token,
    pub parameters: Vec<Parameter>,
    pub body: ir::Block,
    pub ret: ir::Type,
}
impl Token {
    pub fn from_ident(ident: &syn::Ident) -> Self {
        Token::Identifier {
            value: ident.to_string(),
            loc: SourceLocation::from_span(ident.span()),
        }
    }
}

fn syn_ty_to_ir_ty(ty: &syn::Type) -> ir::Type {
    match ty {
        syn::Type::Path(syn::TypePath { path, .. }) => ir::Type::Path(ir::Path(
            path.segments.iter().map(|x| x.ident.to_string()).collect(),
        )),
        _ => unreachable!(),
    }
}
struct SymbolTable {
    scopes: Vec<HashMap<Path, Var>>,
    ver: HashMap<Path, usize>,
}
impl SymbolTable {
    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }
    fn pop_scope(&mut self) {
        self.scopes.pop();
    }
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            ver: HashMap::new(),
        }
    }
    fn add(&mut self, path: Path, v: Var) {
        assert!(path.0.len() == 1);
        self.scopes.last_mut().unwrap().insert(path, v);
    }
    fn get(&self, path: &Path) -> Option<Var> {
        for s in self.scopes.iter().rev() {
            if let Some(v) = s.get(path) {
                return Some(v.clone());
            }
        }
        None
    }
}
struct ParseContext {
    sym: SymbolTable,
    var_gen: usize,
}
impl ParseContext {
    fn gen_var(&mut self) -> Var {
        let i = self.var_gen;
        self.var_gen += 1;
        Var::Id(i)
    }
    fn get_ident(&self, e: &syn::Expr) -> Path {
        match e {
            syn::Expr::Path(syn::ExprPath { path, .. }) => Path(
                path.segments
                    .iter()
                    .map(|s| {
                        if !s.arguments.is_empty() {
                            panic!("generic is not supported!")
                        }
                        s.ident.to_string()
                    })
                    .collect(),
            ),
            _ => panic!("expected identifier or path"),
        }
    }
    fn get_named_var(&self, path: &Path) -> Var {
        self.sym
            .get(path)
            .unwrap_or_else(|| panic!("undefined variable {:?}", path))
    }
    fn add_named_var(&mut self, mut path: Path) -> Var {
        let o = path.clone();
        let ver = self.sym.ver.get(&o).map_or(0, |x| *x);
        {
            let last = path.0.last_mut().unwrap();
            last.push_str(&ver.to_string());
        }
        self.sym.ver.insert(o.clone(), ver + 1);
        let var = Var::Named(path);
        self.sym.add(o, var.clone());
        var
    }
    fn parse_expr(&mut self, e: &syn::Expr, block: &mut ir::Block) -> ir::Var {
        match e {
            syn::Expr::Lit(syn::ExprLit { lit, .. }) => match lit {
                syn::Lit::Float(f) => {
                    let v = self.gen_var();
                    block.bindings.push(Let {
                        var: v.clone(),
                        val: ir::Expr::Atom(ir::Atom::Literal(
                            f.base10_digits().into(),
                            ir::Primitive::F32,
                        )),
                    });
                    v
                }
                _ => unreachable!(),
            },
            syn::Expr::Path(syn::ExprPath { path, .. }) => {
                let path = Path(
                    path.segments
                        .iter()
                        .map(|s| {
                            if !s.arguments.is_empty() {
                                panic!("generic is not supported!")
                            }
                            s.ident.to_string()
                        })
                        .collect(),
                );
                // ir::Var::Named(path
                self.get_named_var(&path)
            }
            syn::Expr::Binary(syn::ExprBinary {
                left, op, right, ..
            }) => {
                let lhs = self.parse_expr(&**left, block);
                let rhs = self.parse_expr(&**right, block);
                let op = match op {
                    syn::BinOp::Add(_) => ir::PrimFunc::Add,
                    syn::BinOp::Sub(_) => ir::PrimFunc::Sub,
                    syn::BinOp::Mul(_) => ir::PrimFunc::Mul,
                    syn::BinOp::Div(_) => ir::PrimFunc::Div,
                    syn::BinOp::Rem(_) => ir::PrimFunc::Rem,
                    syn::BinOp::And(_) => ir::PrimFunc::And,
                    syn::BinOp::Or(_) => ir::PrimFunc::Or,
                    syn::BinOp::BitXor(_) => todo!(),
                    syn::BinOp::BitAnd(_) => todo!(),
                    syn::BinOp::BitOr(_) => todo!(),
                    syn::BinOp::Shl(_) => ir::PrimFunc::Shl,
                    syn::BinOp::Shr(_) => ir::PrimFunc::Shr,
                    syn::BinOp::Eq(_) => ir::PrimFunc::Eq,
                    syn::BinOp::Lt(_) => ir::PrimFunc::Lt,
                    syn::BinOp::Le(_) => ir::PrimFunc::Le,
                    syn::BinOp::Ne(_) => ir::PrimFunc::Ne,
                    syn::BinOp::Ge(_) => ir::PrimFunc::Ge,
                    syn::BinOp::Gt(_) => ir::PrimFunc::Gt,
                    syn::BinOp::AddEq(_) => todo!(),
                    syn::BinOp::SubEq(_) => todo!(),
                    syn::BinOp::MulEq(_) => todo!(),
                    syn::BinOp::DivEq(_) => todo!(),
                    syn::BinOp::RemEq(_) => todo!(),
                    syn::BinOp::BitXorEq(_) => todo!(),
                    syn::BinOp::BitAndEq(_) => todo!(),
                    syn::BinOp::BitOrEq(_) => todo!(),
                    syn::BinOp::ShlEq(_) => todo!(),
                    syn::BinOp::ShrEq(_) => todo!(),
                };
                let var = self.gen_var();
                let binding = ir::Let {
                    var: var.clone(),
                    val: ir::Expr::Call(ir::Func::PrimFunc(op), vec![lhs, rhs]),
                };
                block.bindings.push(binding);
                var
            }
            syn::Expr::Unary(syn::ExprUnary { op, expr, .. }) => {
                let expr = self.parse_expr(&**expr, block);
                let op = match op {
                    syn::UnOp::Neg(_) => ir::PrimFunc::Neg,
                    syn::UnOp::Not(_) => ir::PrimFunc::Not,
                    _ => unreachable!(),
                };
                let var = self.gen_var();
                let binding = ir::Let {
                    var: var.clone(),
                    val: ir::Expr::Call(ir::Func::PrimFunc(op), vec![expr]),
                };
                block.bindings.push(binding);
                var
            }
            syn::Expr::Field(syn::ExprField { base, member, .. }) => {
                let member: ir::Member = member.into();
                let base = self.parse_expr(base, block);
                let var = self.gen_var();
                let binding = ir::Let {
                    var: var.clone(),
                    val: ir::Expr::Extract(base, member),
                };
                block.bindings.push(binding);
                var
            }
            syn::Expr::Call(syn::ExprCall { func, args, .. }) => {
                let args: Vec<Var> = args.iter().map(|e| self.parse_expr(e, block)).collect();
                let func = self.get_ident(func);
                let var = self.gen_var();
                let binding = ir::Let {
                    var: var.clone(),
                    val: ir::Expr::Call(
                        ir::Func::Named {
                            path: func,
                            is_method: false,
                        },
                        args,
                    ),
                };
                block.bindings.push(binding);
                var
            }
            syn::Expr::MethodCall(syn::ExprMethodCall {
                args,
                method,
                receiver,
                ..
            }) => {
                let mut args: Vec<Var> = args.iter().map(|e| self.parse_expr(e, block)).collect();
                let r = self.parse_expr(&*receiver, block);
                args.insert(0, r);
                let func = Path(vec![method.to_string()]);
                let var = self.gen_var();
                let binding = ir::Let {
                    var: var.clone(),
                    val: ir::Expr::Call(
                        ir::Func::Named {
                            path: func,
                            is_method: true,
                        },
                        args,
                    ),
                };
                block.bindings.push(binding);
                var
            }
            syn::Expr::Tuple(syn::ExprTuple { elems, .. }) => {
                let elems: Vec<Var> = elems.iter().map(|e| self.parse_expr(e, block)).collect();
                let var = self.gen_var();
                let binding = ir::Let {
                    var: var.clone(),
                    val: ir::Expr::Call(ir::Func::PrimFunc(PrimFunc::Tuple), elems),
                };
                block.bindings.push(binding);
                var
            }
            syn::Expr::Macro(_) => todo!(),
            syn::Expr::If(syn::ExprIf {
                cond,
                then_branch,
                else_branch,
                ..
            }) => {
                if else_branch.is_none() {
                    panic!("else branch must be present");
                }
                let cond = self.parse_expr(&**cond, block);
                let mut then = ir::Block::new();
                let mut else_ = ir::Block::new();
                self.parse_block(then_branch, &mut then);
                {
                    let r = self.parse_expr(&else_branch.as_ref().unwrap().1, &mut else_);
                    else_.ret = Some(r);
                }
                let var = self.gen_var();
                let binding = ir::Let {
                    var: var.clone(),
                    val: ir::Expr::IfThenElse {
                        cond,
                        then: Box::new(then),
                        else_: Box::new(else_),
                    },
                };
                block.bindings.push(binding);
                var
            }
            syn::Expr::Block(syn::ExprBlock { block: block_, .. }) => {
                let mut b = ir::Block::new();
                self.parse_block(block_, &mut b);
                let v = b.ret.clone().unwrap();
                for i in b.bindings {
                    block.bindings.push(i);
                }
                v
            }
            _ => todo!(),
        }
    }
    fn parse_block(&mut self, b: &syn::Block, ir_block: &mut ir::Block) {
        self.sym.push_scope();
        for stmt in &b.stmts {
            match stmt {
                syn::Stmt::Local(syn::Local { pat, init, .. }) => match pat {
                    syn::Pat::Ident(ident) => {
                        if ident.by_ref.is_some() {
                            panic!("arg must not pass by ref");
                        }
                        if ident.mutability.is_some() {
                            panic!("arg must not be mut");
                        }

                        let init = &init.as_ref().unwrap().1;
                        let init = self.parse_expr(&**init, ir_block);
                        let var = Path(vec![ident.ident.to_string()]);
                        let var = self.add_named_var(var);
                        ir_block.bindings.push(ir::Let {
                            var,
                            val: ir::Expr::Var(init),
                        })
                    }
                    syn::Pat::Type(syn::PatType { pat, ty, .. }) => match &**pat {
                        syn::Pat::Ident(ident) => {
                            if ident.by_ref.is_some() {
                                panic!("arg must not pass by ref");
                            }
                            if ident.mutability.is_some() {
                                panic!("arg must not be mut");
                            }

                            let init = &init.as_ref().unwrap().1;
                            let init = self.parse_expr(&**init, ir_block);
                            let var = Path(vec![ident.ident.to_string()]);
                            let var = self.add_named_var(var);
                            ir_block.bindings.push(ir::Let {
                                var,
                                val: ir::Expr::Var(init),
                            })
                        }
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                },
                syn::Stmt::Item(_) => {}
                syn::Stmt::Expr(e) => {
                    let v = self.parse_expr(e, ir_block);
                    if ir_block.ret.is_some() {
                        panic!("multiple implicit return per block")
                    }
                    ir_block.ret = Some(v);
                }
                syn::Stmt::Semi(e, _) => match e {
                    syn::Expr::Assign(syn::ExprAssign { left, right, .. }) => {
                        let rhs = self.parse_expr(&**right, ir_block);
                        let left = self.get_ident(&**left);
                        let left = self.add_named_var(left);
                        let binding = ir::Let {
                            var: left,
                            val: ir::Expr::Var(rhs),
                        };
                        ir_block.bindings.push(binding);
                    }
                    _ => unreachable!(),
                },
            }
        }
        self.sym.pop_scope();
        if ir_block.ret.is_none() {
            panic!("block must return value!")
        }
    }
}

pub fn parse_str(s: &str) -> Function {
    let st: syn::ItemFn = syn::parse_str(s).unwrap();
    let sig = &st.sig;
    let name = Token::from_ident(&sig.ident);
    let sig_params = &sig.inputs;
    let mut params: Vec<Parameter> = vec![];
    let mut ctx = ParseContext {
        sym: SymbolTable::new(),
        var_gen: 0,
    };
    for arg in sig_params {
        match arg {
            syn::FnArg::Receiver(_) => todo!(),
            syn::FnArg::Typed(syn::PatType { pat, ty, .. }) => match &**pat {
                syn::Pat::Ident(ident) => {
                    if ident.by_ref.is_some() {
                        panic!("arg must not pass by ref");
                    }
                    if ident.mutability.is_some() {
                        panic!("arg must not be mut");
                    }
                    let p = ctx.add_named_var(Path(vec![ident.ident.to_string()]));
                    params.push(Parameter {
                        name: p,
                        ty: syn_ty_to_ir_ty(ty),
                        requires_gradient: true,
                    });
                }
                _ => unreachable!(),
            },
        }
    }
    let mut block = ir::Block::new();

    ctx.parse_block(&st.block, &mut block);
    Function {
        name,
        parameters: params,
        body: block,
        ret: syn_ty_to_ir_ty(match &st.sig.output {
            syn::ReturnType::Default => panic!("function must return value!"),
            syn::ReturnType::Type(_, ty) => &**ty,
        }),
    }
}
