use crate::{generic::GenericPair, FromAny, Node, NodeInput, NodeOutput, PossibleInputs};
use std::any::Any;

#[derive(Debug, Clone)]
enum DivisionNodeInput {
    F32(GenericPair<f32, f32>),
    U32(GenericPair<u32, u32>),
}

impl DivisionNodeInput {
    fn op(self) -> Box<dyn Any> {
        match self {
            DivisionNodeInput::F32(group) => group.op(std::ops::Div::div),
            DivisionNodeInput::U32(group) => group.op(std::ops::Div::div),
        }
    }

    fn types() -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static INPUTS: Lazy<PossibleInputs> = Lazy::new(|| {
            use crate::InputGroup;
            static GROUPS: Lazy<Vec<InputGroup>> = Lazy::new(|| {
                let float = GenericPair::<f32, f32>::gen_groups("numerator", "denominator");
                let unsigned = GenericPair::<u32, u32>::gen_groups("numerator", "denominator");
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

impl FromAny for DivisionNodeInput {
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()>
    where
        Self: Sized,
    {
        if let Ok(output) = GenericPair::<f32, f32>::from_any(inputs) {
            Ok(DivisionNodeInput::F32(output))
        } else if let Ok(output) = GenericPair::<u32, u32>::from_any(inputs) {
            Ok(DivisionNodeInput::U32(output))
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Copy, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DivisionNode;

impl NodeInput for DivisionNode {
    fn inputs(&self) -> PossibleInputs {
        DivisionNodeInput::types()
    }
}

impl NodeOutput for DivisionNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        DivisionNodeInput::from_any(inputs).map(DivisionNodeInput::op)
    }
}

#[typetag::serde]
impl Node for DivisionNode {
    fn name(&self) -> &'static str {
        "division"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inputs() {
        let inputs = GenericPair::<f32, f32>::gen_groups();
        assert_eq!(9, inputs.len());

        let inputs = DivisionNode.inputs();
        assert_eq!(18, inputs.groups.len());
    }
}
