use super::arithmetic::ArithmeticNodeInput;
use crate::{InputStack, PossibleInputs};
use std::any::Any;

#[derive(Debug, Copy, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RatioNode;

impl crate::NodeInput for RatioNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        use crate::FromAnyProto;
        use once_cell::sync::Lazy;
        static CACHE: Lazy<PossibleInputs> =
            Lazy::new(|| ArithmeticNodeInput::possible_inputs(&["numerator", "denominator"]));
        PossibleInputs::new(&*CACHE.groups)
    }
}

impl crate::NodeOutput for RatioNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        crate::FromAnyProto::from_any(InputStack::new(inputs, ..)).map(ArithmeticNodeInput::ratio)
    }
}

#[typetag::serde]
impl crate::Node for RatioNode {
    fn name(&self) -> &'static str {
        "ratio"
    }
}
