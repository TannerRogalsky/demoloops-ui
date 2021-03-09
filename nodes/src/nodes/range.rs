use crate::{FromAny, Many, Node, NodeInput, NodeOutput, One};
use std::any::Any;

#[derive(Copy, Clone)]
struct RangeNodeInput {
    length: One<u32>,
}

impl RangeNodeInput {
    fn can_match(inputs: &[Box<dyn Any>]) -> bool {
        if inputs.len() == 1 {
            inputs[0].is::<One<u32>>()
        } else {
            false
        }
    }

    fn op(self) -> Box<dyn Any> {
        Box::new(Many::from(0..self.length.inner()))
    }
}

impl FromAny for RangeNodeInput {
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
        if let Some(length) = inputs.pop() {
            match length.downcast::<One<u32>>() {
                Ok(length) => Ok(RangeNodeInput { length: *length }),
                Err(v) => {
                    inputs.push(v);
                    Err(())
                }
            }
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RangeNode;

impl NodeInput for RangeNode {
    fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool {
        RangeNodeInput::can_match(inputs)
    }
}

impl NodeOutput for RangeNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        RangeNodeInput::from_any(inputs).map(RangeNodeInput::op)
    }
}

#[typetag::serde]
impl Node for RangeNode {
    fn name(&self) -> &'static str {
        "range"
    }
}

#[derive(Copy, Clone)]
struct Range2DNodeInput {
    x: RangeNodeInput,
    y: RangeNodeInput,
}

impl Range2DNodeInput {
    fn can_match(inputs: &[Box<dyn Any>]) -> bool {
        if inputs.len() == 2 {
            RangeNodeInput::can_match(&inputs[..1]) && RangeNodeInput::can_match(&inputs[1..])
        } else {
            false
        }
    }

    fn op(self) -> Box<dyn Any> {
        let y = self.y.length.inner();
        let iter = (0..self.x.length.inner()).flat_map(move |vx| (0..y).map(move |vy| vy + vx * y));
        Box::new(Many::from(iter))
    }
}

impl FromAny for Range2DNodeInput {
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
        // FIXME: this doesn't repopulate the inputs properly in some cases
        if inputs.len() == 2 {
            if let Ok(x) = RangeNodeInput::from_any(inputs) {
                if let Ok(y) = RangeNodeInput::from_any(inputs) {
                    Ok(Range2DNodeInput { x, y })
                } else {
                    Err(())
                }
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Range2DNode;

impl NodeInput for Range2DNode {
    fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool {
        Range2DNodeInput::can_match(inputs)
    }
}

impl NodeOutput for Range2DNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        Range2DNodeInput::from_any(inputs).map(Range2DNodeInput::op)
    }
}

#[typetag::serde]
impl Node for Range2DNode {
    fn name(&self) -> &'static str {
        "2d range"
    }
}
