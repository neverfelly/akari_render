use crate::runtime::AdContext;

pub trait AddAdjoint<Rhs>: Sized {
    type Output;
    fn add_adjoint(self, rhs: Rhs, ctx: &mut AdContext) -> Self::Output;
}

pub trait SubAdjoint<Rhs>: Sized {
    type Output;
    fn sub_adjoint(self, rhs: Rhs, ctx: &mut AdContext) -> Self::Output;
}

pub trait MulAdjoint<Rhs>: Sized {
    type Output;
    fn mul_adjoint(self, rhs: Rhs, ctx: &mut AdContext) -> Self::Output;
}

pub trait DivAdjoint<Rhs>: Sized {
    type Output;
    fn div_adjoint(self, rhs: Rhs, ctx: &mut AdContext) -> Self::Output;
}
