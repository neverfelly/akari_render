use std::{
    marker::PhantomData,
    ops::{Add, AddAssign},
};

pub struct AdContext {
    alloc: bumpalo::Bump,
    reset_grads: Vec<&'static dyn Fn()>,
    back_propagators: Vec<&'static dyn Fn()>,
}

// pub trait AdFunction<I, O> {
//     fn forward(&'static self, ctx: &mut AdContext, input: I) -> (O, AdFrame);
// }

pub trait Differentiable: Copy + AddAssign {
    fn zero() -> Self;
}
pub trait ToAdjoint {
    type Output: Differentiable;
    fn to_adjoint(&self, ctx: &mut AdContext) -> Self::Output;
}
impl AdContext {
    pub fn new() -> Self {
        Self {
            back_propagators: vec![],
            alloc: bumpalo::Bump::new(),
            reset_grads: vec![],
        }
    }
    pub unsafe fn alloc<T>(&mut self, val: T) -> *mut T {
        self.alloc.alloc(val)
    }
    pub unsafe fn alloc_slice(&mut self, count: usize) -> &'static [*mut f32] {
        let s = self
            .alloc
            .alloc_slice_fill_with(count, |_| std::ptr::null_mut::<f32>());
        std::mem::transmute(s)
    }
    pub unsafe fn alloc_slice_copy(&mut self, data: &[*mut f32]) -> &'static [*mut f32] {
        let s = self.alloc.alloc_slice_copy(data);
        std::mem::transmute(s)
    }
    // pub fn push<I, O, F: AdFunction<I, O>>(&mut self, f: &'static F, input: I) -> O {
    //     let (output, frame) = f.forward(self, input);
    //     self.list.push(frame);
    //     output
    // }
    pub fn push_reset_grad_func<F: Fn()>(&mut self, f: F) {
        let f = self.alloc.alloc(f) as &F as &dyn Fn();
        unsafe {
            let f: &'static dyn Fn() = std::mem::transmute(f);
            self.reset_grads.push(f);
        }
    }
    pub fn push_back_propagator<F: Fn()>(&mut self, f: F) {
        let f = self.alloc.alloc(f) as &F as &dyn Fn();
        unsafe {
            let f: &'static dyn Fn() = std::mem::transmute(f);
            self.back_propagators.push(f);
        }
    }
    pub fn zero_grad(&mut self) {
        for x in &self.reset_grads {
            (x)();
        }
    }
    pub fn backward(&mut self) {
        // unsafe {
        //     let last = self.list.last_mut().unwrap();
        //     last.d_outputs
        //         .iter()
        //         .zip(gradients.iter())
        //         .for_each(|(x, y)| std::ptr::write(*x, *y));
        // }
        for bp in self.back_propagators.iter().rev() {
            (bp)();
        }
    }
    pub unsafe fn reset(&mut self) {
        self.reset_grads.clear();
        self.back_propagators.clear();
        self.alloc.reset();
    }
}

#[derive(Clone, Copy)]
pub struct Dual<T: Differentiable> {
    pub primal: *mut T,
    pub adjoint: *mut T,
}
impl<T: Differentiable> Dual<T> {
    pub unsafe fn primal(&self) -> &T {
        &*self.primal
    }
    pub unsafe fn primal_mut(&self) -> &mut T {
        &mut *self.primal
    }
    pub unsafe fn adjoint(&self) -> &T {
        &*self.adjoint
    }
    pub unsafe fn adjoint_mut(&self) -> &mut T {
        &mut *self.adjoint
    }
    pub unsafe fn add_gradient(&self, gradient: T) {
        *self.adjoint_mut() += gradient;
    }
    pub unsafe fn new(ctx: &mut AdContext, primal: T) -> Self {
        let v = &mut *ctx.alloc((primal, T::zero()));
        Self {
            primal: &mut v.0 as *mut T,
            adjoint: &mut v.1 as *mut T,
        }
    }
    pub unsafe fn reset_grad(&self) {
        std::ptr::write(self.adjoint, T::zero());
    }
    pub unsafe fn zero(ctx: &mut AdContext) -> Self {
        Self::new(ctx, T::zero())
    }
    // pub unsafe fn assign(&self, rhs: Dual<T>) {
    //     std::ptr::copy(rhs.primal, self.primal, 1);
    //     std::ptr::copy(rhs.adjoint, self.adjoint, 1);
    // }
    pub unsafe fn set_primal(&self, rhs: T) {
        std::ptr::write(self.primal, rhs);
    }
}
impl Differentiable for f32 {
    fn zero() -> Self {
        0.0
    }
}

