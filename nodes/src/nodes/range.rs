use crate::{
    FromAny, InputGroup, InputInfo, Many, Node, NodeInput, NodeOutput, One, PossibleInputs,
};
use std::any::Any;

#[derive(Copy, Clone)]
struct RangeNodeInput {
    length: One<u32>,
}

impl RangeNodeInput {
    fn op(self) -> Box<dyn Any> {
        Box::new(Many::from(0..self.length.inner()))
    }

    fn types() -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static GROUPS: Lazy<Vec<InputGroup>> = Lazy::new(|| {
            static INFO: Lazy<Vec<InputInfo>> = Lazy::new(|| {
                vec![InputInfo {
                    name: "length",
                    ty_name: "One<u32>",
                    type_id: std::any::TypeId::of::<One<u32>>(),
                }]
            });
            vec![InputGroup {
                info: (&*INFO).into(),
            }]
        });
        PossibleInputs { groups: &*GROUPS }
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
    fn inputs(&self) -> PossibleInputs {
        RangeNodeInput::types()
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
    fn op(self) -> Box<dyn Any> {
        let y = self.y.length.inner();
        let iter = (0..self.x.length.inner()).flat_map(move |vx| (0..y).map(move |vy| vy + vx * y));
        Box::new(Many::from(iter))
    }

    fn types() -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static GROUPS: Lazy<Vec<InputGroup>> = Lazy::new(|| {
            static INFO: Lazy<Vec<InputInfo>> = Lazy::new(|| {
                vec![
                    InputInfo {
                        name: "x",
                        ty_name: "One<f32>",
                        type_id: std::any::TypeId::of::<One<f32>>(),
                    },
                    InputInfo {
                        name: "y",
                        ty_name: "One<f32>",
                        type_id: std::any::TypeId::of::<One<f32>>(),
                    },
                ]
            });
            vec![InputGroup {
                info: (&*INFO).into(),
            }]
        });
        PossibleInputs { groups: &*GROUPS }
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
    fn inputs(&self) -> PossibleInputs {
        Range2DNodeInput::types()
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
