use nodes::{
    FromAnyProto, InputStack, Node, NodeInput, NodeOutput, One, OneOrMany, PossibleInputs,
};
use solstice_2d::{Rad, Transform3D};
use std::any::Any;

#[derive(FromAnyProto, nodes::InputComponent)]
struct TranslationInput {
    x: Option<OneOrMany<f32>>,
    y: Option<OneOrMany<f32>>,
    z: Option<OneOrMany<f32>>,
}

impl TranslationInput {
    fn op(self) -> Box<dyn Any> {
        let TranslationInput { x, y, z } = self;
        let x = x.unwrap_or(OneOrMany::One(One::new(0.)));
        let y = y.unwrap_or(OneOrMany::One(One::new(0.)));
        let z = z.unwrap_or(OneOrMany::One(One::new(0.)));

        nodes::one_many::op3(x, y, z, Transform3D::translation).into_boxed_inner()
    }
}

#[derive(FromAnyProto, nodes::InputComponent)]
struct EulerRotationInput {
    roll: Option<OneOrMany<f32>>,
    pitch: Option<OneOrMany<f32>>,
    yaw: Option<OneOrMany<f32>>,
}

impl EulerRotationInput {
    fn op(self) -> Box<dyn Any> {
        let EulerRotationInput { roll, pitch, yaw } = self;
        let roll = roll.unwrap_or(OneOrMany::One(One::new(0.)));
        let pitch = pitch.unwrap_or(OneOrMany::One(One::new(0.)));
        let yaw = yaw.unwrap_or(OneOrMany::One(One::new(0.)));

        let op = |x, y, z| Transform3D::rotation(Rad(x), Rad(y), Rad(z));
        nodes::one_many::op3(roll, pitch, yaw, op).into_boxed_inner()
    }
}

#[derive(FromAnyProto, nodes::InputComponent)]
struct ScalingInput {
    x: Option<OneOrMany<f32>>,
    y: Option<OneOrMany<f32>>,
    z: Option<OneOrMany<f32>>,
}

impl ScalingInput {
    fn op(self) -> Box<dyn Any> {
        let ScalingInput { x, y, z } = self;
        let x = x.unwrap_or(OneOrMany::One(One::new(1.)));
        let y = y.unwrap_or(OneOrMany::One(One::new(1.)));
        let z = z.unwrap_or(OneOrMany::One(One::new(1.)));

        nodes::one_many::op3(x, y, z, Transform3D::scale).into_boxed_inner()
    }
}

#[derive(Default, Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranslationNode;

impl NodeInput for TranslationNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static CACHE: Lazy<PossibleInputs> =
            Lazy::new(|| TranslationInput::possible_inputs(&["x", "y", "z"]));
        PossibleInputs::new(&*CACHE.groups)
    }
}

impl NodeOutput for TranslationNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        FromAnyProto::from_any(InputStack::new(inputs, ..)).map(TranslationInput::op)
    }
}

#[typetag::serde]
impl Node for TranslationNode {
    fn name(&self) -> &'static str {
        "translation"
    }
}

#[derive(Default, Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct RotationNode;

impl NodeInput for RotationNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static CACHE: Lazy<PossibleInputs> =
            Lazy::new(|| EulerRotationInput::possible_inputs(&["roll", "pitch", "yaw"]));
        PossibleInputs::new(&*CACHE.groups)
    }
}

impl NodeOutput for RotationNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        FromAnyProto::from_any(InputStack::new(inputs, ..)).map(EulerRotationInput::op)
    }
}

#[typetag::serde]
impl Node for RotationNode {
    fn name(&self) -> &'static str {
        "rotation"
    }
}

#[derive(Default, Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScalingNode;

impl NodeInput for ScalingNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static CACHE: Lazy<PossibleInputs> =
            Lazy::new(|| ScalingInput::possible_inputs(&["x", "y", "z"]));
        PossibleInputs::new(&*CACHE.groups)
    }
}

impl NodeOutput for ScalingNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        FromAnyProto::from_any(InputStack::new(inputs, ..)).map(ScalingInput::op)
    }
}

#[typetag::serde]
impl Node for ScalingNode {
    fn name(&self) -> &'static str {
        "scaling"
    }
}
