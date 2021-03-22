use nodes::{InputGroup, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use solstice_2d::Rectangle;
use std::any::Any;

#[derive(Debug)]
struct RectangleInput<X, Y, W, H> {
    x: OneOrMany<X>,
    y: OneOrMany<Y>,
    width: OneOrMany<W>,
    height: OneOrMany<H>,
}

impl<X, Y, W, H> RectangleInput<X, Y, W, H>
where
    X: 'static,
    Y: 'static,
    W: 'static,
    H: 'static,
{
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
        if inputs.len() < 4 {
            return Err(());
        }

        let valid = OneOrMany::<X>::is(&*inputs[0])
            && OneOrMany::<Y>::is(&*inputs[1])
            && OneOrMany::<W>::is(&*inputs[2])
            && OneOrMany::<H>::is(&*inputs[3]);

        if !valid {
            return Err(());
        }

        fn take<T: 'static>(v: Box<dyn Any>) -> OneOrMany<T> {
            OneOrMany::<T>::downcast(v).unwrap()
        }

        let mut inputs = inputs.drain(0..4);
        let x = take(inputs.next().unwrap());
        let y = take(inputs.next().unwrap());
        let width = take(inputs.next().unwrap());
        let height = take(inputs.next().unwrap());
        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }
}

macro_rules! group_impl {
    ($x: ty, $y: ty, $w: ty, $h: ty) => {
        impl RectangleInput<$x, $y, $w, $h> {
            fn gen_groups() -> Vec<InputGroup<'static>> {
                use ::nodes::InputInfo;
                use itertools::Itertools;

                let x = OneOrMany::<$x>::type_ids();
                let y = OneOrMany::<$y>::type_ids();
                let width = OneOrMany::<$w>::type_ids();
                let height = OneOrMany::<$h>::type_ids();
                let groups = std::array::IntoIter::new([x, y, width, height])
                    .map(|v| std::array::IntoIter::new(v))
                    .multi_cartesian_product()
                    .map(|mut types| {
                        let mut types = types.drain(..);
                        InputGroup {
                            info: vec![
                                InputInfo {
                                    name: "x",
                                    ty_name: stringify!($x),
                                    type_id: types.next().unwrap(),
                                },
                                InputInfo {
                                    name: "y",
                                    ty_name: stringify!($y),
                                    type_id: types.next().unwrap(),
                                },
                                InputInfo {
                                    name: "width",
                                    ty_name: stringify!($w),
                                    type_id: types.next().unwrap(),
                                },
                                InputInfo {
                                    name: "height",
                                    ty_name: stringify!($h),
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
                    Lazy::new(RectangleInput::<$x, $y, $w, $h>::gen_groups);
                &*GROUPS
            }
        }
    };
}

impl RectangleInput<f32, f32, f32, f32> {
    fn op(self) -> Box<dyn Any> {
        use ::nodes::one_many::op4;
        op4(self.x, self.y, self.width, self.height, Rectangle::new).into_boxed_inner()
    }
}
group_impl!(f32, f32, f32, f32);

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RectangleNode;

impl NodeInput for RectangleNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        PossibleInputs {
            groups: RectangleInput::types(),
        }
    }
}

impl NodeOutput for RectangleNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        RectangleInput::<f32, f32, f32, f32>::from_any(inputs)
            .map(RectangleInput::<f32, f32, f32, f32>::op)
    }
}

#[typetag::serde]
impl Node for RectangleNode {
    fn name(&self) -> &'static str {
        "rectangle"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::nodes::{Many, One};

    #[test]
    fn one() {
        let mut inputs: Vec<Box<dyn Any>> = vec![
            Box::new(One::new(100f32)),
            Box::new(One::new(100f32)),
            Box::new(One::new(100f32)),
            Box::new(One::new(100f32)),
        ];
        let output = RectangleNode::default().op(&mut inputs);
        let rect = output.unwrap().downcast::<One<Rectangle>>();
        assert!(rect.is_ok())
    }

    #[test]
    fn many() {
        let count = 4usize;
        let mut inputs = (0..count)
            .map::<Box<dyn Any>, _>(|_| Box::new(Many::from(vec![1f32, 2., 3., 4.])))
            .collect::<Vec<_>>();
        let output = RectangleNode::default().op(&mut inputs);
        let rects = output.unwrap().downcast::<Many<Rectangle>>();
        let rects = rects.unwrap().collect::<Vec<_>>();
        assert_eq!(count, rects.len());
    }

    #[test]
    fn best_match() {
        let inputs: Vec<Box<dyn Any>> = vec![
            Box::new(One::new(1f32)),
            Box::new(One::new(2f32)),
            Box::new(One::new(3f32)),
            Box::new(One::new(4u32)),
        ];
        let input_info = RectangleNode.inputs();
        let group = input_info.best_match(&inputs).expect("expected error");
        assert_eq!(3, group.score(&inputs));
    }
}
