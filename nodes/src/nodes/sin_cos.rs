use crate::{FromAny, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use std::any::Any;

enum Input {
    F32(OneOrMany<f32>),
    U32(OneOrMany<u32>),
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
            std::array::IntoIter::new(OneOrMany::<f32>::type_ids())
                .chain(std::array::IntoIter::new(OneOrMany::<u32>::type_ids()))
                .map(|type_id| {
                    use crate::InputInfo;
                    InputGroup {
                        info: vec![InputInfo {
                            name: "number",
                            ty_name: "number",
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
        use crate::one_many::op1;
        match self {
            Input::F32(inner) => op1(inner, f32::sin).into_boxed_inner(),
            Input::U32(inner) => op1(inner, |v| (v as f32).sin()).into_boxed_inner(),
        }
    }

    fn cos(self) -> Box<dyn Any> {
        use crate::one_many::op1;
        match self {
            Input::F32(inner) => op1(inner, f32::cos).into_boxed_inner(),
            Input::U32(inner) => op1(inner, |v| (v as f32).cos()).into_boxed_inner(),
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
