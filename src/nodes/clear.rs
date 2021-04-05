use nodes::{FromAnyProto, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use solstice_2d::Color;
use std::any::Any;

type ClearInput = (OneOrMany<Color>,);

fn op((color,): ClearInput) -> Box<dyn Any> {
    use ::nodes::one_many::op1 as op;
    op(color, crate::command::ClearCommand::new).into_boxed_inner()
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ClearNode;

impl NodeInput for ClearNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static CACHE: Lazy<PossibleInputs> = Lazy::new(|| ClearInput::possible_inputs(&["color"]));
        PossibleInputs::new(&*CACHE.groups)
    }
}

impl NodeOutput for ClearNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        FromAnyProto::from_any(nodes::InputStack::new(inputs, ..)).map(op)
    }
}

#[typetag::serde]
impl Node for ClearNode {
    fn name(&self) -> &'static str {
        "clear"
    }
}
