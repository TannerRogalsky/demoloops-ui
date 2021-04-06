use nodes::{FromAnyProto, InputStack, Node, NodeInput, NodeOutput, One, PossibleInputs};
use solstice_2d::solstice::shader::RawUniformValue;
use std::any::Any;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum UniformType {
    SignedInt,
    Float,
    Mat2,
    Mat3,
    Mat4,
    Vec2,
    Vec3,
    Vec4,
}
impl UniformType {
    fn downcast(self, input: Box<dyn Any>) -> Result<RawUniformValue, Box<dyn Any>> {
        fn downcast<T>(input: Box<dyn Any>) -> Result<RawUniformValue, Box<dyn Any>>
        where
            T: Into<RawUniformValue> + 'static,
        {
            input.downcast::<One<T>>().map(|v| v.inner().into())
        }

        match self {
            UniformType::SignedInt => downcast::<i32>(input),
            UniformType::Float => downcast::<f32>(input),
            UniformType::Mat2 => downcast::<mint::ColumnMatrix2<f32>>(input),
            UniformType::Mat3 => downcast::<mint::ColumnMatrix3<f32>>(input),
            UniformType::Mat4 => downcast::<mint::ColumnMatrix4<f32>>(input),
            UniformType::Vec2 => downcast::<mint::Vector2<f32>>(input),
            UniformType::Vec3 => downcast::<mint::Vector3<f32>>(input),
            UniformType::Vec4 => downcast::<mint::Vector4<f32>>(input),
        }
    }
}
impl std::convert::TryFrom<&str> for UniformType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let ty = match value {
            "int" => UniformType::SignedInt,
            "float" => UniformType::Float,
            "mat2" => UniformType::Mat2,
            "mat3" => UniformType::Mat3,
            "mat4" => UniformType::Mat4,
            "vec2" => UniformType::Vec2,
            "vec3" => UniformType::Vec3,
            "vec4" => UniformType::Vec4,
            _ => return Err(()),
        };
        Ok(ty)
    }
}
impl Into<&'static str> for UniformType {
    fn into(self) -> &'static str {
        match self {
            UniformType::SignedInt => "signed int",
            UniformType::Float => "float",
            UniformType::Mat2 => "2x2 matrix",
            UniformType::Mat3 => "3x3 matrix",
            UniformType::Mat4 => "4x4 matrix",
            UniformType::Vec2 => "vec2",
            UniformType::Vec3 => "vec3",
            UniformType::Vec4 => "vec4",
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Uniform<'a> {
    name: &'a str,
    ty: UniformType,
    #[allow(unused)]
    array_size: Option<usize>,
}

fn split_with_index(s: &str) -> impl Iterator<Item = (&str, usize)> {
    let mut index = 0;
    std::iter::from_fn(move || {
        if index == s.len() {
            return None;
        }

        let start = index;
        for char in s[start..].chars() {
            let i = index;
            index += 1;
            if char.is_whitespace() {
                return Some((&s[start..i], i));
            }
        }

        Some((&s[start..], index))
    })
}

fn parse_uniforms(src: &str) -> impl Iterator<Item = Uniform> {
    fn parse_name(name: &str) -> (&str, Option<usize>) {
        let mut split = name
            .split(|c: char| !c.is_alphanumeric())
            .filter(|c| !c.is_empty());
        let maybe_name = split.next();
        let rest = split.next();
        if let (Some(name), Some(rest)) = (maybe_name, rest) {
            (name, rest.parse::<usize>().ok())
        } else {
            (maybe_name.unwrap_or(name), None)
        }
    }

    src.lines()
        .filter(|line| line.contains("uniform") || line.contains("extern"))
        .filter_map(|uniform| {
            let mut components = split_with_index(uniform);
            let _qualifier = components.next()?;
            let (ty, index) = components.next()?;
            let ty = std::convert::TryInto::try_into(ty).ok()?;
            let (name, array_size) = parse_name(&uniform[index..]);
            Some(Uniform {
                name,
                ty,
                array_size,
            })
        })
}

