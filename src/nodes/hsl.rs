use nodes::{InputGroup, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use solstice_2d::Color;
use std::any::Any;

struct HSLInput {
    h: OneOrMany<f32>,
    s: OneOrMany<f32>,
    l: OneOrMany<f32>,
}

impl HSLInput {
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
        if inputs.len() < 3 {
            return Err(());
        }

        let valid = inputs[0..3]
            .iter()
            .all(|input| OneOrMany::<f32>::is(&**input));

        if !valid {
            return Err(());
        }

        let mut inputs = inputs
            .drain(0..3)
            .map(OneOrMany::<f32>::downcast)
            .map(Result::unwrap);
        let h = inputs.next().unwrap();
        let s = inputs.next().unwrap();
        let l = inputs.next().unwrap();
        Ok(Self { h, s, l })
    }
}

impl HSLInput {
    fn op(self) -> Box<dyn Any> {
        use ::nodes::one_many::op3;
        fn hue_to_rgb(p: f32, q: f32, t: f32) -> f32 {
            // Normalize
            let t = if t < 0.0 {
                t + 1.0
            } else if t > 1.0 {
                t - 1.0
            } else {
                t
            };

            if t < 1.0 / 6.0 {
                p + (q - p) * 6.0 * t
            } else if t < 1.0 / 2.0 {
                q
            } else if t < 2.0 / 3.0 {
                p + (q - p) * (2.0 / 3.0 - t) * 6.0
            } else {
                p
            }
        }

        fn hsl(h: f32, s: f32, l: f32) -> Color {
            if s == 0.0 {
                // Achromatic, i.e., grey.
                return Color::new(l, l, l, 1.);
            }

            let h = h / (std::f32::consts::PI * 2.);
            let s = s;
            let l = l;

            let q = if l < 0.5 {
                l * (1.0 + s)
            } else {
                l + s - (l * s)
            };
            let p = 2.0 * l - q;

            Color::new(
                hue_to_rgb(p, q, h + 1.0 / 3.0),
                hue_to_rgb(p, q, h),
                hue_to_rgb(p, q, h - 1.0 / 3.0),
                1.,
            )
        }

        op3(self.h, self.s, self.l, hsl).into_boxed_inner()
    }

    fn gen_groups() -> Vec<InputGroup<'static>> {
        use itertools::Itertools;
        use nodes::InputInfo;

        let h = std::array::IntoIter::new(OneOrMany::<f32>::type_ids());
        let s = std::array::IntoIter::new(OneOrMany::<f32>::type_ids());
        let l = std::array::IntoIter::new(OneOrMany::<f32>::type_ids());
        let groups = h
            .cartesian_product(s.cartesian_product(l))
            .map(|(h, (s, l))| InputGroup {
                info: vec![
                    InputInfo {
                        name: "hue",
                        ty_name: "f32",
                        type_id: h,
                    },
                    InputInfo {
                        name: "saturation",
                        ty_name: "f32",
                        type_id: s,
                    },
                    InputInfo {
                        name: "light",
                        ty_name: "f32",
                        type_id: l,
                    },
                ]
                .into(),
            })
            .collect::<Vec<_>>();
        groups
    }

    fn types() -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;

        static GROUPS: Lazy<Vec<InputGroup<'static>>> = Lazy::new(HSLInput::gen_groups);
        PossibleInputs { groups: &*GROUPS }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct HSLNode;

impl NodeInput for HSLNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        HSLInput::types()
    }
}

impl NodeOutput for HSLNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        HSLInput::from_any(inputs).map(HSLInput::op)
    }
}

#[typetag::serde]
impl Node for HSLNode {
    fn name(&self) -> &'static str {
        "hsl"
    }
}
