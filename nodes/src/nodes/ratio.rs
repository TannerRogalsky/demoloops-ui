use crate::{FromAny, Many, Node, NodeInput, NodeOutput, One, Pair};
use std::any::Any;

#[derive(Debug, Clone)]
enum RatioGroup<T>
where
    T: 'static,
{
    OneOne(Pair<One<T>, One<T>>),
    OneMany(Pair<One<T>, Many<T>>),
}

impl<T> RatioGroup<T>
where
    T: std::ops::Rem<Output = T> + Into<f64> + Copy + 'static,
{
    fn can_match(inputs: &[Box<dyn Any>]) -> bool {
        Pair::<One<T>, One<T>>::can_match(inputs) || Pair::<One<T>, Many<T>>::can_match(inputs)
    }
}

impl<T> RatioGroup<T> {
    fn from_any(inputs: &mut Vec<Box<dyn std::any::Any>>) -> Result<Self, ()> {
        if let Ok(one_one) = Pair::<One<T>, One<T>>::from_any(inputs) {
            Ok(RatioGroup::OneOne(one_one))
        } else if let Ok(one_many) = Pair::<One<T>, Many<T>>::from_any(inputs) {
            Ok(RatioGroup::OneMany(one_many))
        } else {
            Err(())
        }
    }
}

impl<T> RatioGroup<T>
where
    T: std::ops::Rem<Output = T> + Into<f64> + Copy + 'static,
{
    fn op(self) -> Box<dyn Any> {
        fn ratio<T>(length: T, count: T) -> f32
        where
            T: std::ops::Rem<Output = T> + Into<f64> + Copy,
        {
            (count.rem(length).into() / length.into()) as f32
        }

        match self {
            Self::OneOne(Pair { lhs, rhs }) => Box::new(One(ratio(lhs.inner(), rhs.inner()))),
            Self::OneMany(Pair { lhs, rhs }) => {
                let out = rhs.inner().map(move |count| ratio(lhs.inner(), count));
                Box::new(Many::from(out))
            }
        }
    }
}

enum RatioNodeInput {
    F32(RatioGroup<f32>),
    U32(RatioGroup<u32>),
}

impl RatioNodeInput {
    fn op(self) -> Box<dyn Any> {
        match self {
            Self::F32(group) => group.op(),
            Self::U32(group) => group.op(),
        }
    }

    fn can_match(inputs: &[Box<dyn Any>]) -> bool {
        RatioGroup::<f32>::can_match(inputs) || RatioGroup::<u32>::can_match(inputs)
    }
}

impl FromAny for RatioNodeInput {
    fn from_any(inputs: &mut Vec<Box<dyn std::any::Any>>) -> Result<Self, ()> {
        if let Ok(output) = RatioGroup::<f32>::from_any(inputs) {
            Ok(RatioNodeInput::F32(output))
        } else if let Ok(output) = RatioGroup::<u32>::from_any(inputs) {
            Ok(RatioNodeInput::U32(output))
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RatioNode;

impl NodeInput for RatioNode {
    fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool {
        RatioNodeInput::can_match(inputs)
    }
}

impl NodeOutput for RatioNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        RatioNodeInput::from_any(inputs).map(RatioNodeInput::op)
    }
}

#[typetag::serde]
impl Node for RatioNode {
    fn name(&self) -> &'static str {
        "ratio"
    }
}
