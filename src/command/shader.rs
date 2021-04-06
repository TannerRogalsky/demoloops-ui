use solstice_2d::solstice::shader::RawUniformValue;

#[derive(Clone, Debug)]
pub struct Shader {
    pub source: String,
    pub uniforms: std::collections::HashMap<String, RawUniformValue>,
}
