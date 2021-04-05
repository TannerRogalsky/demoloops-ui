use crate::{FromAnyProto, InputComponent, InputStack, OneOrMany, PossibleInputs};
use std::any::Any;

#[derive(FromAnyProto, InputComponent)]
enum Input {
    F32(OneOrMany<f32>),
    U32(OneOrMany<u32>),
}

impl Input {
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

    fn inputs() -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static CACHE: Lazy<PossibleInputs> = Lazy::new(|| Input::possible_inputs(&["number"]));
        PossibleInputs::new(&*CACHE.groups)
    }
}

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SineNode;

impl crate::NodeInput for SineNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        Input::inputs()
    }
}

impl crate::NodeOutput for SineNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        FromAnyProto::from_any(InputStack::new(inputs, ..)).map(Input::sin)
    }
}

#[typetag::serde]
impl crate::Node for SineNode {
    fn name(&self) -> &'static str {
        "sine"
    }
}

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CosNode;

impl crate::NodeInput for CosNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        Input::inputs()
    }
}

impl crate::NodeOutput for CosNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        FromAnyProto::from_any(InputStack::new(inputs, ..)).map(Input::cos)
    }
}

#[typetag::serde]
impl crate::Node for CosNode {
    fn name(&self) -> &'static str {
        "cosine"
    }
}