pub enum Two<A, B> {
    A(A),
    B(B),
}
pub unsafe fn lift_lifetime<'a, T>(x: &'a T) -> &'static T {
    std::mem::transmute(x)
}

#[cfg(test)]
mod test {
    use super::*;
    fn sqr(x: f32) -> f32 {
        x * x
    }
    fn _sqr_ad(ctx: &mut AdContext, x: Dual<f32>) -> (Dual<f32>, impl Fn(), impl Fn()) {
        unsafe {
            let y = Dual::new(ctx, *x.primal() * *x.primal());
            let reset_grad = move || {
                y.reset_grad();
            };
            let back_propagator = move || x.add_gradient(2.0 * *x.primal() * *y.adjoint());
            (y, reset_grad, back_propagator)
        }
    }
    fn sqr_ad(ctx: &mut AdContext, x: Dual<f32>) -> Dual<f32> {
        let (ret, reset_grad, back_propagator) = _sqr_ad(ctx, x);
        ctx.push_reset_grad_func(reset_grad);
        ctx.push_back_propagator(back_propagator);
        ret
    }
    fn pow4(x: f32) -> f32 {
        sqr(sqr(x))
    }
    fn _pow4_ad(ctx: &mut AdContext, x: Dual<f32>) -> (Dual<f32>, impl Fn(), impl Fn()) {
        unsafe {
            // let mut x2: Dual<_> = Dual::zero(ctx);
            // let mut x4: Dual<_> = Dual::zero(ctx);
            let (x2, reset_grad0, back_propagator0) = _sqr_ad(ctx, x);
            // x2 = ret;
            let (x4, reset_grad1, back_propagator1) = _sqr_ad(ctx, x2);
            // x4 = ret;
            let reset_grad = move || {
                x2.reset_grad();
                x4.reset_grad();
                reset_grad0();
                reset_grad1();
            };
            let back_propagator = move || {
                back_propagator1();
                back_propagator0();
            };
            (x4, reset_grad, back_propagator)
        }
    }
    fn pow4_ad(ctx: &mut AdContext, x: Dual<f32>) -> Dual<f32> {
        let (ret, reset_grad, back_propagator) = _pow4_ad(ctx, x);
        ctx.push_reset_grad_func(reset_grad);
        ctx.push_back_propagator(back_propagator);
        ret
    }
    fn g(x: f32) -> f32 {
        if x > 1.0 {
            pow4(x)
        } else {
            sqr(x)
        }
    }
    fn _g_ad(ctx: &mut AdContext, x: Dual<f32>) -> (Dual<f32>, impl Fn(), impl Fn()) {
        unsafe {
            let tmp = if *x.primal() > 1.0 {
                Two::A(_pow4_ad(ctx, x))
            } else {
                Two::B(_sqr_ad(ctx, x))
            };
            let ret = match tmp {
                Two::A((ret, ..)) => ret,
                Two::B((ret, ..)) => ret,
            };
            let tmp = lift_lifetime(&*ctx.alloc(tmp));

            let reset_grad = move || match &tmp {
                Two::A((_, reset_grad, ..)) => (reset_grad)(),
                Two::B((_, reset_grad, ..)) => (reset_grad)(),
            };
            let back_propagator = move || match &tmp {
                Two::A((_, _, back_propagator)) => (back_propagator)(), 
                Two::B((_, _, back_propagator)) => (back_propagator)(),
            };
            (ret, reset_grad, back_propagator)
        }
    }
    fn g_ad(ctx: &mut AdContext, x: Dual<f32>) -> Dual<f32> {
        let (ret, reset_grad, back_propagator) = _g_ad(ctx, x);
        ctx.push_reset_grad_func(reset_grad);
        ctx.push_back_propagator(back_propagator);
        ret
    }
    #[test]
    fn test_sqr() {
        let x = 2.0;
        let y = sqr(x);
        let mut ctx = AdContext::new();
        unsafe {
            let dualx = Dual::<f32>::new(&mut ctx, x);
            let dualy = sqr_ad(&mut ctx, dualx);

            std::ptr::write(dualy.adjoint, 1.0);

            ctx.backward();
            assert!((y - 4.0).abs() < 1e-4);
            assert!((std::ptr::read(dualx.adjoint) - 2.0 * x).abs() < 1e-4);
        }
    }

