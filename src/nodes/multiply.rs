use nodes::{
    FromAnyProto, InputStack, OneOrMany, PossibleInputs,
};
use solstice_2d::Transform3D;
use std::any::Any;

#[derive(FromAnyProto, nodes::InputComponent)]
struct TransformMultiplyInput {
    lhs: OneOrMany<Transform3D>,
    rhs: OneOrMany<Transform3D>,
}

#[derive(FromAnyProto, nodes::InputComponent)]
enum MultiplyInput {
    Standard(nodes::ArithmeticNodeInput),
    Extended(TransformMultiplyInput)
}

impl MultiplyInput {
    fn op(self) -> Box<dyn Any> {
        match self {
            MultiplyInput::Standard(v) => v.mul(),
            MultiplyInput::Extended(v) => {
                nodes::one_many::op2(v.lhs, v.rhs, std::ops::Mul::mul).into_boxed_inner()
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ExtendedMultiplyNode;

impl nodes::NodeInput for ExtendedMultiplyNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static CACHE: Lazy<PossibleInputs> =
            Lazy::new(|| MultiplyInput::possible_inputs(&["lhs", "rhs"]));
        PossibleInputs::new(&*CACHE.groups)
    }
}

impl nodes::NodeOutput for ExtendedMultiplyNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        nodes::FromAnyProto::from_any(InputStack::new(inputs, ..)).map(MultiplyInput::op)
    }
}

#[typetag::serde]
impl nodes::Node for ExtendedMultiplyNode {
    fn name(&self) -> &'static str {
        "multiply"
    }
}
