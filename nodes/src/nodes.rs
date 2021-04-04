mod add;
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
    use crate::{FromAny, InputGroup, InputSupplemental, OneOrMany};
    use std::any::Any;

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
            impl<$($name: 'static),+> FromAny for ($(OneOrMany<$name>,)+) {
                fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
                    let len = count_idents!($($name, )+);
                    if inputs.len() < len {
                        return Err(())
                    }

                    let mut checker = inputs.iter();
                    $(
                        if !OneOrMany::<$name>::is(&**checker.next().unwrap()) {
                            return Err(())
                        }
                    )+

                    let mut inputs = inputs.drain(0..len);
                    Ok(($(
                        OneOrMany::<$name>::downcast(inputs.next().unwrap()).unwrap(),
                    )+))
                }
            }

            impl<$($name: 'static + Clone + std::fmt::Debug),+> InputSupplemental for ($(OneOrMany<$name>,)+) {
                fn types(names: &'static [&str]) -> Vec<InputGroup<'static>> {
                    use itertools::Itertools;
                    let groups = std::array::IntoIter::new([$(OneOrMany::<$name>::type_ids(),)+])
                        .map(|v| std::array::IntoIter::new(v))
                        .multi_cartesian_product()
                        .map(|types| InputGroup {
                            info: std::array::IntoIter::new([$(std::any::type_name::<$name>(),)+])
                            .zip(names.iter().copied().zip(types))
                            .map(|(ty_name, (name, type_id))| crate::InputInfo {
                                name: name.into(),
                                ty_name,
                                type_id,
                            })
                            .collect(),
                        })
                        .collect::<Vec<_>>();
                    groups
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
    use crate::{FromAny, InputSupplemental, Many, One, OneOrMany};
    use std::any::{Any, TypeId};

    #[test]
    fn tuples() {
        let input_info = <(OneOrMany<u32>,)>::types(&["value"]);
        assert_eq!(
            vec![
                TypeId::of::<OneOrMany<u32>>(),
                TypeId::of::<One<u32>>(),
                TypeId::of::<Many<u32>>()
            ],
            input_info
                .iter()
                .map(|group| group.info[0].type_id)
                .collect::<Vec<_>>()
        );
        let input_info = <(OneOrMany<u32>, OneOrMany<u32>)>::types(&["lhs", "rhs"]);
        assert_eq!(input_info.len(), 9);

        let mut inputs: Vec<Box<dyn Any>> = vec![];
        inputs.push(Box::new(One::new(1f32)));
        inputs.push(Box::new(One::new(2f32)));
        let v: (OneOrMany<f32>, OneOrMany<f32>) = FromAny::from_any(&mut inputs).unwrap();
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
            FromAny::from_any(&mut inputs).unwrap();
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
