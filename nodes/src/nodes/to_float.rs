use crate::{FromAny, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use std::any::Any;

enum Input {
    U32(OneOrMany<u32>),
    F32(OneOrMany<f32>),
}

impl Input {
    fn op(self) -> Box<dyn Any> {
        use crate::one_many::op1;
        let v = match self {
            Input::U32(v) => op1(v, |v| v as f32),
            Input::F32(v) => v,
        };
        match v {
            OneOrMany::One(v) => Box::new(v),
            OneOrMany::Many(v) => Box::new(v),
        }
    }
}

impl FromAny for Input {
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
        if let Some(input) = inputs.get(0) {
            if OneOrMany::<f32>::is(&**input) {
                let inner = OneOrMany::<f32>::downcast(inputs.remove(0)).unwrap();
                Ok(Self::F32(inner))
            } else if OneOrMany::<u32>::is(&**input) {
                let inner = OneOrMany::<u32>::downcast(inputs.remove(0)).unwrap();
                Ok(Self::U32(inner))
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

impl Input {
    fn inputs() -> PossibleInputs<'static> {
        use crate::InputGroup;
        use once_cell::sync::Lazy;
        static GROUPS: Lazy<Vec<InputGroup>> = Lazy::new(|| {
            let float = OneOrMany::<f32>::type_ids();
            let unsigned = OneOrMany::<u32>::type_ids();
            std::array::IntoIter::new(float)
                .map(|type_id| {
                    use crate::InputInfo;
                    InputGroup {
                        info: vec![InputInfo {
                            name: "number",
                            ty_name: "f32",
                            type_id,
                        }]
                        .into(),
                    }
                })
                .chain(std::array::IntoIter::new(unsigned).map(|type_id| {
                    use crate::InputInfo;
                    InputGroup {
                        info: vec![InputInfo {
                            name: "number",
                            ty_name: "u32",
                            type_id,
                        }]
                        .into(),
                    }
                }))
                .collect()
        });

        PossibleInputs { groups: &*GROUPS }
    }
}

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ToFloatNode;

impl NodeInput for ToFloatNode {
    fn inputs(&self) -> PossibleInputs {
        Input::inputs()
    }
}

impl NodeOutput for ToFloatNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        Input::from_any(inputs).map(Input::op)
    }
}

#[typetag::serde]
impl Node for ToFloatNode {
    fn name(&self) -> &'static str {
        "to float"
    }
}
