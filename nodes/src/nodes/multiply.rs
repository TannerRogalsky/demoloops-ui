use crate::{FromAny, Many, Node, NodeInput, NodeOutput, One, Pair};
use std::any::Any;

#[derive(Debug, Clone)]
enum MultiplyGroup<T> {
    OneOne(Pair<One<T>, One<T>>),
    OneMany(Pair<One<T>, Many<T>>),
    ManyMany(Pair<Many<T>, Many<T>>),
}

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
    fn can_match(inputs: &[Box<dyn Any>]) -> bool {
        Pair::<One<T>, One<T>>::can_match(inputs)
            || Pair::<One<T>, Many<T>>::can_match(inputs)
            || Pair::<Many<T>, Many<T>>::can_match(inputs)
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

    fn can_match(inputs: &[Box<dyn Any>]) -> bool {
        MultiplyGroup::<f32>::can_match(inputs) || MultiplyGroup::<u32>::can_match(inputs)
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
    fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool {
        MultiplyNodeInput::can_match(inputs)
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
    #![allow(unused)]
    use crate::{Many, NodeOutput, One};

    #[derive(Debug, Copy, Clone)]
    struct InputInfo {
        name: &'static str,
        ty_name: &'static str,
    }

    #[derive(Debug, Eq, PartialEq)]
    struct Pair<A, B> {
        lhs: A,
        rhs: B,
    }

    impl<A, B> Pair<A, B>
    where
        A: 'static,
        B: 'static,
    {
        pub fn from_any(inputs: &mut Vec<Box<dyn std::any::Any>>) -> Result<Self, ()> {
            if let Some(rhs) = inputs.pop() {
                if let Some(lhs) = inputs.pop() {
                    let lhs = lhs.downcast::<A>();
                    let rhs = rhs.downcast::<B>();
                    match (lhs, rhs) {
                        (Ok(lhs), Ok(rhs)) => Ok(Self {
                            lhs: *lhs,
                            rhs: *rhs,
                        }),
                        (Ok(lhs), Err(rhs)) => {
                            inputs.push(lhs);
                            inputs.push(rhs);
                            Err(())
                        }
                        (Err(lhs), Ok(rhs)) => {
                            inputs.push(lhs);
                            inputs.push(rhs);
                            Err(())
                        }
                        (Err(lhs), Err(rhs)) => {
                            inputs.push(lhs);
                            inputs.push(rhs);
                            Err(())
                        }
                    }
                } else {
                    inputs.push(rhs);
                    Err(())
                }
            } else {
                Err(())
            }
        }
    }

    macro_rules! pair_impl {
        ($a: ty, $b: ty) => {
            impl Pair<$a, $b> {
                const fn types() -> &'static [InputInfo; 2] {
                    const V: [InputInfo; 2] = [
                        InputInfo {
                            name: "lhs",
                            ty_name: stringify!($a),
                        },
                        InputInfo {
                            name: "rhs",
                            ty_name: stringify!($b),
                        },
                    ];
                    &V
                }
            }
        };
    }

    pair_impl!(One<f32>, One<f32>);
    pair_impl!(One<f32>, Many<f32>);
    pair_impl!(Many<f32>, Many<f32>);
    pair_impl!(One<u32>, One<u32>);
    pair_impl!(One<u32>, Many<u32>);
    pair_impl!(Many<u32>, Many<u32>);

    enum MultGroup<T> {
        OneOne(Pair<One<T>, One<T>>),
        OneMany(Pair<One<T>, Many<T>>),
        ManyMany(Pair<Many<T>, Many<T>>),
    }

    impl<T> MultGroup<T>
    where
        T: 'static,
    {
        pub fn from_any(inputs: &mut Vec<Box<dyn std::any::Any>>) -> Result<Self, ()> {
            if let Ok(one_one) = Pair::<One<T>, One<T>>::from_any(inputs) {
                Ok(MultGroup::OneOne(one_one))
            } else if let Ok(one_many) = Pair::<One<T>, Many<T>>::from_any(inputs) {
                Ok(MultGroup::OneMany(one_many))
            } else if let Ok(many_many) = Pair::<Many<T>, Many<T>>::from_any(inputs) {
                Ok(MultGroup::ManyMany(many_many))
            } else {
                Err(())
            }
        }
    }

    macro_rules! group_impl {
        ($t: ty) => {
            impl MultGroup<$t> {
                const fn types() -> [&'static [InputInfo]; 3] {
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

    // This would need to flatten the inputs for all fields.
    enum Mult {
        F32(MultGroup<f32>),
        U32(MultGroup<u32>),
    }

    impl Mult {
        pub fn from_any(inputs: &mut Vec<Box<dyn std::any::Any>>) -> Result<Self, ()> {
            if let Ok(output) = MultGroup::<f32>::from_any(inputs) {
                Ok(Mult::F32(output))
            } else if let Ok(output) = MultGroup::<u32>::from_any(inputs) {
                Ok(Mult::U32(output))
            } else {
                Err(())
            }
        }
    }

    #[test]
    fn static_ref_trait() {
        trait Test: std::fmt::Debug {
            fn len(&self) -> usize;
        }
        impl Test for [InputInfo] {
            fn len(&self) -> usize {
                self.len()
            }
        }

        const fn concat<'a, const A: usize, const B: usize, const C: usize>(
            a: [&'a [InputInfo]; A],
            b: [&'a [InputInfo]; B],
        ) -> [&'a [InputInfo]; C] {
            // Assert that `A + B == C`.
            // These overflow if that is not the case, which produces an error at compile-time.
            let _ = C - (A + B); // Assert that `A + B <= C`
            let _ = (A + B) - C; // Assert that `A + B >= C`

            let mut result: [&'a [InputInfo]; C] = [&[]; C];

            let mut i = 0;
            while i < A {
                result[i] = a[i];
                i += 1;
            }

            while i < A + B {
                result[i] = b[i - A];
                i += 1;
            }

            result
        }

        const fn types() -> &'static [&'static [InputInfo]] {
            const O3: [&'static [InputInfo]; 6] =
                concat(MultGroup::<f32>::types(), MultGroup::<u32>::types());
            &O3
        }
        let o3 = types();
        assert_eq!(6, o3.len());

        {
            let mut inputs: Vec<Box<dyn std::any::Any>> = vec![];
            inputs.push(Box::new(One::new(1u32)));
            inputs.push(Box::new(One::new(2u32)));
            let p = Pair::<One<u32>, One<u32>>::from_any(&mut inputs).unwrap();
            assert_eq!(
                Pair {
                    lhs: One::new(1u32),
                    rhs: One::new(2u32)
                },
                p
            );
            assert!(inputs.is_empty());

            // extra weird types
            inputs.push(Box::new(()));
            inputs.push(Box::new(|| 1));
            let p = Pair::<One<f32>, Many<f32>>::from_any(&mut inputs);
            assert!(p.is_err());
            assert_eq!(2, inputs.len());
        }

        {
            let mut inputs: Vec<Box<dyn std::any::Any>> = vec![];
            inputs.push(Box::new(One::new(3u32)));
            inputs.push(Box::new(One::new(2u32)));

            let node = super::MultiplyNode;
            let output = node.op(&mut inputs);
            assert!(inputs.is_empty());
            let output = output.unwrap().downcast::<One<u32>>().unwrap();
            assert_eq!(6u32, output.inner());
        }
    }
}
