use crate::{generic::GenericPair, FromAny, Node, NodeInput, NodeOutput, PossibleInputs};
use std::any::Any;

#[derive(Debug, Clone)]
enum MultiplyNodeInput {
    F32(GenericPair<f32, f32>),
    U32(GenericPair<u32, u32>),
}

impl MultiplyNodeInput {
    fn op(self) -> Box<dyn Any> {
        match self {
            MultiplyNodeInput::F32(group) => group.op(std::ops::Mul::mul),
            MultiplyNodeInput::U32(group) => group.op(std::ops::Mul::mul),
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

impl FromAny for MultiplyNodeInput {
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()>
    where
        Self: Sized,
    {
        if let Ok(output) = GenericPair::<f32, f32>::from_any(inputs) {
            Ok(MultiplyNodeInput::F32(output))
        } else if let Ok(output) = GenericPair::<u32, u32>::from_any(inputs) {
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
        let inputs = GenericPair::<f32, f32>::gen_groups("", "");
        assert_eq!(9, inputs.len());

        let inputs = MultiplyNode.inputs();
        assert_eq!(18, inputs.groups.len());
    }
}
