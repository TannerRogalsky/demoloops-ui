use crate::{Many, Node, NodeInput, NodeOutput, One};
use std::any::Any;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RangeNode {
    u32: std::marker::PhantomData<One<u32>>,
}

impl NodeInput for RangeNode {
    fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool {
        if inputs.len() == 1 {
            inputs[0].is::<One<u32>>()
        } else {
            false
        }
    }
}

impl NodeOutput for RangeNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        if self.inputs_match(&inputs) {
            let length = inputs.remove(0);
            let length = length.downcast::<One<u32>>().unwrap();
            Ok(Box::new(Into::<Many<u32>>::into(0u32..(length.0))))
        } else {
            Err(())
        }
    }
}

#[typetag::serde]
impl Node for RangeNode {}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Range2DNode {
    x: RangeNode,
    y: RangeNode,
}

impl NodeInput for Range2DNode {
    fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool {
        if inputs.len() == 2 {
            self.x.inputs_match(&inputs[..1]) && self.y.inputs_match(&inputs[1..])
        } else {
            false
        }
    }
}

impl NodeOutput for Range2DNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        if self.inputs_match(&inputs) {
            let y = inputs.pop().unwrap().downcast::<One<u32>>().unwrap().0;
            let x = inputs.pop().unwrap().downcast::<One<u32>>().unwrap().0;
            Ok(Box::new(Into::<Many<u32>>::into(
                (0..x).flat_map(move |vx| (0..y).map(move |vy| vy + vx * y)),
            )))
        } else {
            Err(())
        }
    }
}

#[typetag::serde]
impl Node for Range2DNode {}
