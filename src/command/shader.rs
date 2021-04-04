use solstice_2d::solstice::shader::RawUniformValue;

pub struct Shader {
    pub source: String,
    pub uniforms: std::collections::HashMap<String, RawUniformValue>,
}
