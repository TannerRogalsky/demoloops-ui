use nodes::{FromAnyProto, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use solstice_2d::RegularPolygon;
use std::any::Any;

type RegularPolygonInput = (
    OneOrMany<f32>,
    OneOrMany<f32>,
    OneOrMany<u32>,
    OneOrMany<f32>,
);

fn op((x, y, vertex_count, radius): RegularPolygonInput) -> Box<dyn Any> {
    nodes::one_many::op4(x, y, vertex_count, radius, RegularPolygon::new).into_boxed_inner()
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
        FromAnyProto::from_any(nodes::InputStack::new(inputs, ..)).map(op)
    }
}

#[typetag::serde]
impl Node for RegularPolygonNode {
    fn name(&self) -> &'static str {
        "regular polygon"
    }
}
