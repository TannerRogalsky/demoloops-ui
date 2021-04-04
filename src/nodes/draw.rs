use crate::command;
use nodes::{InputGroup, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use solstice_2d::{Color, PerlinTextureSettings, Rectangle, RegularPolygon};

#[derive(nodes::InputComponent, nodes::FromAnyProto)]
enum Geometry {
    Rectangle(OneOrMany<Rectangle>),
    RegularPolygon(OneOrMany<RegularPolygon>),
}

#[derive(nodes::InputComponent, nodes::FromAnyProto)]
enum Texture {
    Noise(OneOrMany<PerlinTextureSettings>),
    Default(OneOrMany<command::DefaultTexture>),
}

#[derive(nodes::InputComponent, nodes::FromAnyProto)]
struct DrawNodeInput {
    geometry: Geometry,
    color: OneOrMany<Color>,
    texture: Texture,
}

impl DrawNodeInput {
    fn op(self) -> Box<dyn std::any::Any> {
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
    fn inputs(&self) -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static CACHE: Lazy<PossibleInputs> = Lazy::new(|| {
            use nodes::FromAnyProto;
            DrawNodeInput::possible_inputs(&["geometry", "color", "texture"])
        });
        PossibleInputs::new(&*CACHE.groups)
    }
}

impl NodeOutput for DrawNode {
    fn op(&self, inputs: &mut Vec<Box<dyn std::any::Any>>) -> Result<Box<dyn std::any::Any>, ()> {
        nodes::FromAnyProto::from_any(nodes::InputStack::new(inputs, ..)).map(DrawNodeInput::op)
    }
}

#[typetag::serde]
impl Node for DrawNode {
    fn name(&self) -> &'static str {
        "draw"
    }
}
