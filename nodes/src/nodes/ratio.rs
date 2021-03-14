use crate::{FromAny, InputGroup, Many, Node, NodeInput, NodeOutput, One, Pair, PossibleInputs};
use std::any::Any;

#[derive(Debug, Clone)]
enum RatioGroup<T>
where
    T: 'static,
{
    OneOne(Pair<One<T>, One<T>>),
    OneMany(Pair<One<T>, Many<T>>),
}

macro_rules! group_impl {
    ($t: ty) => {
        impl RatioGroup<$t> {
            fn types() -> [InputGroup<'static>; 2] {
                [
                    Pair::<One<$t>, One<$t>>::types(),
                    Pair::<One<$t>, Many<$t>>::types(),
                ]
            }
        }
    };
}
group_impl!(f32);
group_impl!(u32);

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

    fn types() -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static GROUPS: Lazy<Vec<InputGroup>> = Lazy::new(|| {
            let float = RatioGroup::<f32>::types();
            let unsigned = RatioGroup::<u32>::types();
            vec![float[0], float[1], unsigned[0], unsigned[1]]
        });
        PossibleInputs { groups: &*GROUPS }
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
    fn inputs(&self) -> PossibleInputs {
        RatioNodeInput::types()
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
