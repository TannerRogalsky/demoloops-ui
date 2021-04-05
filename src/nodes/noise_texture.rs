use nodes::{FromAnyProto, InputComponent, InputStack, OneOrMany, PossibleInputs};
use solstice_2d::PerlinTextureSettings;
use std::any::Any;

#[derive(FromAnyProto, InputComponent)]
struct NoiseTextureInput {
    seed: Option<OneOrMany<u32>>,
    width: Option<OneOrMany<u32>>,
    height: Option<OneOrMany<u32>>,
}

fn op(v: NoiseTextureInput) -> Box<dyn Any> {
    use nodes::one_many::op3;
    let seed = v.seed.unwrap_or(OneOrMany::One(nodes::One::new(0)));
    let width = v.width.unwrap_or(OneOrMany::One(nodes::One::new(64)));
    let height = v.height.unwrap_or(OneOrMany::One(nodes::One::new(64)));
    op3(seed, width, height, |seed, width, height| {
        PerlinTextureSettings {
            seed: seed as _,
            width: width as _,
            height: height as _,
            period: width / 2,
            levels: 2,
            attenuation: std::convert::TryInto::try_into(0f32).unwrap(),
            color: true,
        }
    })
    .into_boxed_inner()
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct NoiseTextureNode;

impl nodes::NodeInput for NoiseTextureNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static CACHE: Lazy<PossibleInputs> =
            Lazy::new(|| NoiseTextureInput::possible_inputs(&["seed", "width", "height"]));
        PossibleInputs::new(&*CACHE.groups)
    }
}

impl nodes::NodeOutput for NoiseTextureNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        FromAnyProto::from_any(InputStack::new(inputs, ..)).map(op)
    }
}

#[typetag::serde]
impl nodes::Node for NoiseTextureNode {
    fn name(&self) -> &'static str {
        "noise"
    }
}
