use nodes::{InputInfo, Many, Node, NodeInput, NodeOutput, One};
use solstice_2d::Rectangle;
use std::any::Any;

struct RectangleInput<X, Y, W, H> {
    x: X,
    y: Y,
    width: W,
    height: H,
}

impl<A, B, C, D> RectangleInput<A, B, C, D>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
{
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
        if inputs.len() == 4 {
            let mut my_inputs = inputs.drain(..);
            let x = my_inputs.next().unwrap().downcast::<A>();
            let y = my_inputs.next().unwrap().downcast::<B>();
            let width = my_inputs.next().unwrap().downcast::<C>();
            let height = my_inputs.next().unwrap().downcast::<D>();
            drop(my_inputs);
            if x.is_ok() && y.is_ok() && width.is_ok() && height.is_ok() {
                Ok(RectangleInput {
                    x: *x.unwrap(),
                    y: *y.unwrap(),
                    width: *width.unwrap(),
                    height: *height.unwrap(),
                })
            } else {
                fn tx<T>(v: Result<Box<T>, Box<dyn Any>>) -> Box<dyn Any>
                where
                    T: 'static,
                {
                    match v {
                        Err(e) => e,
                        Ok(v) => v,
                    }
                }
                inputs.push(tx(x));
                inputs.push(tx(y));
                inputs.push(tx(width));
                inputs.push(tx(height));
                Err(())
            }
        } else {
            Err(())
        }
    }
}

impl<A, B, C, D> RectangleInput<A, B, C, D>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
{
    fn can_match(inputs: &[Box<dyn Any>]) -> bool {
        inputs.len() == 4
            && inputs[0].is::<A>()
            && inputs[1].is::<B>()
            && inputs[2].is::<C>()
            && inputs[3].is::<D>()
    }
}

macro_rules! rect_types {
    ($x: ty, $y: ty, $w: ty, $h: ty) => {
        impl RectangleInput<$x, $y, $w, $h> {
            pub const fn types() -> &'static [InputInfo; 4] {
                &[
                    InputInfo {
                        name: "x",
                        ty_name: stringify!($x),
                    },
                    InputInfo {
                        name: "y",
                        ty_name: stringify!($y),
                    },
                    InputInfo {
                        name: "width",
                        ty_name: stringify!($w),
                    },
                    InputInfo {
                        name: "height",
                        ty_name: stringify!($h),
                    },
                ]
            }
        }
    };
}

rect_types!(One<f32>, One<f32>, One<f32>, One<f32>);
rect_types!(Many<f32>, One<f32>, One<f32>, One<f32>);
rect_types!(Many<f32>, Many<f32>, Many<f32>, Many<f32>);

enum RectangleNodeInput {
    One(RectangleInput<One<f32>, One<f32>, One<f32>, One<f32>>),
    ManyOneOneOne(RectangleInput<Many<f32>, One<f32>, One<f32>, One<f32>>),
    Many(RectangleInput<Many<f32>, Many<f32>, Many<f32>, Many<f32>>),
}

impl RectangleNodeInput {
    fn can_match(inputs: &[Box<dyn Any>]) -> bool {
        RectangleInput::<One<f32>, One<f32>, One<f32>, One<f32>>::can_match(inputs)
            || RectangleInput::<Many<f32>, One<f32>, One<f32>, One<f32>>::can_match(inputs)
            || RectangleInput::<Many<f32>, Many<f32>, Many<f32>, Many<f32>>::can_match(inputs)
    }
    const fn types() -> &'static [&'static [InputInfo]] {
        const GROUPS: [&'static [InputInfo]; 3] = [
            RectangleInput::<One<f32>, One<f32>, One<f32>, One<f32>>::types(),
            RectangleInput::<Many<f32>, One<f32>, One<f32>, One<f32>>::types(),
            RectangleInput::<Many<f32>, Many<f32>, Many<f32>, Many<f32>>::types(),
        ];
        &GROUPS
    }
    fn op(self) -> Box<dyn Any> {
        match self {
            Self::One(RectangleInput {
                x,
                y,
                width,
                height,
            }) => Box::new(One::new(Rectangle::new(
                x.inner(),
                y.inner(),
                width.inner(),
                height.inner(),
            ))),
            Self::Many(RectangleInput {
                x,
                y,
                width,
                height,
            }) => {
                let out = x
                    .inner()
                    .zip(y.inner())
                    .zip(width.inner())
                    .zip(height.inner())
                    .map(|(((x, y), width), height)| Rectangle::new(x, y, width, height));
                Box::new(Many::from(out))
            }
            RectangleNodeInput::ManyOneOneOne(RectangleInput {
                x,
                y,
                width,
                height,
            }) => {
                let y = y.inner();
                let width = width.inner();
                let height = height.inner();
                let out = x.inner().map(move |x| Rectangle::new(x, y, width, height));
                Box::new(Many::from(out))
            }
        }
    }
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
        if let Ok(output) =
            RectangleInput::<One<f32>, One<f32>, One<f32>, One<f32>>::from_any(inputs)
        {
            Ok(RectangleNodeInput::One(output))
        } else if let Ok(output) =
            RectangleInput::<Many<f32>, One<f32>, One<f32>, One<f32>>::from_any(inputs)
        {
            Ok(RectangleNodeInput::ManyOneOneOne(output))
        } else if let Ok(output) =
            RectangleInput::<Many<f32>, Many<f32>, Many<f32>, Many<f32>>::from_any(inputs)
        {
            Ok(RectangleNodeInput::Many(output))
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RectangleNode;

impl NodeInput for RectangleNode {
    fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool {
        RectangleNodeInput::can_match(inputs)
    }

    fn inputs(&self) -> &'static [&'static [InputInfo]] {
        RectangleNodeInput::types()
    }
}

impl NodeOutput for RectangleNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        RectangleNodeInput::from_any(inputs).map(RectangleNodeInput::op)
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
}
