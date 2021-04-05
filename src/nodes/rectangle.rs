use nodes::{FromAnyProto, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use solstice_2d::Rectangle;
use std::any::Any;

type RectangleInput = (
    OneOrMany<f32>,
    OneOrMany<f32>,
    OneOrMany<f32>,
    OneOrMany<f32>,
);

fn op((x, y, width, height): RectangleInput) -> Box<dyn Any> {
    use ::nodes::one_many::op4;
    op4(x, y, width, height, Rectangle::new).into_boxed_inner()
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RectangleNode;

impl NodeInput for RectangleNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static CACHE: Lazy<PossibleInputs> =
            Lazy::new(|| RectangleInput::possible_inputs(&["x", "y", "width", "height"]));
        PossibleInputs::new(&*CACHE.groups)
    }
}

impl NodeOutput for RectangleNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        FromAnyProto::from_any(nodes::InputStack::new(inputs, ..)).map(op)
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
