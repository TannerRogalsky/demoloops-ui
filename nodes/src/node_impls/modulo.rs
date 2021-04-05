use crate::{FromAny, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use std::any::Any;

type F32F32 = (OneOrMany<f32>, OneOrMany<f32>);
type U32U32 = (OneOrMany<u32>, OneOrMany<u32>);

enum ModuloGroup {
    F32(F32F32),
    U32(U32U32),
}

impl ModuloGroup {
    fn op(self) -> Box<dyn Any> {
        use crate::one_many::op2_tuple;
        match self {
            ModuloGroup::F32(v) => op2_tuple(v, std::ops::Rem::rem).into_boxed_inner(),
            ModuloGroup::U32(v) => {
                op2_tuple(v, |lhs, rhs| lhs.checked_rem(rhs).unwrap_or(0)).into_boxed_inner()
            }
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ModuloNode;

impl NodeInput for ModuloNode {
    fn inputs(&self) -> PossibleInputs<'static> {
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

impl NodeOutput for ModuloNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        if let Ok(output) = F32F32::from_any(inputs) {
            Ok(ModuloGroup::F32(output).op())
        } else if let Ok(output) = U32U32::from_any(inputs) {
            Ok(ModuloGroup::U32(output).op())
        } else {
            Err(())
        }
    }
}

#[typetag::serde]
impl Node for ModuloNode {
    fn name(&self) -> &'static str {
        "modulo"
    }
}
