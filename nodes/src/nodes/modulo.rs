use crate::{FromAny, InputGroup, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use std::any::Any;

struct ModuloGroup<X, Y> {
    numerator: OneOrMany<X>,
    denominator: OneOrMany<Y>,
}

impl<X, Y> FromAny for ModuloGroup<X, Y>
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
        impl ModuloGroup<$x, $y> {
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
                    Lazy::new(ModuloGroup::<$x, $y>::gen_groups);
                &*GROUPS
            }
        }
    };
}

impl<X, Y, Z> ModuloGroup<X, Y>
where
    X: std::ops::Rem<Y, Output = Z> + Clone + std::fmt::Debug + 'static,
    Y: Clone + std::fmt::Debug + 'static,
    Z: Clone + std::fmt::Debug + 'static,
{
    pub fn op(self) -> Box<dyn Any> {
        use crate::one_many::op2;
        let result = op2(self.numerator, self.denominator, std::ops::Rem::rem);
        match result {
            OneOrMany::One(v) => Box::new(v),
            OneOrMany::Many(v) => Box::new(v),
        }
    }
}

group_impl!(u32, u32);
group_impl!(f32, f32);

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ModuloNode;

impl NodeInput for ModuloNode {
    fn inputs(&self) -> PossibleInputs {
        use once_cell::sync::Lazy;
        static GROUPS: Lazy<Vec<InputGroup>> = Lazy::new(|| {
            let float = ModuloGroup::<f32, f32>::types();
            let unsigned = ModuloGroup::<u32, u32>::types();
            let mut groups = vec![];
            groups.extend_from_slice(&float);
            groups.extend_from_slice(&unsigned);
            groups
        });
        PossibleInputs { groups: &*GROUPS }
    }
}

impl NodeOutput for ModuloNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        if let Ok(output) = ModuloGroup::<f32, f32>::from_any(inputs) {
            Ok(output.op())
        } else if let Ok(output) = ModuloGroup::<u32, u32>::from_any(inputs) {
            Ok(output.op())
        } else {
            Err(())
        }
    }
}

#[typetag::serde]
impl Node for ModuloNode {
    fn name(&self) -> &'static str {
        "modulo"
    }
}
