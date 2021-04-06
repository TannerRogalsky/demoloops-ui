use super::arithmetic::ArithmeticNodeInput;
use crate::{InputStack, PossibleInputs};
use std::any::Any;

#[derive(Debug, Copy, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AddNode;

impl crate::NodeInput for AddNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        ArithmeticNodeInput::types()
    }
}

impl crate::NodeOutput for AddNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        crate::FromAnyProto::from_any(InputStack::new(inputs, ..)).map(ArithmeticNodeInput::add)
    }
}

#[typetag::serde]
impl crate::Node for AddNode {
    fn name(&self) -> &'static str {
        "add"
    }
}
