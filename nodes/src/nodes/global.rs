use crate::{One, PossibleInputs};
use std::any::Any;
use std::sync::atomic::{AtomicU32, Ordering};

static TICK: AtomicU32 = AtomicU32::new(1);

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct GlobalNode;

impl GlobalNode {
    pub fn incr() -> u32 {
        TICK.fetch_add(1, Ordering::SeqCst)
    }

    pub fn load() -> u32 {
        TICK.load(Ordering::SeqCst)
    }
}

impl crate::NodeInput for GlobalNode {
    fn is_terminator(&self) -> bool {
        true
    }
    fn inputs(&self) -> PossibleInputs {
        PossibleInputs { groups: &[] }
    }
}

impl crate::NodeOutput for GlobalNode {
    fn op(&self, _inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        Ok(Box::new(One::new(Self::load())))
    }
}

#[typetag::serde]
impl crate::Node for GlobalNode {
    fn name(&self) -> &'static str {
        "global"
    }
}
