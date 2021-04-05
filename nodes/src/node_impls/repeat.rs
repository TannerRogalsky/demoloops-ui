use crate::{FromAnyProto, InputComponent, InputStack, Many, One, OneOrMany, PossibleInputs};
use std::any::Any;

#[derive(FromAnyProto, InputComponent)]
struct RepeatNodeInput {
    count: One<u32>,
    value: OneOrMany<u32>,
}

impl RepeatNodeInput {
    fn op(self) -> Box<dyn Any> {
        let count = self.count.inner();
        let iter = match self.value {
            OneOrMany::One(v) => {
                let v = v.inner();
                Many::from((0..count).map(move |_| v))
            }
            OneOrMany::Many(v) => {
                let v = v.inner();
                Many::from((0..count).flat_map(move |r| v.clone().map(move |_| r)))
            }
        };
        Box::new(iter)
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RepeatNode;

impl crate::NodeInput for RepeatNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static CACHE: Lazy<PossibleInputs> =
            Lazy::new(|| RepeatNodeInput::possible_inputs(&["count", "value"]));
        PossibleInputs::new(&*CACHE.groups)
    }
}

impl crate::NodeOutput for RepeatNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        FromAnyProto::from_any(InputStack::new(inputs, ..)).map(RepeatNodeInput::op)
    }
}

#[typetag::serde]
impl crate::Node for RepeatNode {
    fn name(&self) -> &'static str {
        "repeat"
    }
}
