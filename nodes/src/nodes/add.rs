use crate::{generic::GenericPair, FromAny, Node, NodeInput, NodeOutput, PossibleInputs};
use std::any::Any;

#[derive(Debug, Clone)]
enum AddNodeInput {
    F32(GenericPair<f32, f32>),
    U32(GenericPair<u32, u32>),
}

impl AddNodeInput {
    fn op(self) -> Box<dyn Any> {
        match self {
            AddNodeInput::F32(group) => group.op(std::ops::Add::add),
            AddNodeInput::U32(group) => group.op(std::ops::Add::add),
        }
    }

    fn types() -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static INPUTS: Lazy<PossibleInputs> = Lazy::new(|| {
            static GROUPS: Lazy<Vec<crate::InputGroup>> = Lazy::new(|| {
                let float = GenericPair::<f32, f32>::gen_groups("lhs", "rhs");
                let unsigned = GenericPair::<u32, u32>::gen_groups("lhs", "rhs");
                let mut acc = Vec::new();
                acc.extend_from_slice(&float);
                acc.extend_from_slice(&unsigned);
                acc
            });

            PossibleInputs { groups: &*GROUPS }
        });
        *INPUTS
    }
}

impl FromAny for AddNodeInput {
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
        if let Ok(output) = GenericPair::<f32, f32>::from_any(inputs) {
            Ok(AddNodeInput::F32(output))
        } else if let Ok(output) = GenericPair::<u32, u32>::from_any(inputs) {
            Ok(AddNodeInput::U32(output))
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Copy, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AddNode;

impl NodeInput for AddNode {
    fn inputs(&self) -> PossibleInputs {
        AddNodeInput::types()
    }
}

impl NodeOutput for AddNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        AddNodeInput::from_any(inputs).map(AddNodeInput::op)
    }
}

#[typetag::serde]
impl Node for AddNode {
    fn name(&self) -> &'static str {
        "add"
    }
}
