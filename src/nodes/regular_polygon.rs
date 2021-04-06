use nodes::{FromAnyProto, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use solstice_2d::RegularPolygon;
use std::any::Any;

#[derive(FromAnyProto, nodes::InputComponent)]
struct RegularPolygonInput {
    x: Option<OneOrMany<f32>>,
    y: Option<OneOrMany<f32>>,
    vertex_count: OneOrMany<u32>,
    radius: OneOrMany<f32>,
}

impl RegularPolygonInput {
    fn op(self) -> Box<dyn Any> {
        let Self { x, y, vertex_count, radius } = self;
        let x = x.unwrap_or(OneOrMany::One(nodes::One::new(0.)));
        let y = y.unwrap_or(OneOrMany::One(nodes::One::new(0.)));
        nodes::one_many::op4(x, y, vertex_count, radius, RegularPolygon::new).into_boxed_inner()
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RegularPolygonNode;

impl NodeInput for RegularPolygonNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static CACHE: Lazy<PossibleInputs> = Lazy::new(|| {
            RegularPolygonInput::possible_inputs(&["x", "y", "vertex_count", "radius"])
        });
        PossibleInputs::new(&*CACHE.groups)
    }
}

impl NodeOutput for RegularPolygonNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        FromAnyProto::from_any(nodes::InputStack::new(inputs, ..)).map(RegularPolygonInput::op)
    }
}

#[typetag::serde]
impl Node for RegularPolygonNode {
    fn name(&self) -> &'static str {
        "regular polygon"
    }
}
