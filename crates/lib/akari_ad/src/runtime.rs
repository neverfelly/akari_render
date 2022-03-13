use std::marker::PhantomData;

pub struct AdContext {
    list: Vec<AdFrame>,
    alloc: bumpalo::Bump,
}

pub struct AdFrame {
    pub buffer: *mut u8,
    pub inputs: &'static [*mut f32],
    pub outputs: &'static [*mut f32],
    pub d_inputs: &'static [*mut f32],
    pub d_outputs: &'static [*mut f32],
    pub propagator: &'static dyn BackPropagator,
}
impl AdFrame {
    pub unsafe fn zero_grad(&mut self) {
        self.d_inputs.iter().for_each(|x| std::ptr::write(*x, 0.0));
        self.d_outputs.iter().for_each(|x| std::ptr::write(*x, 0.0));
        self.propagator.zero_grad(self);
    }
}

pub unsafe trait BackPropagator {
    fn zero_grad(&self, ctx: &mut AdFrame);
    fn backward(&self, ctx: &mut AdFrame);
}
pub trait AdFunction<I, O> {
    fn forward(&'static self, ctx: &mut AdContext, input: I) -> (O, AdFrame);
}

pub trait Adjoint: Copy {
    fn zero() -> Self;
}
pub trait ToAdjoint {
    type Output: Adjoint;
    fn to_adjoint(&self, ctx: &mut AdContext) -> Self::Output;
}
impl AdContext {
    pub fn new() -> Self {
        Self {
            list: vec![],
            alloc: bumpalo::Bump::new(),
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
    pub fn push<I, O, F: AdFunction<I, O>>(&mut self, f: &'static F, input: I) -> O {
        let (output, frame) = f.forward(self, input);
        self.list.push(frame);
        output
    }
    pub fn zero_grad(&mut self) {
        for frame in &mut self.list {
            unsafe {
                frame.zero_grad();
            }
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
        for i in (0..self.list.len()).rev() {
            let frame = &mut self.list[i];
            let propagator = frame.propagator;
            propagator.backward(frame);
        }
    }
}

#[derive(Clone, Copy)]
pub struct Dual<T: Adjoint> {
    pub primal: *mut T,
    pub adjoint: *mut T,
}
impl<T: Adjoint> Dual<T> {
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
    pub unsafe fn new(ctx: &mut AdContext, primal: T) -> Self {
        let v = &mut *ctx.alloc((primal, T::zero()));
        Self {
            primal: &mut v.0 as *mut T,
            adjoint: &mut v.1 as *mut T,
        }
    }
}
impl Adjoint for f32 {
    fn zero() -> Self {
        0.0
    }
}
mod test {
    use super::*;
    fn sqr(x: f32) -> f32 {
        x * x
    }
    fn sin(x: f32) -> f32 {
        x.sin()
    }
    struct Sqr {}

    static SQR: Sqr = Sqr {};
    fn sqr_ad<'a>(ctx: &'a mut AdContext, input: Dual<f32>) -> Dual<f32> {
        ctx.push(&SQR, input)
    }
    impl AdFunction<Dual<f32>, Dual<f32>> for Sqr {
        fn forward(&'static self, ctx: &mut AdContext, input: Dual<f32>) -> (Dual<f32>, AdFrame) {
            unsafe {
                let output = Dual::<f32>::new(ctx, sqr(*input.primal));
                let frame = AdFrame {
                    buffer: std::ptr::null_mut(),
                    inputs: ctx.alloc_slice_copy(&[input.primal]),
                    outputs: ctx.alloc_slice_copy(&[output.primal]),
                    d_inputs: ctx.alloc_slice_copy(&[input.adjoint]),
                    d_outputs: ctx.alloc_slice_copy(&[output.adjoint]),
                    propagator: &SQR,
                };
                (output, frame)
            }
        }
    }
    unsafe impl BackPropagator for Sqr {
        fn zero_grad(&self, _ctx: &mut AdFrame) {}

        fn backward(&self, ctx: &mut AdFrame) {
            unsafe {
                let dy = &*ctx.d_outputs[0];
                let dx = &mut *ctx.d_inputs[0];
                let x = *&mut *ctx.inputs[0];
                *dx = dy * 2.0 * x;
            }
        }
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
            assert!((std::ptr::read(dualx.adjoint) - 4.0 * x.powi(3)).abs() < 1e-4);
        }
    }
}
