use crate::{FromAnyProto, InputComponent, InputStack, Many, OneOrMany, PossibleInputs};
use std::any::Any;

#[derive(FromAnyProto, InputComponent)]
struct RangeNodeInput {
    length: OneOrMany<u32>,
}

impl RangeNodeInput {
    fn op(self) -> Box<dyn Any> {
        let iter = match self.length {
            OneOrMany::One(length) => Many::from(0..length.inner()),
            OneOrMany::Many(length) => {
                let iter = length.inner();
                Many::from(iter.clone().flat_map(move |_| iter.clone()))
            }
        };
        Box::new(iter)
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RangeNode;

impl crate::NodeInput for RangeNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static CACHE: Lazy<PossibleInputs> =
            Lazy::new(|| RangeNodeInput::possible_inputs(&["length"]));
        PossibleInputs::new(&*CACHE.groups)
    }
}

impl crate::NodeOutput for RangeNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        FromAnyProto::from_any(InputStack::new(inputs, ..)).map(RangeNodeInput::op)
    }
}

#[typetag::serde]
impl crate::Node for RangeNode {
    fn name(&self) -> &'static str {
        "range"
    }
}