    #[test]
    fn test_sqr2() {
        let x = 2.0;
        let y = sqr(sqr(x));
        let mut ctx = AdContext::new();
        unsafe {
            let dualx = Dual::<f32>::new(&mut ctx, x);
            let dualy = sqr_ad(&mut ctx, dualx);
            let dualy = sqr_ad(&mut ctx, dualy);

            std::ptr::write(dualy.adjoint, 1.0);

            ctx.backward();
            assert!((y - 16.0).abs() < 1e-4);
            assert!(
                (std::ptr::read(dualx.adjoint) - 4.0 * x.powi(3)).abs() < 1e-4,
                "{} != {}",
                std::ptr::read(dualx.adjoint),
                4.0 * x.powi(3)
            );
        }
    }
    #[test]
    fn test_pow4() {
        let x = 2.0;
        let y = sqr(sqr(x));
        let mut ctx = AdContext::new();
        unsafe {
            let dualx = Dual::<f32>::new(&mut ctx, x);
            let dualy = pow4_ad(&mut ctx, dualx);

            std::ptr::write(dualy.adjoint, 1.0);

            ctx.backward();
            assert!((y - 16.0).abs() < 1e-4);
            assert!(
                (std::ptr::read(dualx.adjoint) - 4.0 * x.powi(3)).abs() < 1e-4,
                "{} != {}",
                std::ptr::read(dualx.adjoint),
                4.0 * x.powi(3)
            );
        }
    }
    #[test]
    fn test_cond1() {
        let x = 2.0;
        let y = g(x);
        let mut ctx = AdContext::new();
        unsafe {
            let dualx = Dual::<f32>::new(&mut ctx, x);
            let dualy = g_ad(&mut ctx, dualx);

            std::ptr::write(dualy.adjoint, 1.0);

            ctx.backward();
            assert!((y - 16.0).abs() < 1e-4);
            assert!(
                (std::ptr::read(dualx.adjoint) - 4.0 * x.powi(3)).abs() < 1e-4,
                "{} != {}",
                std::ptr::read(dualx.adjoint),
                4.0 * x.powi(3)
            );
        }
    }
    #[test]
    fn test_cond2() {
        let x = 0.5;
        let y = g(x);
        let mut ctx = AdContext::new();
        unsafe {
            let dualx = Dual::<f32>::new(&mut ctx, x);
            let dualy = g_ad(&mut ctx, dualx);

            std::ptr::write(dualy.adjoint, 1.0);

            ctx.backward();
            assert!((y - 0.25).abs() < 1e-4);
            assert!(
                (std::ptr::read(dualx.adjoint) - 2.0 * x).abs() < 1e-4,
                "{} != {}",
                std::ptr::read(dualx.adjoint),
                2.0 * x
            );
        }
    }
}