type ShaderInput = crate::command::Shader;

impl FromAnyProto for ShaderInput {
    fn from_any(inputs: nodes::InputStack<'_, Box<dyn Any>>) -> Result<Self, ()> {
        if let Some(src) = inputs.as_slice().get(0) {
            if src.is::<One<String>>() {
                let mut inputs = inputs.consume();
                let source = inputs
                    .next()
                    .unwrap()
                    .downcast::<One<String>>()
                    .unwrap()
                    .inner();
                // TODO: need idempotent check first
                let uniforms = parse_uniforms(&source)
                    .zip(inputs)
                    .map(|(uniform, input)| {
                        let v = uniform.ty.downcast(input)?;
                        Ok((uniform.name.to_owned(), v))
                    })
                    .collect::<Result<_, Box<dyn Any>>>()
                    .map_err(|_| ())?;
                Ok(ShaderInput { source, uniforms })
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    fn possible_inputs(_names: &'static [&str]) -> PossibleInputs<'static> {
        todo!()
    }
}

fn op(shader: ShaderInput) -> Box<dyn Any> {
    Box::new(One::new(shader))
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ShaderNode {
    src: std::cell::RefCell<Option<String>>,
}

impl NodeInput for ShaderNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        use nodes::{InputGroup, InputInfo};
        let src = self.src.borrow();
        if let Some(src) = &*src {
            let source_input = std::iter::once(InputInfo {
                name: "shader text".into(),
                ty_name: "String",
                type_id: std::any::TypeId::of::<One<String>>(),
                optional: false,
            });
            let uniforms_input = parse_uniforms(src).map(|uniform| InputInfo {
                name: uniform.name.to_owned().into(),
                ty_name: uniform.ty.into(),
                type_id: std::any::TypeId::of::<Uniform>(),
                optional: false,
            });
            let uniforms = source_input.chain(uniforms_input).collect::<Vec<_>>();
            let input_groups = vec![InputGroup {
                info: uniforms.into(),
            }];
            PossibleInputs::new(input_groups)
        } else {
            use once_cell::sync::Lazy;
            static GROUPS: Lazy<Vec<InputGroup<'static>>> = Lazy::new(|| {
                vec![InputGroup {
                    info: vec![InputInfo {
                        name: "shader text".into(),
                        ty_name: "String",
                        type_id: std::any::TypeId::of::<One<String>>(),
                        optional: false,
                    }]
                    .into(),
                }]
            });
            PossibleInputs::new(&*GROUPS)
        }
    }
}

impl NodeOutput for ShaderNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        let r = ShaderInput::from_any(InputStack::new(inputs, ..));
        if let Ok(v) = &r {
            self.src.replace(Some(v.source.clone()));
        }
        r.map(op)
    }
}

#[typetag::serde]
impl Node for ShaderNode {
    fn name(&self) -> &'static str {
        "shader"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WITH_UNIFORM: &str = r#"
#ifdef VERTEX
vec4 pos(mat4 transform_projection, vec4 vertex_position) {
    return transform_projection * vertex_position;
}
#endif

#ifdef FRAGMENT
uniform vec4 color;

vec4 effect(vec4 color, Image texture, vec2 texture_coords, vec2 screen_coords) {
    return color;
}
#endif
"#;

    const WITHOUT_UNIFORM: &str = r#"
#ifdef VERTEX
vec4 pos(mat4 transform_projection, vec4 vertex_position) {
    return transform_projection * vertex_position;
}
#endif

#ifdef FRAGMENT
vec4 effect(vec4 color, Image texture, vec2 texture_coords, vec2 screen_coords) {
    return Texel(texture, texture_coords) * color;
}
#endif
"#;

