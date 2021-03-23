use nodes::{Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use solstice_2d::PerlinTextureSettings;
use std::any::Any;

type NoiseTextureInput = (OneOrMany<u32>, OneOrMany<u32>);

fn op((width, height): NoiseTextureInput) -> Box<dyn Any> {
    nodes::one_many::op2(width, height, |width, height| PerlinTextureSettings {
        seed: 0,
        width: width as _,
        height: height as _,
        period: 0,
        levels: 0,
        attenuation: std::convert::TryInto::try_into(0f32).unwrap(),
        color: false,
    }).into_boxed_inner()
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct NoiseTextureNode;

impl NodeInput for NoiseTextureNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        use nodes::InputSupplemental;
        use once_cell::sync::Lazy;
        static GROUPS: Lazy<Vec<nodes::InputGroup<'static>>> =
            Lazy::new(|| NoiseTextureInput::types(&["width", "height"]));
        PossibleInputs { groups: &*GROUPS }
    }
}

impl NodeOutput for NoiseTextureNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        nodes::FromAny::from_any(inputs).map(op)
    }
}

#[typetag::serde]
impl Node for NoiseTextureNode {
    fn name(&self) -> &'static str {
        "noise"
    }
}
