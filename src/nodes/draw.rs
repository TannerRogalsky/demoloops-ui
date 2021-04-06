use crate::command;
use nodes::{Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use solstice_2d::{Color, PerlinTextureSettings, Rectangle, RegularPolygon, Transform3D};

#[derive(nodes::InputComponent, nodes::FromAnyProto)]
enum Geometry {
    Rectangle(OneOrMany<Rectangle>),
    RegularPolygon(OneOrMany<RegularPolygon>),
}

#[derive(nodes::InputComponent, nodes::FromAnyProto)]
struct DrawNodeInput {
    geometry: Geometry,
    transform: Option<OneOrMany<Transform3D>>,
    color: Option<OneOrMany<Color>>,
    texture: Option<OneOrMany<PerlinTextureSettings>>,
    shader: Option<OneOrMany<command::Shader>>,
}

impl DrawNodeInput {
    fn op(self) -> Box<dyn std::any::Any> {
        use nodes::one_many::{op4, op5};
        let color = self
            .color
            .unwrap_or(OneOrMany::One(nodes::One::new(Color::new(1., 1., 1., 1.))));
        let texture = match self.texture {
            None => OneOrMany::One(nodes::One::new(None)),
            Some(noise) => nodes::one_many::op1(noise, |v| Some(v)),
        };
        let transform = self
            .transform
            .unwrap_or_else(|| OneOrMany::One(nodes::One::new(Transform3D::default())));

        match self.geometry {
            Geometry::Rectangle(geometry) => match self.shader {
                None => op4(
                    geometry,
                    transform,
                    color,
                    texture,
                    command::DrawCommand::new,
                ),
                Some(shader) => op5(
                    geometry,
                    transform,
                    color,
                    texture,
                    shader,
                    command::DrawCommand::with_shader,
                ),
            },
            Geometry::RegularPolygon(geometry) => match self.shader {
                None => op4(
                    geometry,
                    transform,
                    color,
                    texture,
                    command::DrawCommand::new,
                ),
                Some(shader) => op5(
                    geometry,
                    transform,
                    color,
                    texture,
                    shader,
                    command::DrawCommand::with_shader,
                ),
            },
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
            DrawNodeInput::possible_inputs(&["geometry", "transform", "color", "texture", "shader"])
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{DrawCommand, Shader};
    use nodes::One;
    use std::any::Any;

    #[test]
    fn inputs() {
        let shader_node = crate::ShaderNode::default();
        let draw_node = DrawNode::default();

        let geometry = Box::new(One::new(crate::Rectangle::new(0., 0., 100., 100.)));

        let mut inputs: Vec<Box<dyn Any>> = vec![];
        inputs.push(Box::new(One::new(SHADER_SRC.to_string())));
        let shader = shader_node.op(&mut inputs).unwrap();
        assert!((&*shader).is::<One<Shader>>());

        inputs.push(geometry);
        inputs.push(Box::new(Option::<()>::None));
        inputs.push(Box::new(Option::<()>::None));
        inputs.push(shader);
        let command = draw_node
            .op(&mut inputs)
            .unwrap()
            .downcast::<One<DrawCommand>>()
            .unwrap();

        assert_eq!(command.inner().shader.unwrap().source.as_str(), SHADER_SRC);
    }

    const SHADER_SRC: &str = r#"
#ifdef VERTEX
vec4 pos(mat4 transform_projection, vec4 vertex_position) {
    return transform_projection * vertex_position;
}
#endif

#ifdef FRAGMENT
uniform float offset;

vec4 effect(vec4 color, Image texture, vec2 texture_coords, vec2 screen_coords) {
    vec2 p = texture_coords - offset;
    float r = sqrt(dot(p, p));
    return mix(color, vec4(0.), r);
}
#endif
"#;
}
