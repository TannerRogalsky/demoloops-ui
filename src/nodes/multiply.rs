use crate::{Many, Node, NodeInput, NodeOutput, One, Pair};
use std::any::Any;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct MultiplyGroup<T>
where
    T: 'static,
{
    one_one: Pair<One<T>, One<T>>,
    one_many: Pair<One<T>, Many<T>>,
    many_many: Pair<Many<T>, Many<T>>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct MultiplyNode {
    f32: MultiplyGroup<f32>,
    u32: MultiplyGroup<u32>,
}

impl<A, B> NodeInput for Pair<A, B>
where
    A: 'static,
    B: 'static,
{
    fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool {
        if inputs.len() == 2 {
            inputs[0].is::<A>() && inputs[1].is::<B>()
        } else {
            false
        }
    }
}

impl<T> NodeInput for MultiplyGroup<T>
where
    T: 'static,
{
    fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool {
        self.one_one.inputs_match(inputs)
            || self.one_many.inputs_match(inputs)
            || self.many_many.inputs_match(inputs)
    }
}

impl<T> NodeOutput for MultiplyGroup<T>
where
    T: std::ops::Mul<Output = T> + 'static + Copy,
{
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        if self.one_one.inputs_match(inputs) {
            let rhs = inputs.remove(1).downcast::<One<T>>().unwrap();
            let lhs = inputs.remove(0).downcast::<One<T>>().unwrap();
            Ok(Box::new(*lhs * *rhs))
        } else if self.one_many.inputs_match(inputs) {
            let rhs = inputs.remove(1).downcast::<Many<T>>().unwrap();
            let lhs = inputs.remove(0).downcast::<One<T>>().unwrap();
            Ok(Box::new(*lhs * *rhs))
        } else if self.many_many.inputs_match(inputs) {
            let rhs = inputs.remove(1).downcast::<Many<T>>().unwrap();
            let lhs = inputs.remove(0).downcast::<Many<T>>().unwrap();
            Ok(Box::new(*lhs * *rhs))
        } else {
            Err(())
        }
    }
}

impl NodeInput for MultiplyNode {
    fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool {
        self.f32.inputs_match(inputs) || self.u32.inputs_match(inputs)
    }
}

impl NodeOutput for MultiplyNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        MultiplyGroup::<f32>::default()
            .op(inputs)
            .or_else(|_e| MultiplyGroup::<u32>::default().op(inputs))
    }
}

#[typetag::serde]
impl Node for MultiplyNode {}
