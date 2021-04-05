use crate::{FromAny, Many, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use std::any::Any;

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

    fn types() -> PossibleInputs<'static> {
        use crate::{InputGroup, InputInfo};
        use once_cell::sync::Lazy;
        static GROUPS: Lazy<Vec<InputGroup>> = Lazy::new(|| {
            std::array::IntoIter::new(OneOrMany::<u32>::type_ids())
                .map(|type_id| InputGroup {
                    info: vec![InputInfo {
                        name: "length".into(),
                        ty_name: "u32",
                        type_id,
                    }]
                    .into(),
                })
                .collect()
        });
        PossibleInputs::new(&*GROUPS)
    }
}

impl FromAny for RangeNodeInput {
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
        if let Some(input) = inputs.get(0) {
            if OneOrMany::<u32>::is(&**input) {
                let length = OneOrMany::<u32>::downcast(inputs.remove(0)).unwrap();
                Ok(Self { length })
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RangeNode;

impl NodeInput for RangeNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        RangeNodeInput::types()
    }
}

impl NodeOutput for RangeNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        RangeNodeInput::from_any(inputs).map(RangeNodeInput::op)
    }
}

#[typetag::serde]
impl Node for RangeNode {
    fn name(&self) -> &'static str {
        "range"
    }
}
