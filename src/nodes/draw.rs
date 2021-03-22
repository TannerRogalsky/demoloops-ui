use nodes::{InputGroup, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use solstice_2d::{Color, Draw, Rectangle};
use std::any::Any;

struct DrawNodeInput {
    geometry: OneOrMany<Rectangle>,
    color: OneOrMany<Color>,
}

impl DrawNodeInput {
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
        if inputs.len() < 2 {
            return Err(());
        }

        let valid = OneOrMany::<Rectangle>::is(&*inputs[0]) && OneOrMany::<Color>::is(&*inputs[1]);

        if !valid {
            return Err(());
        }

        fn take<T: 'static>(v: Box<dyn Any>) -> OneOrMany<T> {
            OneOrMany::<T>::downcast(v).unwrap()
        }

        let mut inputs = inputs.drain(0..2);
        let geometry = take(inputs.next().unwrap());
        let color = take(inputs.next().unwrap());
        Ok(Self { geometry, color })
    }

    fn op(self) -> Box<dyn Any> {
        let mut dl = solstice_2d::DrawList::default();
        match (self.geometry, self.color) {
            (OneOrMany::One(geometry), OneOrMany::One(color)) => {
                dl.draw_with_color(geometry.inner(), color.inner());
            }
            (geometry, color) => {
                for (geometry, color) in geometry.zip(color) {
                    dl.draw_with_color(geometry, color);
                }
            }
        }
        Box::new(nodes::One::new(dl))
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DrawNode;

impl NodeInput for DrawNode {
    fn inputs(&self) -> PossibleInputs {
        use once_cell::sync::Lazy;
        static GROUPS: Lazy<Vec<InputGroup>> = Lazy::new(|| {
            use ::nodes::InputInfo;
            use itertools::Itertools;

            let rectangle = OneOrMany::<Rectangle>::type_ids();
            let color = OneOrMany::<Color>::type_ids();
            std::array::IntoIter::new(rectangle)
                .cartesian_product(std::array::IntoIter::new(color))
                .map(|(rectangle, color)| InputGroup {
                    info: vec![
                        InputInfo {
                            name: "geometry",
                            ty_name: "Rectangle",
                            type_id: rectangle,
                        },
                        InputInfo {
                            name: "color",
                            ty_name: "Color",
                            type_id: color,
                        },
                    ]
                    .into(),
                })
                .collect::<Vec<_>>()
        });
        PossibleInputs { groups: &*GROUPS }
    }
}

impl NodeOutput for DrawNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        DrawNodeInput::from_any(inputs).map(DrawNodeInput::op)
    }
}

#[typetag::serde]
impl Node for DrawNode {
    fn name(&self) -> &'static str {
        "draw"
    }
}
