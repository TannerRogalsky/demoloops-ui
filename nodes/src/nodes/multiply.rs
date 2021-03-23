use crate::{FromAny, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use std::any::Any;

type F32F32 = (OneOrMany<f32>, OneOrMany<f32>);
type U32U32 = (OneOrMany<u32>, OneOrMany<u32>);

#[derive(Debug, Clone)]
enum MultiplyNodeInput {
    F32(F32F32),
    U32(U32U32),
}

impl MultiplyNodeInput {
    fn op(self) -> Box<dyn Any> {
        use crate::one_many::op2_tuple;
        match self {
            MultiplyNodeInput::F32(v) => op2_tuple(v, std::ops::Mul::mul).into_boxed_inner(),
            MultiplyNodeInput::U32(v) => op2_tuple(v, std::ops::Mul::mul).into_boxed_inner(),
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

        PossibleInputs { groups: &*GROUPS }
    }
}

impl FromAny for MultiplyNodeInput {
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
        if let Ok(output) = F32F32::from_any(inputs) {
            Ok(MultiplyNodeInput::F32(output))
        } else if let Ok(output) = U32U32::from_any(inputs) {
            Ok(MultiplyNodeInput::U32(output))
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Copy, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct MultiplyNode;

impl NodeInput for MultiplyNode {
    fn inputs(&self) -> PossibleInputs {
        MultiplyNodeInput::types()
    }
}

impl NodeOutput for MultiplyNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        MultiplyNodeInput::from_any(inputs).map(MultiplyNodeInput::op)
    }
}

#[typetag::serde]
impl Node for MultiplyNode {
    fn name(&self) -> &'static str {
        "multiply"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inputs() {
        let inputs = MultiplyNode.inputs();
        assert_eq!(18, inputs.groups.len());
    }
}
