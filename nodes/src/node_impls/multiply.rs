use super::arithmetic::ArithmeticNodeInput;
use crate::{InputStack, PossibleInputs};
use std::any::Any;

#[derive(Debug, Copy, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct MultiplyNode;

impl crate::NodeInput for MultiplyNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        ArithmeticNodeInput::types()
    }
}

impl crate::NodeOutput for MultiplyNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        crate::FromAnyProto::from_any(InputStack::new(inputs, ..)).map(ArithmeticNodeInput::mul)
    }
}

#[typetag::serde]
impl crate::Node for MultiplyNode {
    fn name(&self) -> &'static str {
        "multiply"
    }
}
