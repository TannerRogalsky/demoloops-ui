mod constant;
mod division;
mod global;
mod modulo;
mod multiply;
mod range;
mod ratio;
mod sin_cos;
mod to_float;

pub use constant::ConstantNode;
pub use division::DivisionNode;
pub use global::GlobalNode;
pub use modulo::ModuloNode;
pub use multiply::MultiplyNode;
pub use range::{Range2DNode, RangeNode};
pub use ratio::RatioNode;
pub use sin_cos::{CosNode, SineNode};
pub use to_float::ToFloatNode;

pub mod generic {
    use crate::{FromAny, InputGroup, OneOrMany};
    use std::any::Any;

    #[derive(Debug, Clone)]
    pub struct GenericPair<LHS, RHS> {
        lhs: OneOrMany<LHS>,
        rhs: OneOrMany<RHS>,
    }

    impl<LHS, RHS> FromAny for GenericPair<LHS, RHS>
    where
        LHS: 'static,
        RHS: 'static,
    {
        fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()>
        where
            Self: Sized,
        {
            if inputs.len() < 2 {
                return Err(());
            }

            let valid = OneOrMany::<LHS>::is(&*inputs[0]) && OneOrMany::<RHS>::is(&*inputs[1]);

            if !valid {
                return Err(());
            }

            let mut inputs = inputs.drain(0..2);
            let lhs = OneOrMany::<LHS>::downcast(inputs.next().unwrap()).unwrap();
            let rhs = OneOrMany::<RHS>::downcast(inputs.next().unwrap()).unwrap();
            Ok(Self { lhs, rhs })
        }
    }

    impl<LHS, RHS> GenericPair<LHS, RHS>
    where
        LHS: 'static,
        RHS: 'static,
    {
        pub fn gen_groups(
            lhs_name: &'static str,
            rhs_name: &'static str,
        ) -> Vec<InputGroup<'static>> {
            use crate::InputInfo;
            use itertools::Itertools;

            let lhs = OneOrMany::<LHS>::type_ids();
            let rhs = OneOrMany::<RHS>::type_ids();
            let groups = std::array::IntoIter::new(lhs)
                .cartesian_product(std::array::IntoIter::new(rhs))
                .map(|(lhs, rhs)| InputGroup {
                    info: vec![
                        InputInfo {
                            name: lhs_name,
                            ty_name: std::any::type_name::<LHS>(),
                            type_id: lhs,
                        },
                        InputInfo {
                            name: rhs_name,
                            ty_name: std::any::type_name::<RHS>(),
                            type_id: rhs,
                        },
                    ]
                    .into(),
                })
                .collect::<Vec<_>>();
            groups
        }

        pub fn op<O, FUNC>(self, op: FUNC) -> Box<dyn Any>
        where
            LHS: Clone + std::fmt::Debug,
            RHS: Clone + std::fmt::Debug,
            O: Clone + std::fmt::Debug + 'static,
            FUNC: Fn(LHS, RHS) -> O + 'static + Clone,
        {
            match crate::one_many::op2(self.lhs, self.rhs, op) {
                OneOrMany::One(v) => Box::new(v),
                OneOrMany::Many(v) => Box::new(v),
            }
        }
    }
}
