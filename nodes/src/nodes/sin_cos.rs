use crate::{FromAny, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use std::any::Any;

struct Input {
    inner: OneOrMany<f32>,
}

impl FromAny for Input {
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
        if let Some(input) = inputs.get(0) {
            if OneOrMany::<f32>::is(&**input) {
                let inner = OneOrMany::<f32>::downcast(inputs.remove(0)).unwrap();
                Ok(Self { inner })
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
            let types = OneOrMany::<f32>::type_ids();
            std::array::IntoIter::new(types)
                .map(|type_id| {
                    use crate::InputInfo;
                    InputGroup {
                        info: vec![InputInfo {
                            name: "f32",
                            ty_name: "f32",
                            type_id,
                        }]
                        .into(),
                    }
                })
                .collect()
        });

        PossibleInputs { groups: &*GROUPS }
    }

    fn sin(self) -> Box<dyn Any> {
        match crate::one_many::op1(self.inner, f32::sin) {
            OneOrMany::One(v) => Box::new(v),
            OneOrMany::Many(v) => Box::new(v),
        }
    }

    fn cos(self) -> Box<dyn Any> {
        match crate::one_many::op1(self.inner, f32::cos) {
            OneOrMany::One(v) => Box::new(v),
            OneOrMany::Many(v) => Box::new(v),
        }
    }
}

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SineNode;

impl NodeInput for SineNode {
    fn inputs(&self) -> PossibleInputs {
        Input::inputs()
    }
}

impl NodeOutput for SineNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        Input::from_any(inputs).map(Input::sin)
    }
}

#[typetag::serde]
impl Node for SineNode {
    fn name(&self) -> &'static str {
        "sine"
    }
}

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CosNode;

impl NodeInput for CosNode {
    fn inputs(&self) -> PossibleInputs {
        Input::inputs()
    }
}

impl NodeOutput for CosNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        Input::from_any(inputs).map(Input::cos)
    }
}

#[typetag::serde]
impl Node for CosNode {
    fn name(&self) -> &'static str {
        "cosine"
    }
}
