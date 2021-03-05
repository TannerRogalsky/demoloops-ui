use crate::{Many, Node, NodeInput, NodeOutput, One};
use solstice_2d::{Color, Draw, Rectangle};
use std::any::Any;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DrawNode;

impl NodeInput for DrawNode {
    fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool {
        if inputs.len() == 1 {
            inputs[0].is::<One<Rectangle>>() || inputs[0].is::<Many<Rectangle>>()
        } else {
            false
        }
    }
}

impl NodeOutput for DrawNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        if self.inputs_match(&inputs) {
            if inputs[0].is::<One<Rectangle>>() {
                let rectangle = inputs.pop().unwrap().downcast::<One<Rectangle>>().unwrap();
                let mut dl = solstice_2d::DrawList::default();
                dl.clear(Color::new(0., 0., 0., 1.));
                dl.draw_with_color(rectangle.0, Color::new(1., 1., 1., 1.));
                Ok(Box::new(One(dl)))
            } else {
                let rectangles = inputs.pop().unwrap().downcast::<Many<Rectangle>>().unwrap();
                let mut dl = solstice_2d::DrawList::default();
                dl.clear(Color::new(0., 0., 0., 1.));
                for rectangle in rectangles.0 {
                    dl.draw_with_color(rectangle, Color::new(1., 1., 1., 1.));
                }
                Ok(Box::new(One(dl)))
            }
        } else {
            Err(())
        }
    }
}

#[typetag::serde]
impl Node for DrawNode {}
