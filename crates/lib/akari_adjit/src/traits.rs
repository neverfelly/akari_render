use crate::runtime::Field;

pub trait Differentiable {
    fn fields() -> Vec<Field>;
    fn name() -> String;
}
