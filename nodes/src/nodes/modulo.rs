use crate::generic::GenericPair;
use crate::{FromAny, Node, NodeInput, NodeOutput, PossibleInputs};
use std::any::Any;

enum ModuloGroup {
    F32(GenericPair<f32, f32>),
    U32(GenericPair<u32, u32>),
}

impl ModuloGroup {
    fn op(self) -> Box<dyn Any> {
        match self {
            ModuloGroup::F32(v) => v.op(std::ops::Rem::rem),
            ModuloGroup::U32(v) => v.op(std::ops::Rem::rem),
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ModuloNode;

impl NodeInput for ModuloNode {
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

impl NodeOutput for ModuloNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        if let Ok(output) = GenericPair::<f32, f32>::from_any(inputs) {
            Ok(ModuloGroup::F32(output).op())
        } else if let Ok(output) = GenericPair::<u32, u32>::from_any(inputs) {
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
