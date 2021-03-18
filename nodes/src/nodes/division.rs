use crate::{FromAny, InputGroup, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use std::any::Any;

#[derive(Debug, Clone)]
struct DivisionGroup<T> {
    lhs: OneOrMany<T>,
    rhs: OneOrMany<T>,
}

impl<T: 'static> FromAny for DivisionGroup<T> {
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()>
    where
        Self: Sized,
    {
        if inputs.len() < 2 {
            return Err(());
        }

        let valid = inputs[0..2]
            .iter()
            .all(|input| OneOrMany::<T>::is(&**input));

        if !valid {
            return Err(());
        }

        let mut inputs = inputs
            .drain(0..2)
            .map(OneOrMany::<T>::downcast)
            .map(Result::unwrap);
        Ok(Self {
            lhs: inputs.next().unwrap(),
            rhs: inputs.next().unwrap(),
        })
    }
}

impl<T> DivisionGroup<T>
where
    T: std::ops::Div<Output = T> + Clone + std::fmt::Debug + 'static,
{
    pub fn op(self) -> Box<dyn Any> {
        match crate::one_many::op2(self.lhs, self.rhs, std::ops::Div::div) {
            OneOrMany::One(v) => Box::new(v),
            OneOrMany::Many(v) => Box::new(v),
        }
    }
}

macro_rules! group_impl {
    ($t: ty) => {
        impl DivisionGroup<$t> {
            fn gen_groups() -> Vec<InputGroup<'static>> {
                use crate::InputInfo;
                use itertools::Itertools;

                let lhs = OneOrMany::<$t>::type_ids();
                let rhs = OneOrMany::<$t>::type_ids();
                let groups = std::array::IntoIter::new(lhs)
                    .cartesian_product(std::array::IntoIter::new(rhs))
                    .map(|(lhs, rhs)| InputGroup {
                        info: vec![
                            InputInfo {
                                name: "lhs",
                                ty_name: stringify!($t),
                                type_id: lhs,
                            },
                            InputInfo {
                                name: "rhs",
                                ty_name: stringify!($t),
                                type_id: rhs,
                            },
                        ]
                        .into(),
                    })
                    .collect::<Vec<_>>();
                groups
            }

            fn types() -> &'static [InputGroup<'static>] {
                use once_cell::sync::Lazy;

                static GROUPS: Lazy<Vec<InputGroup<'static>>> =
                    Lazy::new(DivisionGroup::<$t>::gen_groups);
                &*GROUPS
            }
        }
    };
}

group_impl!(u32);
group_impl!(f32);

#[derive(Debug, Clone)]
enum DivisionNodeInput {
    F32(DivisionGroup<f32>),
    U32(DivisionGroup<u32>),
}

impl DivisionNodeInput {
    fn op(self) -> Box<dyn Any> {
        match self {
            DivisionNodeInput::F32(group) => group.op(),
            DivisionNodeInput::U32(group) => group.op(),
        }
    }

    fn types() -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static INPUTS: Lazy<PossibleInputs> = Lazy::new(|| {
            static GROUPS: Lazy<Vec<InputGroup>> = Lazy::new(|| {
                let float = DivisionGroup::<f32>::types();
                let unsigned = DivisionGroup::<u32>::types();
                let mut acc = Vec::new();
                acc.extend_from_slice(&float);
                acc.extend_from_slice(&unsigned);
                acc
            });

            PossibleInputs { groups: &*GROUPS }
        });
        *INPUTS
    }
}

impl FromAny for DivisionNodeInput {
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()>
    where
        Self: Sized,
    {
        if let Ok(output) = DivisionGroup::<f32>::from_any(inputs) {
            Ok(DivisionNodeInput::F32(output))
        } else if let Ok(output) = DivisionGroup::<u32>::from_any(inputs) {
            Ok(DivisionNodeInput::U32(output))
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Copy, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DivisionNode;

impl NodeInput for DivisionNode {
    fn inputs(&self) -> PossibleInputs {
        DivisionNodeInput::types()
    }
}

impl NodeOutput for DivisionNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        DivisionNodeInput::from_any(inputs).map(DivisionNodeInput::op)
    }
}

#[typetag::serde]
impl Node for DivisionNode {
    fn name(&self) -> &'static str {
        "division"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inputs() {
        let inputs = DivisionGroup::<f32>::types();
        assert_eq!(9, inputs.len());

        let inputs = DivisionNode.inputs();
        assert_eq!(18, inputs.groups.len());
    }
}
