use crate::{FromAnyProto, InputComponent, OneOrMany, PossibleInputs};
use std::any::Any;

#[derive(Debug, Clone, InputComponent, FromAnyProto)]
pub enum ArithmeticNodeInput {
    F32F32((OneOrMany<f32>, OneOrMany<f32>)),
    F32U32((OneOrMany<f32>, OneOrMany<u32>)),
    U32F32((OneOrMany<u32>, OneOrMany<f32>)),
    U32U32((OneOrMany<u32>, OneOrMany<u32>)),
}

struct Pair<A, B> {
    lhs: OneOrMany<A>,
    rhs: OneOrMany<B>,
}

impl<A, B> From<(OneOrMany<A>, OneOrMany<B>)> for Pair<A, B> {
    fn from((lhs, rhs): (OneOrMany<A>, OneOrMany<B>)) -> Self {
        Pair { lhs, rhs }
    }
}

impl<A, B> Pair<A, B>
where
    A: Clone + std::fmt::Debug + 'static,
    B: Clone + std::fmt::Debug + 'static,
{
    fn op<O, F: Fn(A, B) -> O>(self, op: F) -> OneOrMany<O>
    where
        O: Clone + std::fmt::Debug + 'static,
        F: Fn(A, B) -> O + 'static + Clone,
    {
        crate::one_many::op2(self.lhs, self.rhs, op)
    }

    fn opf<O, F: Fn(A, B) -> O>(self, op: F) -> Box<dyn Any>
    where
        O: Clone + std::fmt::Debug + 'static,
        F: Fn(A, B) -> O + 'static + Clone,
    {
        self.op(op).into_boxed_inner()
    }

    fn op_left<O, F: Fn(A) -> O>(self, op: F) -> Pair<O, B>
    where
        O: Clone + std::fmt::Debug + 'static,
        F: Fn(A) -> O + 'static + Clone,
    {
        let lhs = crate::one_many::op1(self.lhs, op);
        Pair { lhs, rhs: self.rhs }
    }

    fn op_right<O, F: Fn(B) -> O>(self, op: F) -> Pair<A, O>
    where
        O: Clone + std::fmt::Debug + 'static,
        F: Fn(B) -> O + 'static + Clone,
    {
        let rhs = crate::one_many::op1(self.rhs, op);
        Pair { lhs: self.lhs, rhs }
    }
}

impl ArithmeticNodeInput {
    pub fn mul(self) -> Box<dyn Any> {
        match self {
            ArithmeticNodeInput::F32F32(v) => Pair::from(v).opf(std::ops::Mul::mul),
            ArithmeticNodeInput::U32U32(v) => Pair::from(v).opf(std::ops::Mul::mul),
            ArithmeticNodeInput::F32U32(v) => {
                Pair::from(v).op_right(|v| v as f32).opf(std::ops::Mul::mul)
            }
            ArithmeticNodeInput::U32F32(v) => {
                Pair::from(v).op_left(|v| v as f32).opf(std::ops::Mul::mul)
            }
        }
    }

    pub fn div(self) -> Box<dyn Any> {
        match self {
            ArithmeticNodeInput::F32F32(v) => Pair::from(v).opf(std::ops::Div::div),
            ArithmeticNodeInput::U32U32(v) => {
                Pair::from(v).opf(|lhs, rhs| lhs.checked_div(rhs).unwrap_or(0))
            }
            ArithmeticNodeInput::F32U32(v) => {
                Pair::from(v).op_right(|v| v as f32).opf(std::ops::Div::div)
            }
            ArithmeticNodeInput::U32F32(v) => {
                Pair::from(v).op_left(|v| v as f32).opf(std::ops::Div::div)
            }
        }
    }

    pub fn types() -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static CACHE: Lazy<PossibleInputs> =
            Lazy::new(|| ArithmeticNodeInput::possible_inputs(&["lhs", "rhs"]));
        PossibleInputs::new(&*CACHE.groups)
    }
}
