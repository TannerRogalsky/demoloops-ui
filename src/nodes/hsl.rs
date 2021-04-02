use nodes::{InputGroup, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use solstice_2d::Color;
use std::any::Any;

type HSLInput = (OneOrMany<f32>, OneOrMany<f32>, OneOrMany<f32>);

fn op((h, s, l): HSLInput) -> Box<dyn Any> {
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

    op3(h, s, l, hsl).into_boxed_inner()
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct HSLNode;

impl NodeInput for HSLNode {
    fn inputs(&self) -> PossibleInputs<'static> {
        use nodes::InputSupplemental;
        use once_cell::sync::Lazy;
        static GROUPS: Lazy<Vec<InputGroup<'static>>> =
            Lazy::new(|| HSLInput::types(&["hue", "saturation", "light"]));
        PossibleInputs::new(&*GROUPS)
    }
}

impl NodeOutput for HSLNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        nodes::FromAny::from_any(inputs).map(op)
    }
}

#[typetag::serde]
impl Node for HSLNode {
    fn name(&self) -> &'static str {
        "hsl"
    }
}
