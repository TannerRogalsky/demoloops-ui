use crate::{generic::GenericPair, FromAny, Node, NodeInput, NodeOutput, PossibleInputs};
use std::any::Any;

enum RatioGroup {
    F32(GenericPair<f32, f32>),
    U32(GenericPair<u32, u32>),
}

impl RatioGroup {
    fn op(self) -> Box<dyn Any> {
        match self {
            RatioGroup::F32(v) => v.op(|count, length| {
                let remainder = count % length;
                remainder / length
            }),
            RatioGroup::U32(v) => v.op(|count, length| {
                let remainder = count % length;
                (remainder as f64 / length as f64) as f32
            }),
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RatioNode;

impl NodeInput for RatioNode {
    fn inputs(&self) -> PossibleInputs {
        use once_cell::sync::Lazy;
        static GROUPS: Lazy<Vec<crate::InputGroup>> = Lazy::new(|| {
            let float = GenericPair::<f32, f32>::gen_groups("numerator", "denominator");
            let unsigned = GenericPair::<u32, u32>::gen_groups("numerator", "denominator");
            let mut groups = vec![];
            groups.extend_from_slice(&float);
            groups.extend_from_slice(&unsigned);
            groups
        });
        PossibleInputs { groups: &*GROUPS }
    }
}

impl NodeOutput for RatioNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        if let Ok(output) = GenericPair::<f32, f32>::from_any(inputs) {
            Ok(RatioGroup::F32(output).op())
        } else if let Ok(output) = GenericPair::<u32, u32>::from_any(inputs) {
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
