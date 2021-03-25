use nodes::{InputGroup, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use solstice_2d::{Color, PerlinTextureSettings, Rectangle, RegularPolygon};
use std::any::Any;

enum Geometry {
    Rectangle(OneOrMany<Rectangle>),
    RegularPolygon(OneOrMany<RegularPolygon>),
}

impl Geometry {
    fn is(v: &dyn Any) -> bool {
        OneOrMany::<Rectangle>::is(v) || OneOrMany::<RegularPolygon>::is(v)
    }

    fn downcast(v: Box<dyn Any>) -> Result<Self, Box<dyn Any>> {
        match OneOrMany::<Rectangle>::downcast(v) {
            Ok(rect) => Ok(Self::Rectangle(rect)),
            Err(v) => OneOrMany::<RegularPolygon>::downcast(v).map(Self::RegularPolygon),
        }
    }
}

struct DrawNodeInput {
    geometry: Geometry,
    color: OneOrMany<Color>,
    texture: OneOrMany<PerlinTextureSettings>,
}

impl DrawNodeInput {
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
        if inputs.len() < 3 {
            return Err(());
        }

        let valid = Geometry::is(&*inputs[0])
            && OneOrMany::<Color>::is(&*inputs[1])
            && OneOrMany::<PerlinTextureSettings>::is(&*inputs[2]);

        if !valid {
            return Err(());
        }

        fn take<T: 'static>(v: Box<dyn Any>) -> OneOrMany<T> {
            OneOrMany::<T>::downcast(v).unwrap()
        }

        let mut inputs = inputs.drain(0..3);
        let geometry = Geometry::downcast(inputs.next().unwrap()).unwrap();
        let color = take(inputs.next().unwrap());
        let texture = take(inputs.next().unwrap());
        Ok(Self {
            geometry,
            color,
            texture,
        })
    }

    fn op(self) -> Box<dyn Any> {
        use nodes::one_many::op3 as op;
        match self.geometry {
            Geometry::Rectangle(geometry) => op(
                geometry,
                self.color,
                self.texture,
                crate::command::DrawCommand::with_texture,
            ),
            Geometry::RegularPolygon(geometry) => op(
                geometry,
                self.color,
                self.texture,
                crate::command::DrawCommand::with_texture,
            ),
        }
        .into_boxed_inner()
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
            let texture = OneOrMany::<PerlinTextureSettings>::type_ids();
            std::array::IntoIter::new([rectangle, color, texture])
                .map(std::array::IntoIter::new)
                .multi_cartesian_product()
                .map(|mut types| InputGroup {
                    info: {
                        let mut types = types.drain(..);
                        vec![
                            InputInfo {
                                name: "geometry",
                                ty_name: "Rectangle",
                                type_id: types.next().unwrap(),
                            },
                            InputInfo {
                                name: "color",
                                ty_name: "Color",
                                type_id: types.next().unwrap(),
                            },
                            InputInfo {
                                name: "texture",
                                ty_name: "texture",
                                type_id: types.next().unwrap(),
                            },
                        ]
                        .into()
                    },
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
