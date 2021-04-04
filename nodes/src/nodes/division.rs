use crate::{FromAny, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use std::any::Any;

type F32F32 = (OneOrMany<f32>, OneOrMany<f32>);
type U32U32 = (OneOrMany<u32>, OneOrMany<u32>);

#[derive(Debug, Clone)]
enum DivisionNodeInput {
    F32(F32F32),
    U32(U32U32),
}

impl DivisionNodeInput {
    fn op(self) -> Box<dyn Any> {
        use crate::one_many::op2_tuple;
        match self {
            DivisionNodeInput::F32(group) => {
                op2_tuple(group, std::ops::Div::div).into_boxed_inner()
            }
            DivisionNodeInput::U32(group) => {
                op2_tuple(group, |lhs, rhs| lhs.checked_div(rhs).unwrap_or(0)).into_boxed_inner()
            }
        }
    }

    fn types() -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static GROUPS: Lazy<Vec<crate::InputGroup>> = Lazy::new(|| {
            use crate::InputSupplemental;
            let float = F32F32::types(&["numerator", "denominator"]);
            let unsigned = U32U32::types(&["numerator", "denominator"]);
            let mut acc = Vec::new();
            acc.extend(float);
            acc.extend(unsigned);
            acc
        });
        PossibleInputs::new(&*GROUPS)
    }
}

impl FromAny for DivisionNodeInput {
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
        if let Ok(output) = F32F32::from_any(inputs) {
            Ok(DivisionNodeInput::F32(output))
        } else if let Ok(output) = U32U32::from_any(inputs) {
            Ok(DivisionNodeInput::U32(output))
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Copy, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DivisionNode;

impl NodeInput for DivisionNode {
    fn inputs(&self) -> PossibleInputs<'static> {
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
