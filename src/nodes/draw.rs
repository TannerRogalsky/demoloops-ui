use nodes::{InputGroup, InputInfo, Many, Node, NodeInput, NodeOutput, One, PossibleInputs};
use solstice_2d::{Color, Draw, Rectangle};
use std::any::Any;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DrawNode;

impl NodeInput for DrawNode {
    fn inputs(&self) -> PossibleInputs {
        use once_cell::sync::Lazy;
        static GROUPS: Lazy<Vec<InputGroup>> = Lazy::new(|| {
            static INFO1: Lazy<Vec<InputInfo>> = Lazy::new(|| {
                vec![InputInfo {
                    name: "geometry",
                    ty_name: "One<Rectangle>",
                    type_id: std::any::TypeId::of::<One<Rectangle>>(),
                }]
            });

            static INFO2: Lazy<Vec<InputInfo>> = Lazy::new(|| {
                vec![InputInfo {
                    name: "geometry",
                    ty_name: "Many<Rectangle>",
                    type_id: std::any::TypeId::of::<Many<Rectangle>>(),
                }]
            });
            vec![
                InputGroup {
                    info: (&*INFO1).into(),
                },
                InputGroup {
                    info: (&*INFO2).into(),
                },
            ]
        });
        PossibleInputs { groups: &*GROUPS }
    }
}

impl NodeOutput for DrawNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        if self.inputs_match(&inputs) {
            if inputs[0].is::<One<Rectangle>>() {
                let rectangle = inputs.pop().unwrap().downcast::<One<Rectangle>>().unwrap();
                let mut dl = solstice_2d::DrawList::default();
                dl.draw_with_color(rectangle.inner(), Color::new(1., 1., 1., 1.));
                Ok(Box::new(One::new(dl)))
            } else {
                let rectangles = inputs.pop().unwrap().downcast::<Many<Rectangle>>().unwrap();
                let mut dl = solstice_2d::DrawList::default();
                for rectangle in rectangles.inner() {
                    dl.draw_with_color(rectangle, Color::new(1., 1., 1., 1.));
                }
                Ok(Box::new(One::new(dl)))
            }
        } else {
            Err(())
        }
    }
}

#[typetag::serde]
impl Node for DrawNode {
    fn name(&self) -> &'static str {
        "draw"
    }
}
