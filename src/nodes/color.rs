use nodes::{InputGroup, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use solstice_2d::Color;
use std::any::Any;

type ColorInput = (
    OneOrMany<f32>,
    OneOrMany<f32>,
    OneOrMany<f32>,
    OneOrMany<f32>,
);

fn op((r, g, b, a): ColorInput) -> Box<dyn Any> {
    nodes::one_many::op4(r, g, b, a, Color::new).into_boxed_inner()
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ColorNode;

impl NodeInput for ColorNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        use nodes::InputSupplemental;
        use once_cell::sync::Lazy;
        static GROUPS: Lazy<Vec<InputGroup<'static>>> =
            Lazy::new(|| ColorInput::types(&["red", "green", "blue", "alpha"]));
        PossibleInputs { groups: &*GROUPS }
    }
}

impl NodeOutput for ColorNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        nodes::FromAny::from_any(inputs).map(op)
    }
}

#[typetag::serde]
impl Node for ColorNode {
    fn name(&self) -> &'static str {
        "color"
    }
}
