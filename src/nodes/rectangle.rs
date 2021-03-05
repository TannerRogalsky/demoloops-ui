use crate::{Many, Node, NodeInput, NodeOutput, One};
use solstice_2d::Rectangle;
use std::any::Any;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RectangleNode;

impl NodeInput for RectangleNode {
    fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool {
        if inputs.len() == 4 {
            inputs.iter().all(|input| input.is::<One<f32>>())
                || inputs.iter().all(|input| input.is::<Many<f32>>())
        } else {
            false
        }
    }
}

impl NodeOutput for RectangleNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        if self.inputs_match(&inputs) {
            if inputs.iter().all(|input| input.is::<One<f32>>()) {
                let height = inputs.pop().unwrap().downcast::<One<f32>>().unwrap().0;
                let width = inputs.pop().unwrap().downcast::<One<f32>>().unwrap().0;
                let y = inputs.pop().unwrap().downcast::<One<f32>>().unwrap().0;
                let x = inputs.pop().unwrap().downcast::<One<f32>>().unwrap().0;
                Ok(Box::new(One(Rectangle::new(x, y, width, height))))
            } else if inputs.iter().all(|input| input.is::<Many<f32>>()) {
                let height = inputs.pop().unwrap().downcast::<Many<f32>>().unwrap().0;
                let width = inputs.pop().unwrap().downcast::<Many<f32>>().unwrap().0;
                let y = inputs.pop().unwrap().downcast::<Many<f32>>().unwrap().0;
                let x = inputs.pop().unwrap().downcast::<Many<f32>>().unwrap().0;

                let out = x
                    .zip(y)
                    .zip(width)
                    .zip(height)
                    .map(|(((x, y), width), height)| Rectangle::new(x, y, width, height));
                Ok(Box::new(Many::<Rectangle>::from(out)))
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

#[typetag::serde]
impl Node for RectangleNode {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one() {
        let mut inputs: Vec<Box<dyn Any>> = vec![
            Box::new(One(100f32)),
            Box::new(One(100f32)),
            Box::new(One(100f32)),
            Box::new(One(100f32)),
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
