use crate::{FromAny, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use std::any::Any;

type F32F32 = (OneOrMany<f32>, OneOrMany<f32>);
type U32U32 = (OneOrMany<u32>, OneOrMany<u32>);

enum RatioGroup {
    F32(F32F32),
    U32(U32U32),
}

impl RatioGroup {
    fn op(self) -> Box<dyn Any> {
        use crate::one_many::op2_tuple;
        match self {
            RatioGroup::F32(v) => op2_tuple(v, |count, length| {
                let remainder = count % length;
                remainder / length
            })
            .into_boxed_inner(),
            RatioGroup::U32(v) => op2_tuple(v, |count, length| {
                if length == 0 {
                    0.
                } else {
                    let remainder = count % length;
                    (remainder as f64 / length as f64) as f32
                }
            })
            .into_boxed_inner(),
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RatioNode;

impl NodeInput for RatioNode {
    fn inputs(&self) -> PossibleInputs {
        use once_cell::sync::Lazy;
        static GROUPS: Lazy<Vec<crate::InputGroup>> = Lazy::new(|| {
            use crate::InputSupplemental;
            let float = F32F32::types(&["numerator", "denominator"]);
            let unsigned = U32U32::types(&["numerator", "denominator"]);
            let mut acc = vec![];
            acc.extend(float);
            acc.extend(unsigned);
            acc
        });
        PossibleInputs { groups: &*GROUPS }
    }
}

impl NodeOutput for RatioNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        if let Ok(output) = F32F32::from_any(inputs) {
            Ok(RatioGroup::F32(output).op())
        } else if let Ok(output) = U32U32::from_any(inputs) {
            Ok(RatioGroup::U32(output).op())
        } else {
            Err(())
        }
    }
}

#[typetag::serde]
impl Node for RatioNode {
    fn name(&self) -> &'static str {
        "ratio"
    }
}
