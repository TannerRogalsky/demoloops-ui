use crate::{FromAny, Many, Node, NodeInput, NodeOutput, One, OneOrMany, PossibleInputs};
use std::any::Any;

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

    fn types() -> PossibleInputs<'static> {
        use crate::{InputGroup, InputInfo};
        use once_cell::sync::Lazy;
        static GROUPS: Lazy<Vec<InputGroup>> = Lazy::new(|| {
            std::array::IntoIter::new(OneOrMany::<u32>::type_ids())
                .map(|type_id| InputGroup {
                    info: vec![
                        InputInfo {
                            name: "count".into(),
                            ty_name: "count",
                            type_id: std::any::TypeId::of::<One<u32>>(),
                        },
                        InputInfo {
                            name: "value".into(),
                            ty_name: "u32",
                            type_id,
                        },
                    ]
                    .into(),
                })
                .collect()
        });
        PossibleInputs::new(&*GROUPS)
    }
}

impl FromAny for RepeatNodeInput {
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
        if inputs.len() >= 2 {
            if (&*inputs[0]).is::<One<u32>>() && OneOrMany::<u32>::is(&*inputs[1]) {
                let mut inputs = inputs.drain(0..2);
                let count = *inputs.next().unwrap().downcast::<One<u32>>().unwrap();
                let value = OneOrMany::<u32>::downcast(inputs.next().unwrap()).unwrap();

                Ok(Self { count, value })
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RepeatNode;

impl NodeInput for RepeatNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        RepeatNodeInput::types()
    }
}

impl NodeOutput for RepeatNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        RepeatNodeInput::from_any(inputs).map(RepeatNodeInput::op)
    }
}

#[typetag::serde]
impl Node for RepeatNode {
    fn name(&self) -> &'static str {
        "repeat"
    }
}