    const WITH_UNIFORM_ARRAY: &str = r#"
#ifdef VERTEX
vec4 pos(mat4 transform_projection, vec4 vertex_position) {
    return transform_projection * vertex_position;
}
#endif

#ifdef FRAGMENT
uniform vec4 unf1[10];
uniform mat3 unf2  [11];
uniform int unf3 ;

vec4 effect(vec4 color, Image texture, vec2 texture_coords, vec2 screen_coords) {
    return color;
}
#endif
"#;

    #[test]
    fn split_with_index_test() {
        let t = "this is   a test";
        let mut iter = split_with_index(t);
        assert_eq!(iter.next(), Some(("this", 4)));
        assert_eq!(iter.next(), Some(("is", 7)));
        assert_eq!(iter.next(), Some(("", 8)));
        assert_eq!(iter.next(), Some(("", 9)));
        assert_eq!(iter.next(), Some(("a", 11)));
        assert_eq!(iter.next(), Some(("test", 16)));
    }

    #[test]
    fn parse_uniforms_test() {
        let uniforms = parse_uniforms(WITH_UNIFORM_ARRAY).collect::<Vec<_>>();
        assert_eq!(
            uniforms,
            vec![
                Uniform {
                    name: "unf1",
                    ty: UniformType::Vec4,
                    array_size: Some(10)
                },
                Uniform {
                    name: "unf2",
                    ty: UniformType::Mat3,
                    array_size: Some(11)
                },
                Uniform {
                    name: "unf3",
                    ty: UniformType::SignedInt,
                    array_size: None
                }
            ]
        );
    }

    #[test]
    fn from_any_test() {
        let shader_node = ShaderNode::default();

        let mut inputs: Vec<Box<dyn Any>> = vec![];
        inputs.push(Box::new(One::new(String::from(WITHOUT_UNIFORM))));
        let shader_data = shader_node.op(&mut inputs).unwrap();
        assert!(shader_data.downcast::<One<ShaderInput>>().is_ok());

        let color_uniform = mint::Vector4::from([0f32, 1., 2., 3.]);
        inputs.push(Box::new(One::new(String::from(WITH_UNIFORM))));
        inputs.push(Box::new(One::new(color_uniform)));
        let shader_data = shader_node.op(&mut inputs).unwrap();
        let shader_data = shader_data.downcast::<One<ShaderInput>>().unwrap();
        assert_eq!(shader_data.source, WITH_UNIFORM);
        assert_eq!(
            shader_data.uniforms["color"],
            RawUniformValue::Vec4(color_uniform)
        );
    }

    #[test]
    fn inputs() {
        let shader_node = ShaderNode::default();

        let input_info = shader_node.inputs();
        assert_eq!(input_info.groups.len(), 1);
        assert_eq!(input_info.groups[0].info.len(), 1);
        assert_eq!(input_info.groups[0].info[0].name, "shader text");

        let mut inputs: Vec<Box<dyn Any>> = vec![];
        inputs.push(Box::new(One::new(WITH_UNIFORM.to_owned())));
        let output = shader_node.op(&mut inputs);
        assert!(output.is_ok());

        let input_info = shader_node.inputs();
        assert_eq!(input_info.groups.len(), 1);
        assert_eq!(input_info.groups[0].info.len(), 2);
        assert_eq!(input_info.groups[0].info[0].name, "shader text");
        assert_eq!(input_info.groups[0].info[1].name, "color");

        inputs.push(Box::new(One::new(WITHOUT_UNIFORM.to_owned())));
        let output = shader_node.op(&mut inputs);
        assert!(output.is_ok());

        let input_info = shader_node.inputs();
        assert_eq!(input_info.groups.len(), 1);
        assert_eq!(input_info.groups[0].info.len(), 1);
        assert_eq!(input_info.groups[0].info[0].name, "shader text");
    }

    #[test]
    fn any_cow() {
        use std::borrow::Cow;
        let control = "test";
        let b: Box<dyn Any> = Box::new(Cow::<'static, str>::Owned(String::from(control)));
        let v = b.downcast::<Cow<'static, str>>();
        assert_eq!(&*v.unwrap(), control);
    }
}
