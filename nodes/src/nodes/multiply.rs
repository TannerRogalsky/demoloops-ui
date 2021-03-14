use crate::{
    FromAny, InputGroup, InputMatchError, Many, Node, NodeInput, NodeOutput, One, Pair,
    PossibleInputs,
};
use std::any::Any;

#[derive(Debug, Clone)]
enum MultiplyGroup<T> {
    OneOne(Pair<One<T>, One<T>>),
    OneMany(Pair<One<T>, Many<T>>),
    ManyMany(Pair<Many<T>, Many<T>>),
}

macro_rules! group_impl {
    ($t: ty) => {
        impl MultiplyGroup<$t> {
            fn types() -> [InputGroup<'static>; 3] {
                [
                    Pair::<One<$t>, One<$t>>::types(),
                    Pair::<One<$t>, Many<$t>>::types(),
                    Pair::<Many<$t>, Many<$t>>::types(),
                ]
            }
        }
    };
}
group_impl!(f32);
group_impl!(u32);

impl<T> MultiplyGroup<T>
where
    T: std::ops::Mul<Output = T> + Copy + 'static,
{
    fn op(self) -> Box<dyn Any> {
        match self {
            MultiplyGroup::OneOne(Pair { lhs, rhs }) => Box::new(lhs * rhs),
            MultiplyGroup::OneMany(Pair { lhs, rhs }) => Box::new(lhs * rhs),
            MultiplyGroup::ManyMany(Pair { lhs, rhs }) => Box::new(lhs * rhs),
        }
    }
}

impl<T> MultiplyGroup<T>
where
    T: 'static,
{
    fn can_match(inputs: &[Box<dyn Any>]) -> Option<InputMatchError> {
        Pair::<One<T>, One<T>>::can_match(inputs)
            .or_else(|| Pair::<One<T>, Many<T>>::can_match(inputs))
            .or_else(|| Pair::<Many<T>, Many<T>>::can_match(inputs))
    }
}

impl<T> FromAny for MultiplyGroup<T>
where
    T: 'static,
{
    fn from_any(inputs: &mut Vec<Box<dyn std::any::Any>>) -> Result<Self, ()> {
        if let Ok(one_one) = Pair::<One<T>, One<T>>::from_any(inputs) {
            Ok(MultiplyGroup::OneOne(one_one))
        } else if let Ok(one_many) = Pair::<One<T>, Many<T>>::from_any(inputs) {
            Ok(MultiplyGroup::OneMany(one_many))
        } else if let Ok(many_many) = Pair::<Many<T>, Many<T>>::from_any(inputs) {
            Ok(MultiplyGroup::ManyMany(many_many))
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Clone)]
enum MultiplyNodeInput {
    F32(MultiplyGroup<f32>),
    U32(MultiplyGroup<u32>),
}

impl MultiplyNodeInput {
    fn op(self) -> Box<dyn Any> {
        match self {
            MultiplyNodeInput::F32(group) => group.op(),
            MultiplyNodeInput::U32(group) => group.op(),
        }
    }

    fn can_match(inputs: &[Box<dyn Any>]) -> Option<InputMatchError> {
        MultiplyGroup::<f32>::can_match(inputs).or_else(|| MultiplyGroup::<u32>::can_match(inputs))
    }

    fn types() -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static INPUTS: Lazy<PossibleInputs> = Lazy::new(|| {
            static GROUPS: Lazy<Vec<InputGroup>> = Lazy::new(|| {
                let float = MultiplyGroup::<f32>::types();
                let unsigned = MultiplyGroup::<u32>::types();
                vec![
                    float[0],
                    float[1],
                    float[2],
                    unsigned[0],
                    unsigned[1],
                    unsigned[2],
                ]
            });

            PossibleInputs { groups: &*GROUPS }
        });
        *INPUTS
    }
}

impl FromAny for MultiplyNodeInput {
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()>
    where
        Self: Sized,
    {
        if let Ok(output) = MultiplyGroup::<f32>::from_any(inputs) {
            Ok(MultiplyNodeInput::F32(output))
        } else if let Ok(output) = MultiplyGroup::<u32>::from_any(inputs) {
            Ok(MultiplyNodeInput::U32(output))
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Copy, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct MultiplyNode;

impl NodeInput for MultiplyNode {
    fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> Option<InputMatchError> {
        MultiplyNodeInput::can_match(inputs)
    }

    fn inputs(&self) -> PossibleInputs {
        MultiplyNodeInput::types()
    }
}

impl NodeOutput for MultiplyNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        MultiplyNodeInput::from_any(inputs).map(MultiplyNodeInput::op)
    }
}

#[typetag::serde]
impl Node for MultiplyNode {
    fn name(&self) -> &'static str {
        "multiply"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inputs() {
        let inputs = MultiplyNode.inputs();
        assert_eq!(6, inputs.groups.len());
    }
}
