use crate::{FromAnyProto, InputComponent, InputStack, OneOrMany, PossibleInputs};
use std::any::Any;

#[derive(FromAnyProto, InputComponent)]
enum Input {
    U32(OneOrMany<u32>),
    F32(OneOrMany<f32>),
}

impl Input {
    fn op(self) -> Box<dyn Any> {
        use crate::one_many::op1;
        match self {
            Input::U32(v) => op1(v, |v| v as f32),
            Input::F32(v) => v,
        }
        .into_boxed_inner()
    }
}

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ToFloatNode;

impl crate::NodeInput for ToFloatNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static CACHE: Lazy<PossibleInputs> = Lazy::new(|| Input::possible_inputs(&["number"]));
        PossibleInputs::new(&*CACHE.groups)
    }
}

impl crate::NodeOutput for ToFloatNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        Input::from_any(InputStack::new(inputs, ..)).map(Input::op)
    }
}

#[typetag::serde]
impl crate::Node for ToFloatNode {
    fn name(&self) -> &'static str {
        "to float"
    }
}
