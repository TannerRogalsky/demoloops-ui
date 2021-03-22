use nodes::{InputGroup, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use solstice_2d::RegularPolygon;
use std::any::Any;

struct RegularPolygonInput {
    x: OneOrMany<f32>,
    y: OneOrMany<f32>,
    vertex_count: OneOrMany<u32>,
    radius: OneOrMany<f32>,
}

impl RegularPolygonInput {
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
        if inputs.len() < 4 {
            return Err(());
        }

        let valid = OneOrMany::<f32>::is(&*inputs[0])
            && OneOrMany::<f32>::is(&*inputs[1])
            && OneOrMany::<u32>::is(&*inputs[2])
            && OneOrMany::<f32>::is(&*inputs[3]);

        if !valid {
            return Err(());
        }

        fn take<T: 'static>(v: Box<dyn Any>) -> OneOrMany<T> {
            OneOrMany::<T>::downcast(v).unwrap()
        }

        let mut inputs = inputs.drain(0..4);
        let x = take(inputs.next().unwrap());
        let y = take(inputs.next().unwrap());
        let vertex_count = take(inputs.next().unwrap());
        let radius = take(inputs.next().unwrap());
        Ok(Self {
            x,
            y,
            vertex_count,
            radius,
        })
    }

    fn gen_groups() -> Vec<InputGroup<'static>> {
        use ::nodes::InputInfo;
        use itertools::Itertools;

        let x = std::array::IntoIter::new(OneOrMany::<f32>::type_ids());
        let y = std::array::IntoIter::new(OneOrMany::<f32>::type_ids());
        let vertex_count = std::array::IntoIter::new(OneOrMany::<u32>::type_ids());
        let radius = std::array::IntoIter::new(OneOrMany::<f32>::type_ids());
        let groups = std::array::IntoIter::new([x, y, vertex_count, radius])
            .multi_cartesian_product()
            .map(|mut types| {
                let mut types = types.drain(..);
                InputGroup {
                    info: vec![
                        InputInfo {
                            name: "x",
                            ty_name: "f32",
                            type_id: types.next().unwrap(),
                        },
                        InputInfo {
                            name: "y",
                            ty_name: "f32",
                            type_id: types.next().unwrap(),
                        },
                        InputInfo {
                            name: "vertex_count",
                            ty_name: "u32",
                            type_id: types.next().unwrap(),
                        },
                        InputInfo {
                            name: "radius",
                            ty_name: "f32",
                            type_id: types.next().unwrap(),
                        },
                    ]
                    .into(),
                }
            })
            .collect::<Vec<_>>();
        groups
    }

    fn types() -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;

        static GROUPS: Lazy<Vec<InputGroup<'static>>> = Lazy::new(RegularPolygonInput::gen_groups);
        PossibleInputs { groups: &*GROUPS }
    }

    fn op(self) -> Box<dyn Any> {
        use nodes::one_many::op4;
        op4(
            self.x,
            self.y,
            self.vertex_count,
            self.radius,
            RegularPolygon::new,
        )
        .into_boxed_inner()
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RegularPolygonNode;

impl NodeInput for RegularPolygonNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        RegularPolygonInput::types()
    }
}

impl NodeOutput for RegularPolygonNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        RegularPolygonInput::from_any(inputs).map(RegularPolygonInput::op)
    }
}

#[typetag::serde]
impl Node for RegularPolygonNode {
    fn name(&self) -> &'static str {
        "regular polygon"
    }
}
