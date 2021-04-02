use crate::{One, PossibleInputs};
use std::any::Any;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ConstantNode {
    Unsigned(u32),
    Float(f32),
    Text(String),
}

impl crate::NodeInput for ConstantNode {
    fn is_terminator(&self) -> bool {
        true
    }
    fn inputs(&self) -> PossibleInputs<'static> {
        let groups: &'static [crate::InputGroup] = &[];
        PossibleInputs::new(groups)
    }
}

impl crate::NodeOutput for ConstantNode {
    fn op(&self, _inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        Ok(match self {
            ConstantNode::Unsigned(output) => Box::new(One(*output)),
            ConstantNode::Float(output) => Box::new(One(*output)),
            ConstantNode::Text(output) => Box::new(One(output.clone())),
        })
    }
}

#[typetag::serde]
impl crate::Node for ConstantNode {
    fn name(&self) -> &'static str {
        "constant"
    }
}
