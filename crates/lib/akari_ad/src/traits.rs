use crate::runtime::AdContext;

macro_rules! def_trait {
    ($name:ident, $method:ident) => {
        pub trait $name<Rhs>: Sized {
            type Output;
            type ResetGrad: Fn();
            type BackPropagator: Fn();
            fn $method(
                self,
                rhs: Rhs,
                ctx: &mut AdContext,
            ) -> (Self::Output, Self::ResetGrad, Self::BackPropagator);
        }
    };
}
def_trait!(AddAdjoint, add_ad);
def_trait!(SubAdjoint, sub_ad);
def_trait!(MulAdjoint, mul_ad);
def_trait!(DivAdjoint, div_ad);
// def_trait!(AddAdjoint, add_ad);
// def_trait!(AddAdjoint, add_ad);

// pub trait SubAdjoint<Rhs>: Sized {
//     type Output;
//     fn sub_adjoint(self, rhs: Rhs, ctx: &mut AdContext) -> Self::Output;
// }

// pub trait MulAdjoint<Rhs>: Sized {
//     type Output;
//     fn mul_adjoint(self, rhs: Rhs, ctx: &mut AdContext) -> Self::Output;
// }

// pub trait DivAdjoint<Rhs>: Sized {
//     type Output;
//     fn div_adjoint(self, rhs: Rhs, ctx: &mut AdContext) -> Self::Output;
// }
