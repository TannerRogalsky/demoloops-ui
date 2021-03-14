use crate::{
    FromAny, InputGroup, InputInfo, InputMatchError, Many, Node, NodeInput, NodeOutput, One,
    PossibleInputs,
};
use std::any::{Any, TypeId};

#[derive(Copy, Clone)]
struct RangeNodeInput {
    length: One<u32>,
}

impl RangeNodeInput {
    fn can_match(inputs: &[Box<dyn Any>]) -> Option<InputMatchError> {
        if inputs.len() == 1 {
            if inputs[0].is::<One<u32>>() {
                None
            } else {
                Some(InputMatchError::TypeMismatch {
                    index: 0,
                    type_id: TypeId::of::<One<u32>>(),
                })
            }
        } else {
            Some(InputMatchError::LengthMismatch { desired: 1 })
        }
    }

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
            vec![InputGroup { info: &*INFO }]
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
    fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> Option<InputMatchError> {
        RangeNodeInput::can_match(inputs)
    }

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
    fn can_match(inputs: &[Box<dyn Any>]) -> Option<InputMatchError> {
        if inputs.len() == 2 {
            RangeNodeInput::can_match(&inputs[..1])
                .or_else(|| RangeNodeInput::can_match(&inputs[1..]))
        } else {
            Some(InputMatchError::LengthMismatch { desired: 2 })
        }
    }

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
            vec![InputGroup { info: &*INFO }]
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
    fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> Option<InputMatchError> {
        Range2DNodeInput::can_match(inputs)
    }

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
