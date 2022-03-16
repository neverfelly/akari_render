fn f(x: f32) -> f32 {
    let x = x+1.0;
    let x = x * x;
    x
}

/*
fn f(x: f32) -> f32 {
    x * x + x
}
fn f(x: f32) -> f32 {
    let a = x * x;
    let b = a + x
    b
}
struct F_{}
struct F_Intermediate_ {
    a:Dual<f32>,
    b:Dual<f32>,
}
static F:F_ = F_{};
impl AdFunction<Dual<f32>, Dual<f32>> for F{
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

*/