use crate::{Many, Node, NodeInput, NodeOutput, One, Pair};
use std::any::Any;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct RatioGroup<T>
where
    T: 'static,
{
    one_one: Pair<One<T>, One<T>>,
    one_many: Pair<One<T>, Many<T>>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RatioNode {
    f32: RatioGroup<f32>,
    u32: RatioGroup<u32>,
}

impl<T> NodeInput for RatioGroup<T>
where
    T: 'static,
{
    fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool {
        self.one_one.inputs_match(inputs) || self.one_many.inputs_match(inputs)
    }
}

impl<T> NodeOutput for RatioGroup<T>
where
    T: std::ops::Rem<Output = T> + Into<f64> + Copy + 'static,
{
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        fn ratio<T>(length: T, count: T) -> f32
        where
            T: std::ops::Rem<Output = T> + Into<f64> + Copy,
        {
            (count.rem(length).into() / length.into()) as f32
        }

        if self.one_one.inputs_match(inputs) {
            let count = inputs.remove(1).downcast::<One<T>>().unwrap();
            let length = inputs.remove(0).downcast::<One<T>>().unwrap();
            Ok(Box::new(One(ratio(length.0, count.0))))
        } else if self.one_many.inputs_match(inputs) {
            let count = inputs.remove(1).downcast::<Many<T>>().unwrap();
            let length = inputs.remove(0).downcast::<One<T>>().unwrap();
            let out = count.0.map(move |count| ratio(length.0, count));
            Ok(Box::new(Many(Box::new(out))))
        } else {
            Err(())
        }
    }
}

impl NodeInput for RatioNode {
    fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool {
        self.f32.inputs_match(inputs) || self.u32.inputs_match(inputs)
    }
}

impl NodeOutput for RatioNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        RatioGroup::<f32>::default()
            .op(inputs)
            .or_else(|_e| RatioGroup::<u32>::default().op(inputs))
    }
}

#[typetag::serde]
impl Node for RatioNode {}
