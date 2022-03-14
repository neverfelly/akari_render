use std::{collections::HashMap, fmt::Write};
struct VarRecord {
    is_const: bool, // if const no need to accum gradient
}
use crate::{
    ir::{self, Expr, Func, Let, Path, Var},
    parse::{Function, Token},
};
pub struct AdCodeGen {
    symbols: HashMap<Var, VarRecord>,
    def_list: Vec<Var>,
}
impl AdCodeGen {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            def_list: vec![],
        }
    }
    fn add_vars(&mut self, block: &ir::Block) {
        for binding in &block.bindings {
            let Let { val, var } = binding;
            match val {
                Expr::IfThenElse { then, else_, .. } => {
                    self.add_vars(then);
                    self.add_vars(else_);
                }
                _ => {}
            }
            let is_const = match val {
                Expr::Atom(a) => match a {
                    ir::Atom::Literal(_, _) => true,
                    ir::Atom::String(_) => true,
                    ir::Atom::Identifier { path, constant } => {
                        *constant
                            || self
                                .symbols
                                .get(&ir::Var::Named(path.clone()))
                                .unwrap()
                                .is_const
                    }
                },
                _ => false,
            };
            if self
                .symbols
                .insert(var.clone(), VarRecord { is_const })
                .is_some()
            {
                panic!("{:?} is defined twice", var);
            }
            self.def_list.push(var.clone());
        }
    }
    pub fn gen_var(&self, var: &Var) -> String {
        match var {
            Var::Id(i) => {
                format!("t{}", i)
            }
            Var::Named(p) => {
                // p.0.join("::")
                if p.0.len() > 1 {
                    p.0.join("::")
                } else {
                    format!("_{}", p.0[0])
                }
            }
        }
    }
    pub fn gen_expr(&self, e: &Expr) -> String {
        match e {
            Expr::Atom(a) => match a {
                ir::Atom::Literal(s, _) => format!("Dual::new(ctx, {})", s.clone()),
                ir::Atom::String(s) => s.escape_default().collect(),
                ir::Atom::Identifier { path, constant } => path.0.join("::"),
            },
            Expr::Var(v) => self.gen_var(v),
            Expr::Call(f, args) => {
                let args: Vec<_> = args
                    .iter()
                    .map(|a| format!("*{}.get()", self.gen_var(a)))
                    .collect();
                // match f {
                //     ir::Func::PrimFunc(f) => match f {
                //         ir::PrimFunc::Neg => format!("-{}", args[0]),
                //         ir::PrimFunc::Not => format!("!{}", args[0]),
                //         ir::PrimFunc::Add => format!("{} + {}", args[0], args[1]),
                //         ir::PrimFunc::Sub => format!("{} - {}", args[0], args[1]),
                //         ir::PrimFunc::Mul => format!("{} * {}",o args[0], args[1]),
                //         ir::PrimFunc::Div => format!("{} / {}", args[0], args[1]),
                //         ir::PrimFunc::Rem => format!("{} % {}", args[0], args[1]),
                //         ir::PrimFunc::And => format!("{} && {}", args[0], args[1]),
                //         ir::PrimFunc::Or => format!("{} || {}", args[0], args[1]),
                //         ir::PrimFunc::BitAnd => format!("{} & {}", args[0], args[1]),
                //         ir::PrimFunc::BitOr => format!("{} | {}", args[0], args[1]),
                //         ir::PrimFunc::Shl => format!("{} << {}", args[0], args[1]),
                //         ir::PrimFunc::Shr => format!("{} >> {}", args[0], args[1]),
                //         ir::PrimFunc::Eq => format!("{} == {}", args[0], args[1]),
                //         ir::PrimFunc::Ne => format!("{} != {}", args[0], args[1]),
                //         ir::PrimFunc::Lt => format!("{} <  {}", args[0], args[1]),
                //         ir::PrimFunc::Gt => format!("{} >  {}", args[0], args[1]),
                //         ir::PrimFunc::Le => format!("{} <= {}", args[0], args[1]),
                //         ir::PrimFunc::Ge => format!("{} >= {}", args[0], args[1]),
                //         ir::PrimFunc::Load => todo!(),
                //         ir::PrimFunc::Tuple => todo!(),
                //     },
                //     ir::Func::Named { path, is_method } => {
                //         format!("{}({})", path.0.join("::"), args.join(","))
                //     }
                // }
                todo!()
            }
            Expr::Extract(_, _) => todo!(),
            Expr::Insert(_, _, _) => todo!(),
            Expr::IfThenElse { cond, then, else_ } => todo!(),
        }
    }
    fn gen_block_forward(&mut self, block: &ir::Block, out: &mut String) {
        out.push_str("{\n");
        for binding in &block.bindings {
            let Let { val, var } = binding;
            let val = self.gen_expr(val);
            let var = self.gen_var(var);
            writeln!(out, "{}.assign({});", var, val).unwrap();
        }
        writeln!(out, "{}\n}}", self.gen_var(block.ret.as_ref().unwrap())).unwrap();
    }
    fn lift_type(&self, ty: &ir::Type) -> String {
        match ty {
            ir::Type::Path(p) => format!("akari_ad::runtime::Dual<{}>", p.0.join("::")),
            ir::Type::Tuple(_) => todo!(),
            ir::Type::Slice(_) => todo!(),
            ir::Type::Inferred => todo!(),
        }
    }
    fn lift_func_name(&self, p: &Token) -> String {
        // let mut s =  p.0.join("::");
        // s.push_str("_ad");
        // s
        format!("{}_ad", p.as_identifier().unwrap())
    }
    fn gen_prolog(&mut self) -> String {
        let mut s = String::new();
        for var in &self.def_list {
            writeln!(
                &mut s,
                "let {}: Dual<_> = Dual::zero(ctx);",
                self.gen_var(var)
            )
            .unwrap();
        }
        writeln!(&mut s, "{}", self.gen_reset_grad()).unwrap();
        s
    }
    fn gen_reset_grad(&mut self) -> String {
        let mut s = String::new();
        writeln!(&mut s, "let reset_grad = move || {{ unsafe {{").unwrap();
        for var in &self.def_list {
            writeln!(&mut s, "{}.reset_grad();", self.gen_var(var)).unwrap();
        }
        writeln!(&mut s, "}} }};").unwrap();
        s
    }
    pub fn gen_forward(&mut self, f: &Function) -> String {
        self.add_vars(&f.body);

        let mut out = String::new();
        let params: Vec<_> = f
            .parameters
            .iter()
            .map(|p| format!("{}: {}", self.gen_var(&p.name), self.lift_type(&p.ty)))
            .collect();
        write!(
            &mut out,
            "pub fn {}(ctx:&mut akari_ad::runtime::AdContext, {}) -> ({}, impl Fn(), impl Fn()) {{\n {} unsafe {{ {}",
            self.lift_func_name(&f.name),
            params.join(","),
            self.lift_type(&f.ret),
            "using akari_ad::runtime::*;using akari_ad::traits::*;\n",
            self.gen_prolog()
        )
        .unwrap();
        write!(&mut out, "let ret = ").unwrap();
        self.gen_block_forward(&f.body, &mut out);
        writeln!(&mut out, ";").unwrap();
        writeln!(&mut out, "(ret, reset_grad, back_propagator)}} }}").unwrap();
        out
    }
}
