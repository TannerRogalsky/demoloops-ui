mod add;
mod arithmetic;
mod constant;
mod division;
mod global;
mod modulo;
mod multiply;
mod range;
mod ratio;
mod repeat;
mod sin_cos;
mod to_float;

pub use add::AddNode;
pub use arithmetic::ArithmeticNodeInput;
pub use constant::ConstantNode;
pub use division::DivisionNode;
pub use global::GlobalNode;
pub use modulo::ModuloNode;
pub use multiply::MultiplyNode;
pub use range::RangeNode;
pub use ratio::RatioNode;
pub use repeat::RepeatNode;
pub use sin_cos::{CosNode, SineNode};
pub use to_float::ToFloatNode;

pub mod generic {
    use crate::{InputComponent, InputGroup};
    use std::any::{Any, TypeId};

    macro_rules! count_idents {
        ($($idents:ident),* $(,)*) => {
            {
                #[allow(dead_code, non_camel_case_types)]
                enum Idents { $($idents,)* __CountIdentsLast }
                const COUNT: usize = Idents::__CountIdentsLast as usize;
                COUNT
            }
        };
    }

    macro_rules! tuple_impl {
        ($( $name:ident )+) => {
            impl<$($name: 'static),+> InputComponent for ($($name,)+) {
                fn is(v: &dyn Any) -> bool {
                    v.is::<($($name,)+)>()
                }

                fn type_ids() -> Vec<TypeId> {
                    vec![TypeId::of::<($($name,)+)>()]
                }

                fn downcast(v: Box<dyn Any>) -> Result<Self, Box<dyn Any>> {
                    v.downcast::<($($name,)+)>().map(|v| *v)

                }
            }

            impl<$($name: InputComponent),+> crate::FromAnyProto for ($($name,)+) {
                fn from_any(inputs: crate::InputStack<'_, Box<dyn std::any::Any>>) -> Result<Self, ()> {
                    let len = count_idents!($($name,)+);
                    if inputs.as_slice().len() != len {
                        return Err(())
                    }

                    let mut checker = inputs.deref_iter();
                    $(
                        if !<$name>::is(checker.next().unwrap()) {
                            return Err(())
                        }
                    )+

                    let mut inputs = inputs.consume();
                    Ok(($(
                        <$name>::downcast(inputs.next().unwrap()).unwrap(),
                    )+))
                }
                fn possible_inputs(names: &'static [&str]) -> crate::PossibleInputs<'static> {
                    use itertools::Itertools;
                    let groups = std::array::IntoIter::new([$(<$name>::type_ids(),)+])
                        .multi_cartesian_product()
                        .map(|types| InputGroup {
                            info: std::array::IntoIter::new([$(std::any::type_name::<$name>(),)+])
                            .zip(names.iter().copied().zip(types))
                            .map(|(ty_name, (name, type_id))| crate::InputInfo {
                                name: name.into(),
                                ty_name,
                                type_id,
                                optional: false,
                            })
                            .collect(),
                        })
                        .collect::<Vec<_>>();
                    crate::PossibleInputs::new(groups)
                }
            }
        };
    }

    tuple_impl!(A);
    tuple_impl!(A B);
    tuple_impl!(A B C);
    tuple_impl!(A B C D);
    tuple_impl!(A B C D E);
    tuple_impl!(A B C D E F);
    tuple_impl!(A B C D E F G);
    tuple_impl!(A B C D E F G H);
}

#[cfg(test)]
mod tests {
    use crate::{FromAnyProto, InputStack, Many, One, OneOrMany};
    use std::any::{Any, TypeId};

    #[test]
    fn tuples() {
        let input_info = <(OneOrMany<u32>,)>::possible_inputs(&["value"]);
        assert_eq!(
            vec![
                TypeId::of::<OneOrMany<u32>>(),
                TypeId::of::<One<u32>>(),
                TypeId::of::<Many<u32>>()
            ]
            .into_iter()
            .collect::<std::collections::HashSet<_>>(),
            input_info
                .groups
                .iter()
                .map(|group| group.info[0].type_id)
                .collect::<std::collections::HashSet<_>>()
        );
        let input_info = <(OneOrMany<u32>, OneOrMany<u32>)>::possible_inputs(&["lhs", "rhs"]);
        assert_eq!(input_info.groups.len(), 9);

        let mut inputs: Vec<Box<dyn Any>> = vec![];
        inputs.push(Box::new(One::new(1f32)));
        inputs.push(Box::new(One::new(2f32)));
        let v: (OneOrMany<f32>, OneOrMany<f32>) =
            FromAnyProto::from_any(InputStack::new(&mut inputs, ..)).unwrap();
        assert_eq!(
            (
                OneOrMany::One(One::new(1f32)),
                OneOrMany::One(One::new(2f32))
            ),
            v
        );

        inputs.push(Box::new(One::new(1u32)));
        inputs.push(Box::new(One::new(2u32)));
        inputs.push(Box::new(One::new(3u32)));
        let v: (OneOrMany<u32>, OneOrMany<u32>, OneOrMany<u32>) =
            FromAnyProto::from_any(InputStack::new(&mut inputs, ..)).unwrap();
        assert_eq!(
            (
                OneOrMany::One(One::new(1u32)),
                OneOrMany::One(One::new(2u32)),
                OneOrMany::One(One::new(3u32))
            ),
            v
        );
    }
}
