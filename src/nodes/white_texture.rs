use nodes::{One, PossibleInputs};
use std::any::Any;

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct WhiteTextureNode;

impl nodes::NodeInput for WhiteTextureNode {
    fn is_terminator(&self) -> bool {
        true
    }
    fn inputs(&self) -> PossibleInputs<'static> {
        let groups: &'static [nodes::InputGroup] = &[];
        PossibleInputs::new(groups)
    }
}

impl nodes::NodeOutput for WhiteTextureNode {
    fn op(&self, _inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        Ok(Box::new(One::new(crate::command::Texture::Default)))
    }
}

#[typetag::serde]
impl nodes::Node for WhiteTextureNode {
    fn name(&self) -> &'static str {
        "white texture"
    }
}