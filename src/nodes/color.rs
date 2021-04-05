use nodes::{FromAnyProto, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
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
        use once_cell::sync::Lazy;
        static CACHE: Lazy<PossibleInputs> =
            Lazy::new(|| ColorInput::possible_inputs(&["red", "green", "blue", "alpha"]));
        PossibleInputs::new(&*CACHE.groups)
    }
}

impl NodeOutput for ColorNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        FromAnyProto::from_any(nodes::InputStack::new(inputs, ..)).map(op)
    }
}

#[typetag::serde]
impl Node for ColorNode {
    fn name(&self) -> &'static str {
        "color"
    }
}
