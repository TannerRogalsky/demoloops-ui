use crate::command;
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

    fn type_ids() -> [std::any::TypeId; 6] {
        let rectangle = OneOrMany::<Rectangle>::type_ids();
        let regular = OneOrMany::<RegularPolygon>::type_ids();
        [
            rectangle[0],
            rectangle[1],
            rectangle[2],
            regular[0],
            regular[1],
            regular[2],
        ]
    }
}

enum Texture {
    Noise(OneOrMany<PerlinTextureSettings>),
    Default(OneOrMany<command::DefaultTexture>),
}

impl Texture {
    fn is(v: &dyn Any) -> bool {
        OneOrMany::<PerlinTextureSettings>::is(v) || OneOrMany::<command::DefaultTexture>::is(v)
    }

    fn downcast(v: Box<dyn Any>) -> Result<Self, Box<dyn Any>> {
        match OneOrMany::<PerlinTextureSettings>::downcast(v) {
            Ok(rect) => Ok(Self::Noise(rect)),
            Err(v) => OneOrMany::<command::DefaultTexture>::downcast(v).map(Self::Default),
        }
    }

    fn type_ids() -> [std::any::TypeId; 6] {
        let noise = OneOrMany::<PerlinTextureSettings>::type_ids();
        let white = OneOrMany::<command::DefaultTexture>::type_ids();
        [noise[0], noise[1], noise[2], white[0], white[1], white[2]]
    }
}

struct DrawNodeInput {
    geometry: Geometry,
    color: OneOrMany<Color>,
    texture: Texture,
}

impl DrawNodeInput {
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
        if inputs.len() < 3 {
            return Err(());
        }

        let valid = Geometry::is(&*inputs[0])
            && OneOrMany::<Color>::is(&*inputs[1])
            && Texture::is(&*inputs[2]);

        if !valid {
            return Err(());
        }

        fn take<T: 'static>(v: Box<dyn Any>) -> OneOrMany<T> {
            OneOrMany::<T>::downcast(v).unwrap()
        }

        let mut inputs = inputs.drain(0..3);
        let geometry = Geometry::downcast(inputs.next().unwrap()).unwrap();
        let color = take(inputs.next().unwrap());
        let texture = Texture::downcast(inputs.next().unwrap()).unwrap();
        Ok(Self {
            geometry,
            color,
            texture,
        })
    }

    fn op(self) -> Box<dyn Any> {
        use nodes::one_many::op3 as op;
        match (self.geometry, self.texture) {
            (Geometry::Rectangle(geometry), Texture::Noise(texture)) => {
                op(geometry, self.color, texture, command::DrawCommand::new)
            }
            (Geometry::RegularPolygon(geometry), Texture::Noise(texture)) => {
                op(geometry, self.color, texture, command::DrawCommand::new)
            }
            (Geometry::Rectangle(geometry), Texture::Default(texture)) => {
                op(geometry, self.color, texture, command::DrawCommand::new)
            }
            (Geometry::RegularPolygon(geometry), Texture::Default(texture)) => {
                op(geometry, self.color, texture, command::DrawCommand::new)
            }
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

            let rectangle = Geometry::type_ids().to_vec();
            let color = OneOrMany::<Color>::type_ids().to_vec();
            let texture = Texture::type_ids().to_vec();
            std::array::IntoIter::new([rectangle, color, texture])
                .map(std::vec::Vec::into_iter)
                .multi_cartesian_product()
                .map(|mut types| InputGroup {
                    info: {
                        let mut types = types.drain(..);
                        vec![
                            InputInfo {
                                name: "geometry",
                                ty_name: "Geometry",
                                type_id: types.next().unwrap(),
                            },
                            InputInfo {
                                name: "color",
                                ty_name: "Color",
                                type_id: types.next().unwrap(),
                            },
                            InputInfo {
                                name: "texture",
                                ty_name: "Texture",
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
