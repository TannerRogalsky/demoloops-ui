use crate::One;
use std::any::Any;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ConstantNode {
    Unsigned(u32),
    Float(f32),
}

impl crate::NodeInput for ConstantNode {
    fn inputs_match(&self, _inputs: &[Box<dyn Any>]) -> bool {
        false
    }

    fn is_terminator(&self) -> bool {
        true
    }
}

impl crate::NodeOutput for ConstantNode {
    fn op(&self, _inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        Ok(match self {
            ConstantNode::Unsigned(output) => Box::new(One(*output)),
            ConstantNode::Float(output) => Box::new(One(*output)),
        })
    }
}

#[typetag::serde]
impl crate::Node for ConstantNode {
    fn name(&self) -> &'static str {
        match self {
            ConstantNode::Unsigned(_) => "unsigned constant",
            ConstantNode::Float(_) => "float constant",
        }
    }
}
