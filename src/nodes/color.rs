use nodes::{InputGroup, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use solstice_2d::Color;
use std::any::Any;

struct ColorInput<R, G, B, A> {
    r: OneOrMany<R>,
    g: OneOrMany<G>,
    b: OneOrMany<B>,
    a: OneOrMany<A>,
}

impl<R, G, B, A> ColorInput<R, G, B, A>
where
    R: 'static,
    G: 'static,
    B: 'static,
    A: 'static,
{
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
        if inputs.len() < 4 {
            return Err(());
        }

        let valid = OneOrMany::<R>::is(&*inputs[0])
            && OneOrMany::<G>::is(&*inputs[1])
            && OneOrMany::<B>::is(&*inputs[2])
            && OneOrMany::<A>::is(&*inputs[3]);

        if !valid {
            return Err(());
        }

        fn take<T: 'static>(v: Box<dyn Any>) -> OneOrMany<T> {
            OneOrMany::<T>::downcast(v).unwrap()
        }

        let mut inputs = inputs.drain(0..4);
        let r = take(inputs.next().unwrap());
        let g = take(inputs.next().unwrap());
        let b = take(inputs.next().unwrap());
        let a = take(inputs.next().unwrap());
        Ok(Self { r, g, b, a })
    }
}

macro_rules! group_impl {
    ($r: ty, $g: ty, $b: ty, $a: ty) => {
        impl ColorInput<$r, $g, $b, $a> {
            fn gen_groups() -> Vec<InputGroup<'static>> {
                use ::nodes::InputInfo;
                use itertools::Itertools;

                let r = OneOrMany::<$r>::type_ids();
                let g = OneOrMany::<$g>::type_ids();
                let b = OneOrMany::<$b>::type_ids();
                let a = OneOrMany::<$a>::type_ids();
                let groups = std::array::IntoIter::new([r, g, b, a])
                    .map(|v| std::array::IntoIter::new(v))
                    .multi_cartesian_product()
                    .map(|mut types| {
                        let mut types = types.drain(..);
                        InputGroup {
                            info: vec![
                                InputInfo {
                                    name: "red",
                                    ty_name: stringify!($r),
                                    type_id: types.next().unwrap(),
                                },
                                InputInfo {
                                    name: "green",
                                    ty_name: stringify!($g),
                                    type_id: types.next().unwrap(),
                                },
                                InputInfo {
                                    name: "blue",
                                    ty_name: stringify!($b),
                                    type_id: types.next().unwrap(),
                                },
                                InputInfo {
                                    name: "alpha",
                                    ty_name: stringify!($a),
                                    type_id: types.next().unwrap(),
                                },
                            ]
                            .into(),
                        }
                    })
                    .collect::<Vec<_>>();
                groups
            }

            fn types() -> &'static [InputGroup<'static>] {
                use once_cell::sync::Lazy;

                static GROUPS: Lazy<Vec<InputGroup<'static>>> =
                    Lazy::new(ColorInput::<$r, $g, $b, $a>::gen_groups);
                &*GROUPS
            }
        }
    };
}
group_impl!(f32, f32, f32, f32);

impl ColorInput<f32, f32, f32, f32> {
    fn op(self) -> Box<dyn Any> {
        use ::nodes::one_many::op4;
        let result = op4(self.r, self.g, self.b, self.a, Color::new);
        match result {
            OneOrMany::One(v) => Box::new(v),
            OneOrMany::Many(v) => Box::new(v),
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ColorNode;

impl NodeInput for ColorNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        PossibleInputs {
            groups: ColorInput::types(),
        }
    }
}

impl NodeOutput for ColorNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        ColorInput::<f32, f32, f32, f32>::from_any(inputs).map(ColorInput::<f32, f32, f32, f32>::op)
    }
}

#[typetag::serde]
impl Node for ColorNode {
    fn name(&self) -> &'static str {
        "color"
    }
}
