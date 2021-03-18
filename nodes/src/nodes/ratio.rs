use crate::{FromAny, InputGroup, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use std::any::Any;

struct RatioGroup<X, Y> {
    numerator: OneOrMany<X>,
    denominator: OneOrMany<Y>,
}

impl<X, Y> FromAny for RatioGroup<X, Y>
where
    X: 'static,
    Y: 'static,
{
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
        if inputs.len() < 2 {
            return Err(());
        }

        let valid = OneOrMany::<X>::is(&*inputs[0]) && OneOrMany::<Y>::is(&*inputs[1]);

        if !valid {
            return Err(());
        }

        fn take<T: 'static>(v: Box<dyn Any>) -> OneOrMany<T> {
            OneOrMany::<T>::downcast(v).unwrap()
        }

        let mut inputs = inputs.drain(0..2);
        let numerator = take(inputs.next().unwrap());
        let denominator = take(inputs.next().unwrap());
        Ok(Self {
            numerator,
            denominator,
        })
    }
}

macro_rules! group_impl {
    ($x: ty, $y: ty) => {
        impl RatioGroup<$x, $y> {
            fn gen_groups() -> Vec<InputGroup<'static>> {
                use crate::InputInfo;
                use itertools::Itertools;

                let lhs = OneOrMany::<$x>::type_ids();
                let rhs = OneOrMany::<$y>::type_ids();
                let groups = std::array::IntoIter::new(lhs)
                    .cartesian_product(std::array::IntoIter::new(rhs))
                    .map(|(lhs, rhs)| InputGroup {
                        info: vec![
                            InputInfo {
                                name: "numerator",
                                ty_name: stringify!($x),
                                type_id: lhs,
                            },
                            InputInfo {
                                name: "denominator",
                                ty_name: stringify!($y),
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
                    Lazy::new(RatioGroup::<$x, $y>::gen_groups);
                &*GROUPS
            }
        }
    };
}

impl<X, Y> RatioGroup<X, Y>
where
    X: std::ops::Rem<Y, Output = Y> + Into<f64> + Clone + std::fmt::Debug + 'static,
    Y: Into<f64> + Clone + std::fmt::Debug + 'static,
{
    pub fn op(self) -> Box<dyn Any> {
        use crate::one_many::op2;
        let result = op2(self.numerator, self.denominator, |count, length| {
            if length.clone().into() == 0. {
                0.
            } else {
                let remainder = count.rem(length.clone());
                (remainder.into() / length.into()) as f32
            }
        });
        match result {
            OneOrMany::One(v) => Box::new(v),
            OneOrMany::Many(v) => Box::new(v),
        }
    }
}

group_impl!(u32, u32);
group_impl!(f32, f32);

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RatioNode;

impl NodeInput for RatioNode {
    fn inputs(&self) -> PossibleInputs {
        use once_cell::sync::Lazy;
        static GROUPS: Lazy<Vec<InputGroup>> = Lazy::new(|| {
            let float = RatioGroup::<f32, f32>::types();
            let unsigned = RatioGroup::<u32, u32>::types();
            let mut groups = vec![];
            groups.extend_from_slice(&float);
            groups.extend_from_slice(&unsigned);
            groups
        });
        PossibleInputs { groups: &*GROUPS }
    }
}

impl NodeOutput for RatioNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        if let Ok(output) = RatioGroup::<f32, f32>::from_any(inputs) {
            Ok(output.op())
        } else if let Ok(output) = RatioGroup::<u32, u32>::from_any(inputs) {
            Ok(output.op())
        } else {
            Err(())
        }
    }
}

#[typetag::serde]
impl Node for RatioNode {
    fn name(&self) -> &'static str {
        "ratio"
    }
}
