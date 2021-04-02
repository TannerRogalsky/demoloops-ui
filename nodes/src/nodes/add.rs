use crate::{FromAny, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use std::any::Any;

type F32F32 = (OneOrMany<f32>, OneOrMany<f32>);
type U32U32 = (OneOrMany<u32>, OneOrMany<u32>);

#[derive(Debug, Clone)]
enum AddNodeInput {
    F32(F32F32),
    U32(U32U32),
}

impl AddNodeInput {
    fn op(self) -> Box<dyn Any> {
        use crate::one_many::op2_tuple;
        match self {
            AddNodeInput::F32(group) => op2_tuple(group, std::ops::Add::add).into_boxed_inner(),
            AddNodeInput::U32(group) => op2_tuple(group, std::ops::Add::add).into_boxed_inner(),
        }
    }

    fn types() -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static GROUPS: Lazy<Vec<crate::InputGroup>> = Lazy::new(|| {
            use crate::InputSupplemental;
            let float = F32F32::types(&["lhs", "rhs"]);
            let unsigned = U32U32::types(&["lhs", "rhs"]);
            let mut acc = Vec::new();
            acc.extend(float);
            acc.extend(unsigned);
            acc
        });

        PossibleInputs::new(&*GROUPS)
    }
}

impl FromAny for AddNodeInput {
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
        if let Ok(output) = F32F32::from_any(inputs) {
            Ok(AddNodeInput::F32(output))
        } else if let Ok(output) = U32U32::from_any(inputs) {
            Ok(AddNodeInput::U32(output))
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Copy, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AddNode;

impl NodeInput for AddNode {
    fn inputs(&self) -> PossibleInputs<'static> {
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
