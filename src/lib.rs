#![feature(const_type_id)]
#![allow(unused)]

use serde::{Deserialize, Serialize};
use slotmap::SlotMap;
use std::any::{Any, TypeId};

#[derive(Clone, Debug)]
pub enum Primitive {
    Float(f32),
    Integer(i32),
    UnsignedInteger(u32),
    Color(Color),
    Command(Command),
}

impl Primitive {
    pub fn type_id(&self) -> TypeId {
        match self {
            Primitive::Float(_) => TypeId::of::<f32>(),
            Primitive::Integer(_) => TypeId::of::<i32>(),
            Primitive::UnsignedInteger(_) => TypeId::of::<u32>(),
            Primitive::Color(_) => TypeId::of::<Color>(),
            Primitive::Command(_) => TypeId::of::<Command>(),
        }
    }

    pub fn variant_eq(&self, other: &dyn Any) -> bool {
        self.type_id() == other.type_id()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Input {
    name: &'static str,
    ty: TypeId,
}

pub enum SetInputError {
    InvalidIndex,
    TypeMismatch,
}

pub trait NodeInput {
    fn inputs(&self) -> &[Input];
    fn set_input(&mut self, index: usize, v: &dyn Any) -> Result<(), SetInputError>;
}

pub trait NodeOutput {
    fn output_type(&self) -> TypeId;
    fn output(&self) -> Box<dyn Any>;
    // fn output(&self) -> Primitive;
}

// index could be restricted to an InputID type
#[typetag::serde(tag = "type")]
pub trait Node:
    Send + Sync + std::fmt::Debug + dyn_clone::DynClone + NodeInput + NodeOutput
{
}
dyn_clone::clone_trait_object!(Node);

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct Time;

const TIME_INTERNAL: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
impl Time {
    pub fn tick() {
        TIME_INTERNAL.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn frame(&self) -> u32 {
        TIME_INTERNAL.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl NodeInput for Time {
    fn inputs(&self) -> &[Input] {
        &[]
    }

    fn set_input(&mut self, index: usize, v: &dyn Any) -> Result<(), SetInputError> {
        Err(SetInputError::InvalidIndex)
    }
}

impl NodeOutput for Time {
    fn output_type(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn output(&self) -> Box<dyn Any> {
        Box::new(*self)
    }
}

#[typetag::serde]
impl Node for Time {}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct Ratio {
    length: u32,
    time: Time,
}

impl NodeInput for Ratio {
    fn inputs(&self) -> &[Input] {
        const INPUTS: &'static [Input] = &[
            Input {
                name: "length",
                ty: TypeId::of::<u32>(),
            },
            Input {
                name: "time",
                ty: TypeId::of::<Time>(),
            },
        ];
        INPUTS
    }

    fn set_input(&mut self, index: usize, v: &dyn Any) -> Result<(), SetInputError> {
        match index {
            0 => match v.downcast_ref::<u32>() {
                Some(length) => {
                    self.length = length.clone();
                    Ok(())
                }
                None => Err(SetInputError::TypeMismatch),
            },
            1 => match v.downcast_ref::<Time>() {
                Some(time) => {
                    self.time = time.clone();
                    Ok(())
                }
                None => Err(SetInputError::TypeMismatch),
            },
            _ => Err(SetInputError::InvalidIndex),
        }
    }
}

impl NodeOutput for Ratio {
    fn output_type(&self) -> TypeId {
        TypeId::of::<f32>()
    }

    fn output(&self) -> Box<dyn Any> {
        let frame = self.time.frame();
        let ratio = (frame % self.length) as f32 / self.length as f32;
        Box::new(ratio)
    }
}

#[typetag::serde]
impl Node for Ratio {}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct Color {
    pub red: f32,
    pub blue: f32,
    pub green: f32,
    pub alpha: f32,
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct Clear {
    pub color: Color,
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct Rectangle {
    pub color: Color,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Command {
    Clear(Clear),
    Rectangle(Rectangle),
}

// impl Index<usize> for Command {
//     type Output = dyn Any;
//
//     fn index(&self, index: usize) -> &Self::Output {
//         match self {
//             Command::Clear(clear) => clear.index(index),
//             Command::Rectangle(rectangle) => rectangle.index(index),
//         }
//     }
// }
//
// impl IndexMut<usize> for Command {
//     fn index_mut(&mut self, index: usize) -> &mut Self::Output {
//         match self {
//             Command::Clear(clear) => clear.index_mut(index),
//             Command::Rectangle(rectangle) => rectangle.index_mut(index),
//         }
//     }
// }

// impl NodeInput for Command {
//     fn inputs(&self) -> &'static [&'static str] {
//         match self {
//             Command::Clear(clear) => clear.inputs(),
//             Command::Rectangle(rectangle) => rectangle.inputs(),
//         }
//     }
// }
//
// impl NodeOutput for Command {
//     fn output(&self) -> Primitive {
//         Primitive::Command(*self)
//     }
// }

// #[typetag::serde]
// impl Node for Command {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screen {
    #[serde(skip)]
    inputs: Vec<Input>,
    commands: Vec<Option<Command>>,
}

impl Screen {
    pub fn new() -> Self {
        Self {
            inputs: vec![],
            commands: vec![],
        }
    }
}

impl NodeInput for Screen {
    fn inputs(&self) -> &[Input] {
        self.inputs.as_slice()
    }

    fn set_input(&mut self, index: usize, v: &dyn Any) -> Result<(), SetInputError> {
        match v.downcast_ref::<Command>() {
            None => Err(SetInputError::TypeMismatch),
            Some(command) => {
                if self.commands.len() <= index {
                    self.commands.resize(index + 1, None);
                    self.inputs.resize(
                        index + 1,
                        Input {
                            name: "command",
                            ty: TypeId::of::<Command>(),
                        },
                    )
                }
                self.commands.insert(index, Some(command.clone()));
                Ok(())
            }
        }
    }
}

impl NodeOutput for Screen {
    fn output_type(&self) -> TypeId {
        TypeId::of::<()>()
    }

    fn output(&self) -> Box<dyn Any> {
        Box::new(())
    }
}

#[typetag::serde]
impl Node for Screen {}

slotmap::new_key_type! {
    pub struct NodeID;
    pub struct InputID;
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connection {
    from: NodeID,
    to: NodeID,
    input: usize,
}

impl Connection {
    pub fn connects(&self, node: NodeID) -> bool {
        self.from == node || self.to == node
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Graph {
    root: NodeID,
    nodes: SlotMap<NodeID, Box<dyn Node>>,
    connections: Vec<Connection>,
}

impl Graph {
    pub fn new() -> Self {
        let mut nodes = SlotMap::<NodeID, Box<dyn Node>>::with_key();
        let root = nodes.insert(Box::new(Screen::new()));
        Self {
            nodes,
            root,
            connections: Vec::new(),
        }
    }

    pub fn root(&self) -> NodeID {
        self.root
    }

    pub fn add_node<N>(&mut self, node: N) -> NodeID
    where
        N: Node + 'static,
    {
        self.nodes.insert(Box::new(node))
    }

    pub fn remove_node(&mut self, node_id: NodeID) -> Option<Box<dyn Node>> {
        if self.nodes.contains_key(node_id) {
            self.connections
                .retain(|connection| !connection.connects(node_id))
        }
        self.nodes.remove(node_id)
    }

    pub fn connect(
        &mut self,
        output_node_id: NodeID,
        input_node_id: NodeID,
        input_index: usize,
    ) -> Option<Connection> {
        let output_ty = self
            .nodes
            .get(output_node_id)
            .map(|output| output.output_type());
        let input_node = self.nodes.get_mut(input_node_id);
        let inputs = input_node.as_ref().map(|input_node| input_node.inputs());
        let input = inputs.and_then(|inputs| inputs.get(input_index));
        if let (Some(input), Some(output_ty)) = (input, output_ty) {
            if input.ty == output_ty {
                let connection = Connection {
                    from: output_node_id,
                    to: input_node_id,
                    input: input_index,
                };
                self.connections.push(connection);
                Some(connection)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn disconnect(&mut self, connection: Connection) {
        self.connections
            .iter()
            .position(|other| *other == connection)
            .map(|position| self.connections.swap_remove(position));
    }

    // /// breadth-first toward root
    // pub fn iter(&self) -> GraphIter {
    //     let current = self.root();
    //
    // }
}

// pub struct GraphIter<'a> {
//     connections: &'a [Connection],
//     nodes: &'a [Box<Node>]
// }
//
// impl<'a> Iterator for GraphIter<'a> {
//     type Item = &'a dyn Node;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         unimplemented!()
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::{Map, Once};
    use std::ops::{Mul, Range, Rem};

    trait Many<T>: Iterator<Item = T> + dyn_clone::DynClone + Any {}
    dyn_clone::clone_trait_object!(<T> Many<T>);
    impl<I, T> Many<T> for I where I: Iterator<Item = T> + Clone + Any {}

    #[test]
    fn many_one() {
        #[derive(Debug, Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
        struct One<T>(T);

        trait ManyTrait<T>: Iterator<Item = T> + dyn_clone::DynClone + std::fmt::Debug {}
        dyn_clone::clone_trait_object!(<T> ManyTrait<T>);
        impl<I, T> ManyTrait<T> for I where I: Iterator<Item = T> + Clone + std::fmt::Debug {}

        // If it turns out that there are only so many different types of iterator then this could
        // be replaced internally with an enum and the From implementation restricted
        #[derive(Debug, Clone)]
        struct Many<T>(Box<dyn ManyTrait<T>>);
        impl<T> Default for Many<T>
        where
            T: 'static,
        {
            fn default() -> Self {
                Self(Box::new(std::iter::empty()))
            }
        }
        impl<I, T> From<I> for Many<T>
        where
            I: IntoIterator<Item = T> + 'static,
            <I as IntoIterator>::IntoIter: Clone + std::fmt::Debug,
        {
            fn from(iter: I) -> Self {
                Many(Box::new(iter.into_iter()))
            }
        }
        impl<T> Many<T> {
            pub fn collect<B>(self) -> B
            where
                B: std::iter::FromIterator<T>,
            {
                self.0.collect()
            }
        }

        impl<T> std::ops::Mul for One<T>
        where
            T: std::ops::Mul<Output = T>,
        {
            type Output = One<T>;

            fn mul(self, rhs: Self) -> Self::Output {
                One(self.0 * rhs.0)
            }
        }
        impl<T> std::ops::Mul<Many<T>> for One<T>
        where
            T: std::ops::Mul<Output = T> + Copy + 'static,
        {
            type Output = Many<T>;

            fn mul(self, rhs: Many<T>) -> Self::Output {
                let lhs = self.0;
                Many(Box::new(rhs.0.map(move |rhs| lhs * rhs)))
            }
        }
        impl<T> std::ops::Mul for Many<T>
        where
            T: std::ops::Mul<Output = T> + 'static,
        {
            type Output = Many<T>;

            fn mul(self, rhs: Self) -> Self::Output {
                Many(Box::new(self.0.zip(rhs.0).map(|(lhs, rhs)| lhs * rhs)))
            }
        }

        #[derive(Debug, Clone, Default)]
        struct Pair<A, B> {
            lhs: std::marker::PhantomData<A>,
            rhs: std::marker::PhantomData<B>,
        }

        #[derive(Debug, Clone, Default)]
        struct MultiplyGroup<T>
        where
            T: 'static,
        {
            one_one: Pair<One<T>, One<T>>,
            one_many: Pair<One<T>, Many<T>>,
            many_many: Pair<Many<T>, Many<T>>,
        }

        #[derive(Debug, Clone, Default)]
        struct MultiplyNode {
            f32: MultiplyGroup<f32>,
            u32: MultiplyGroup<u32>,
        }

        impl<A, B> NodeInputProto for Pair<A, B>
        where
            A: 'static,
            B: 'static,
        {
            fn inputs_match(inputs: &[Box<dyn Any>]) -> bool {
                if inputs.len() == 2 {
                    inputs[0].is::<A>() && inputs[1].is::<B>()
                } else {
                    false
                }
            }
        }

        impl<T> NodeInputProto for MultiplyGroup<T>
        where
            T: 'static,
        {
            fn inputs_match(inputs: &[Box<dyn Any>]) -> bool {
                <Pair<One<T>, One<T>> as NodeInputProto>::inputs_match(inputs)
                    || <Pair<One<T>, Many<T>> as NodeInputProto>::inputs_match(inputs)
                    || <Pair<Many<T>, Many<T>> as NodeInputProto>::inputs_match(inputs)
            }
        }

        impl<T> NodeOutputProto for MultiplyGroup<T>
        where
            T: std::ops::Mul<Output = T> + 'static + Copy,
        {
            fn op(self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
                if <Pair<One<T>, One<T>> as NodeInputProto>::inputs_match(inputs) {
                    let rhs = inputs.remove(1).downcast::<One<T>>().unwrap();
                    let lhs = inputs.remove(0).downcast::<One<T>>().unwrap();
                    Ok(Box::new(*lhs * *rhs))
                } else if <Pair<One<T>, Many<T>> as NodeInputProto>::inputs_match(inputs) {
                    let rhs = inputs.remove(1).downcast::<Many<T>>().unwrap();
                    let lhs = inputs.remove(0).downcast::<One<T>>().unwrap();
                    Ok(Box::new(*lhs * *rhs))
                } else if <Pair<Many<T>, Many<T>> as NodeInputProto>::inputs_match(inputs) {
                    let rhs = inputs.remove(1).downcast::<Many<T>>().unwrap();
                    let lhs = inputs.remove(0).downcast::<Many<T>>().unwrap();
                    Ok(Box::new(*lhs * *rhs))
                } else {
                    Err(())
                }
            }
        }

        impl NodeInputProto for MultiplyNode {
            fn inputs_match(inputs: &[Box<dyn Any>]) -> bool {
                <MultiplyGroup<f32> as NodeInputProto>::inputs_match(inputs)
                    || <MultiplyGroup<u32> as NodeInputProto>::inputs_match(inputs)
            }
        }

        impl NodeOutputProto for MultiplyNode {
            fn op(self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
                MultiplyGroup::<f32>::default()
                    .op(inputs)
                    .or_else(|_e| MultiplyGroup::<u32>::default().op(inputs))
            }
        }

        #[derive(Debug, Clone, Default)]
        struct RatioGroup<T>
        where
            T: 'static,
        {
            one_one: Pair<One<T>, One<T>>,
            one_many: Pair<One<T>, Many<T>>,
        }

        #[derive(Debug, Clone, Default)]
        struct RatioNode {
            f32: RatioGroup<f32>,
            u32: RatioGroup<u32>,
        }

        impl<T> NodeInputProto for RatioGroup<T>
        where
            T: 'static,
        {
            fn inputs_match(inputs: &[Box<dyn Any>]) -> bool {
                <Pair<One<T>, One<T>> as NodeInputProto>::inputs_match(inputs)
                    || <Pair<One<T>, Many<T>> as NodeInputProto>::inputs_match(inputs)
            }
        }

        impl<T> NodeOutputProto for RatioGroup<T>
        where
            T: std::ops::Rem<Output = T> + Into<f64> + Copy + 'static,
        {
            fn op(self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
                fn ratio<T>(length: T, count: T) -> f32
                where
                    T: std::ops::Rem<Output = T> + Into<f64> + Copy,
                {
                    (Into::<f64>::into(count.rem(length)) / length.into()) as f32
                }

                if <Pair<One<T>, One<T>> as NodeInputProto>::inputs_match(inputs) {
                    let count = inputs.remove(1).downcast::<One<T>>().unwrap();
                    let length = inputs.remove(0).downcast::<One<T>>().unwrap();
                    Ok(Box::new(One(ratio(length.0, count.0))))
                } else if <Pair<One<T>, Many<T>> as NodeInputProto>::inputs_match(inputs) {
                    let count = inputs.remove(1).downcast::<Many<T>>().unwrap();
                    let length = inputs.remove(0).downcast::<One<T>>().unwrap();
                    let out = count.0.map(move |count| ratio(length.0, count));
                    Ok(Box::new(Many(Box::new(out))))
                } else {
                    Err(())
                }
            }
        }

        impl NodeInputProto for RatioNode {
            fn inputs_match(inputs: &[Box<dyn Any>]) -> bool {
                RatioGroup::<f32>::inputs_match(inputs) || RatioGroup::<u32>::inputs_match(inputs)
            }
        }

        impl NodeOutputProto for RatioNode {
            fn op(self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
                RatioGroup::<f32>::default()
                    .op(inputs)
                    .or_else(|_e| RatioGroup::<u32>::default().op(inputs))
            }
        }

        #[derive(Debug, Clone, Default)]
        struct RangeNode {
            u32: std::marker::PhantomData<One<u32>>,
        }

        impl NodeInputProto for RangeNode {
            fn inputs_match(inputs: &[Box<dyn Any>]) -> bool {
                if inputs.len() == 1 {
                    inputs[0].is::<One<u32>>()
                } else {
                    false
                }
            }
        }

        impl NodeOutputProto for RangeNode {
            fn op(self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
                if Self::inputs_match(&inputs) {
                    let length = inputs.remove(0);
                    let length = length.downcast::<One<u32>>().unwrap();
                    Ok(Box::new(Into::<Many<u32>>::into(0u32..(length.0))))
                } else {
                    Err(())
                }
            }
        }

        #[derive(Debug, Clone)]
        enum ConstantNode {
            Unsigned(u32),
            Float(f32),
        }

        impl NodeInputProto for ConstantNode {
            fn inputs_match(inputs: &[Box<dyn Any>]) -> bool {
                false
            }

            fn is_terminator() -> bool {
                true
            }
        }

        impl NodeOutputProto for ConstantNode {
            fn op(self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
                Ok(match self {
                    ConstantNode::Unsigned(output) => Box::new(One(output)),
                    ConstantNode::Float(output) => Box::new(One(output)),
                })
            }
        }

        trait NodeInputProto {
            fn inputs_match(inputs: &[Box<dyn Any>]) -> bool;
            fn is_terminator() -> bool {
                false
            }
        }

        trait NodeOutputProto {
            fn op(self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()>;
        }

        trait NodeProto {}
        impl<T> NodeProto for T where T: NodeInputProto + NodeOutputProto {}

        {
            let a = One(2u32);
            let b = One(4u32);
            let c = a * b;
            assert_eq!(One(8u32), c);

            let d: Many<u32> = vec![2u32, 3, 4].into();
            let e = One(2u32) * d;
            assert_eq!(vec![4u32, 6, 8], e.clone().collect::<Vec<_>>());

            let f = e * Into::<Many<u32>>::into(vec![4u32, 3, 2]);
            assert_eq!(vec![16u32, 18, 16], f.collect::<Vec<_>>());
        }

        {
            let mut buffer: Vec<Box<dyn Any>> = Vec::new();
            buffer.push(Box::new(One(2u32)));
            buffer.push(Box::new(Into::<Many<u32>>::into(vec![2u32, 3, 4])));
            assert!(<Pair<One<u32>, Many<u32>>>::inputs_match(&buffer));
            assert!(MultiplyGroup::<u32>::inputs_match(&buffer));
            assert!(MultiplyNode::inputs_match(&buffer));

            let output = MultiplyNode::default().op(&mut buffer).unwrap();
            assert!(buffer.is_empty());
            buffer.push(output);
            buffer.push(Box::new(Into::<Many<u32>>::into(vec![3u32, 4, 5])));
            let output = MultiplyNode::default().op(&mut buffer).unwrap();

            let output = output.downcast::<Many<u32>>().unwrap();
            assert_eq!(vec![12u32, 24, 40], output.collect::<Vec<_>>());
        }

        {
            let mut buffer: Vec<Box<dyn Any>> = Vec::new();
            buffer.push(Box::new(One(3u32)));
            buffer.push(Box::new(Into::<Many<u32>>::into(0u32..3)));
            assert!(RatioNode::inputs_match(&buffer));

            let output = RatioNode::default().op(&mut buffer).unwrap();
            assert!(buffer.is_empty());

            let output = output.downcast::<Many<f32>>().unwrap();
            assert_eq!(vec![0f32, 1. / 3., 2. / 3.], output.collect::<Vec<_>>());
        }

        {
            let mut buffer: Vec<Box<dyn Any>> = vec![];

            let mut nodes: Vec<Box<dyn NodeProto>> = vec![];
            let constant = ConstantNode::Unsigned(3);
            let range = RangeNode::default();
            let ratio = RatioNode::default();
            let multiply = MultiplyNode::default();

            let constant_output = constant.clone().op(&mut buffer).unwrap();

            buffer.push(constant_output);
            let range_output = range.op(&mut buffer).unwrap();

            let constant_output = constant.clone().op(&mut buffer).unwrap();

            buffer.push(constant_output);
            buffer.push(range_output);
            let ratio_output = ratio.op(&mut buffer).unwrap();

            let constant_output = ConstantNode::Float(2.).op(&mut buffer).unwrap();

            buffer.push(constant_output);
            buffer.push(ratio_output);
            let multiply_output = multiply.op(&mut buffer).unwrap();

            let output = multiply_output.downcast::<Many<f32>>().unwrap();
            assert_eq!(
                vec![0f32, 1. / 3. * 2., 2. / 3. * 2.],
                output.collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn maybe_iter_trait() {
        trait Mopa: Any {}
        // trait Mopa: Any {
        //     fn into_iter<T>(self: Box<Self>) -> Result<Box<dyn Iterator<Item = T>>, Box<dyn Mopa>>;
        // }

        fn into_iter<I, T>(b: Box<Many<I, T>>) -> Result<Box<dyn Iterator<Item = T>>, Box<dyn Mopa>>
        where
            I: Iterator<Item = T> + 'static,
            T: 'static,
        {
            let raw: *mut dyn Iterator<Item = T> = Box::into_raw(b);
            Ok(unsafe { Box::from_raw(raw as *mut dyn Iterator<Item = T>) })
        }

        // struct One<T>(T);
        // struct Many<T>(Box<dyn Iterator<Item = T>>);
        struct Many<I, T>(I)
        where
            I: Iterator<Item = T>;

        impl<I, T> Iterator for Many<I, T>
        where
            I: Iterator<Item = T>,
        {
            type Item = T;

            fn next(&mut self) -> Option<Self::Item> {
                self.0.next()
            }
        }
        impl<I, T> Mopa for Many<I, T>
        where
            I: Iterator<Item = T> + Any,
            T: Any,
        {
            //     fn into_iter(self: Box<Self>) -> Result<Box<dyn Iterator<Item = T>>, Box<dyn Mopa>> {
            //         let raw: *mut dyn Iterator<Item = T> = Box::into_raw(self);
            //         Ok(unsafe { Box::from_raw(raw as *mut dyn Iterator<Item = T>) })
            //     }
        }

        // impl<I, T> Many<I, T> where I: Iterator<Item = T> + 'static, T: 'static {
        //     fn cast(self: Box<Self>) -> Box<dyn Mopa<Iter = I>> {
        //         let raw: *mut dyn Mopa<Item = T> = Box::into_raw(self);
        //         unsafe { Box::from_raw(raw as *mut dyn Mopa<Item = T>) }
        //     }
        // }

        // impl<I, V> Mopa for Many<I, V>
        // where
        //     I: Iterator<Item = V> + 'static,
        //     V: 'static,
        // {
        //     fn into_many<T>(
        //         self: Box<Self, Global>,
        //     ) -> Result<Box<dyn Iterator<Item = T>, Global>, Box<Self, Global>>
        //     {
        //         unimplemented!()
        //     }
        // }

        // fn downcast<T>(mopa: Box<dyn Mopa>) -> Box<dyn Iterator<Item = T>> {
        //     unsafe {
        //         let raw: *mut dyn Mopa = Box::into_raw(mopa);
        //         Box::from_raw(raw as *mut dyn Iterator<Item = T>)
        //     }
        // }

        let m: Box<dyn Mopa> = Box::new(Many(std::iter::once(1)));
        // let v = into_iter(m.downcast().unwrap());
        // let m: Box<Many<_, i32>> = m.downcast().unwrap();
        // let r = m.cast();
        // for v in r {
        //     println!("{:?}", v);
        // }

        // impl<I, T> Mopa for Many<I, T> where I: Any + Iterator<Item = T>, T: Any {
        //     fn into_many<V>(self) -> Result<Box<dyn Iterator<Item=V>>, Self> {
        //         unimplemented!()
        //     }
        // }

        // impl<I, T> Mopa<T> for Box<Many<I, T>> where I: Iterator<Item = T> + Any, T: Any {
        //     fn into_one(self) -> Result<T, Self> {
        //         unimplemented!()
        //     }
        //
        //     fn into_many(self) -> Result<Box<dyn Iterator<Item=T>>, Self> {
        //         unimplemented!()
        //     }
        // }

        // impl<T> Mopa for One<T> {
        //     fn into_one<T>(self) -> Result<T, Self> {
        //         Ok(self.0)
        //     }
        //
        //     fn into_many<T>(self: Box<Self, Global>) -> Result<Box<_, Global>, Box<Self, Global>> {
        //         Err(self)
        //     }
        // }
        //
        // impl<T> Mopa for Many<T> where T: Iterator<Item = T> {
        //     fn into_one<T>(self) -> Result<T, Self> {
        //         Err(self)
        //     }
        //
        //     fn into_many<T>(self: Box<Self, Global>) -> Result<Box<_, Global>, Box<Self, Global>> {
        //         Ok(self.0)
        //     }
        // }
    }

    #[test]
    fn convalesce() {
        struct Multiply<A, B> {
            lhs: std::marker::PhantomData<A>,
            rhs: std::marker::PhantomData<B>,
        }

        // type MulOneToMany<T> = Multiply<T, Box<dyn Many<T>>>;
        // type MulManyToMany<T> = Multiply<Box<dyn Many<T>>, Box<dyn Many<T>>>;

        // fn mul_one_to_many<'a, T>(
        //     lhs: &'a T,
        //     rhs: impl Iterator<Item = &'a T> + 'a,
        // ) -> impl Iterator<Item = T> + 'a
        //     where
        //         &'a T: std::ops::Mul<&'a T, Output = T>,
        // {
        //     rhs.map(move |rhs| lhs * rhs)
        // }

        // fn mul_one_to_many<'a, T>(
        //     lhs: &'a T,
        //     rhs: impl Iterator<Item = T> + 'a,
        // ) -> impl Iterator<Item = T> + 'a
        //     where
        //         &'a T: std::ops::Mul<T, Output = T>,
        // {
        //     rhs.map(move |rhs| lhs * rhs)
        // }

        fn mul_one_to_many<T>(lhs: T, rhs: impl Iterator<Item = T>) -> impl Iterator<Item = T>
        where
            T: std::ops::Mul<Output = T> + Copy,
        {
            rhs.map(move |rhs| lhs * rhs)
        }

        // fn mul_one_to_many<T>(lhs: T, rhs: impl Many<T> + Clone) -> impl Many<T>
        //     where
        //         T: std::ops::Mul<Output = T> + Copy + 'static,
        // {
        //     rhs.map(move |rhs| lhs * rhs)
        // }

        let i = mul_one_to_many(2, std::iter::once(2)).collect::<Vec<_>>();
        assert_eq!(i, vec![4]);

        // let i = mul_one_to_many(&2, vec![2, 3, 4].into_iter());
        // let i = mul_one_to_many(&2, i).collect::<Vec<_>>();
        // assert_eq!(i, vec![4 * 2, 6 * 2, 8 * 2]);

        impl<T> Multiply<T, T>
        where
            T: Any,
            T: std::ops::Mul<T, Output = T> + Copy,
        {
            fn inputs_match(inputs: &[Box<dyn Any>]) -> bool {
                if inputs.len() != 2 {
                    return false;
                }
                inputs[0].is::<T>() && inputs[1].is::<T>()
                    || inputs[0].is::<T>() && inputs[1].is::<Box<dyn Many<T>>>()
            }

            fn op(inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
                if !Self::inputs_match(&inputs) {
                    Err(())
                } else {
                    let rhs = inputs.remove(1).downcast::<Box<dyn Many<T>>>().unwrap();
                    let lhs = inputs.remove(0).downcast::<T>().unwrap();

                    let o: Box<Box<dyn Many<_>>> =
                        Box::new(Box::new(rhs.map(move |rhs| *lhs * rhs)));
                    Ok(o)

                    // let rhs = inputs.remove(1).downcast::<T>();
                    // let lhs = inputs.remove(0).downcast::<T>();
                    //
                    // if let (Ok(lhs), Ok(rhs)) = (lhs, rhs) {
                    //     Ok(Box::new(*lhs * *rhs))
                    // } else {
                    //     Err(())
                    // }
                }
            }

            // fn op(&self, inputs: &'a [Box<dyn Any>]) -> Result<Box<dyn Any + 'a>, ()> {
            //     if !self.inputs_match(inputs) {
            //         return Err(());
            //     }
            //     if let (Some(lhs), Some(rhs)) =
            //         (inputs[0].downcast_ref::<T>(), inputs[1].downcast_ref::<T>())
            //     {
            //         Ok(Box::new(lhs * rhs))
            //     } else if let (Some(lhs), Some(rhs)) = (
            //         inputs[0].downcast_ref::<T>(),
            //         inputs[1].downcast_ref::<Box<dyn Many<&T>>>(),
            //     ) {
            //         Ok(Box::new(rhs.clone().map(move |rhs| lhs * rhs)))
            //     } else if let (Some(lhs), Some(rhs)) = (
            //         inputs[0].downcast_ref::<Box<dyn Many<&T>>>(),
            //         inputs[1].downcast_ref::<Box<dyn Many<&T>>>(),
            //     ) {
            //         Ok(Box::new(
            //             lhs.clone().zip(rhs.clone()).map(|(lhs, rhs)| lhs * rhs),
            //         ))
            //     } else {
            //         Err(())
            //     }
            // }
        }

        struct MultiplyNode {
            i32_i32: Multiply<i32, i32>,
            u32_u32: Multiply<u32, u32>,
            f32_f32: Multiply<f32, f32>,
        }

        assert!(Multiply::<i32, i32>::inputs_match(&[
            Box::new(4),
            Box::new(2129)
        ]));
        let i: Box<Box<dyn Many<_>>> = Box::new(Box::new(std::iter::once(123)));
        assert!(Multiply::<i32, i32>::inputs_match(&[
            Box::new(4),
            i.clone()
        ]));
        let mut inputs: Vec<Box<dyn Any>> = vec![Box::new(4), i];
        let output = Multiply::<i32, i32>::op(&mut inputs);
        assert!(output.is_ok());
        let mut inputs: Vec<Box<dyn Any>> = vec![Box::new(2), output.unwrap()];
        let output = Multiply::<i32, i32>::op(&mut inputs);
        assert!(output.is_ok());
    }

    #[test]
    fn ty_test() {
        fn frame() -> u32 {
            0
        }

        fn ratio<T>(length: T, count: T) -> f32
        where
            T: std::ops::Rem<Output = T> + Into<f64> + Copy,
        {
            (Into::<f64>::into(count.rem(length)) / length.into()) as f32
        }
        fn count(count: u32) -> Range<u32> {
            0..count
        }

        pub struct CountRatio {
            length: u32,
            current: u32,
        }
        impl Iterator for CountRatio {
            type Item = f32;

            fn next(&mut self) -> Option<Self::Item> {
                if self.current >= self.length {
                    let r = Some(ratio(self.length, self.current));
                    self.current += 1;
                    r
                } else {
                    None
                }
            }
        }
        fn count_ratios(amount: u32) -> CountRatio {
            CountRatio {
                length: amount,
                current: 0,
            }
        }

        #[derive(Clone)]
        enum OneOrMany<T> {
            One(T),
            Many(Box<dyn Many<T>>),
        }

        #[derive(Clone)]
        struct Multiply<T> {
            lhs: OneOrMany<T>,
            rhs: OneOrMany<T>,
        }

        impl<T> Multiply<T>
        where
            T: std::ops::Mul<Output = T> + Copy + 'static,
        {
            fn op(self) -> OneOrMany<T> {
                use OneOrMany::{Many as M, One as O};
                match (self.lhs, self.rhs) {
                    (O(lhs), O(rhs)) => O(lhs * rhs),
                    (O(lhs), M(rhs)) => M(Box::new(rhs.map(move |rhs| lhs * rhs))),
                    (M(lhs), O(rhs)) => M(Box::new(lhs.map(move |lhs| lhs * rhs))),
                    (M(lhs), M(rhs)) => M(Box::new(lhs.zip(rhs).map(|(lhs, rhs)| lhs * rhs))),
                }
            }
        }

        #[derive(Clone)]
        enum MultiplyNode {
            UnsignedInt(Multiply<u32>),
        }

        struct Ratio<T> {
            total: T,
            iter: OneOrMany<T>,
        }

        impl Ratio<u32> {
            fn op(self) -> OneOrMany<f32> {
                let length = self.total;
                match self.iter {
                    OneOrMany::One(count) => OneOrMany::One(ratio(length, count)),
                    OneOrMany::Many(iter) => {
                        OneOrMany::Many(Box::new(iter.map(move |current| ratio(length, current))))
                    }
                }
            }
        }

        {
            const WIDTH: u32 = 5;
            const HEIGHT: u32 = 10;
            let x = count(WIDTH);
            let y = count(HEIGHT);
            let offset = Multiply {
                lhs: OneOrMany::One(HEIGHT),
                rhs: OneOrMany::Many(Box::new(x)),
            };
            let total = Multiply {
                lhs: OneOrMany::One(WIDTH),
                rhs: OneOrMany::One(HEIGHT),
            };
            // let r = Ratio {
            //     total: WIDTH * HEIGHT,
            //     iter: offset.op(),
            // };
        }

        // impl NodeOutput for MultiplyNode {
        //     fn output_type(&self) -> TypeId {
        //         match self {
        //             MultiplyNode::UnsignedInt(_) => TypeId::of::<u32>(),
        //         }
        //     }
        //
        //     fn output(&self) -> Box<dyn Any> {
        //         match self {
        //             MultiplyNode::UnsignedInt(multiply) => {}
        //         }
        //     }
        // }

        // let v = (0..5).map(|x| (0..10).map(move |y| y + x * 10).collect::<Vec<_>>()).collect::<Vec<_>>();
        // println!("{:?}", v);

        // impl NodeOutput for std::iter::Map<std::ops::Range<u32>, fn(u32) -> u32> {
        //     fn output_type(&self) -> TypeId {
        //         TypeId::of::<Self>()
        //     }
        //
        //     fn output(&self) -> Box<dyn Any> {
        //         Box::new(self.clone())
        //     }
        // }
    }

    #[test]
    fn it_works() {
        // let mut graph = Graph::new();
        // let root_node = graph.root();
        //
        // let clear_node = graph.add_node(Clear {
        //     color: Color {
        //         red: 1.0,
        //         blue: 0.0,
        //         green: 0.0,
        //         alpha: 1.0,
        //     },
        // });
        // let rectangle_node = graph.add_node(Rectangle {
        //     color: Color {
        //         red: 0.0,
        //         blue: 1.0,
        //         green: 0.0,
        //         alpha: 1.0,
        //     },
        //     x: 0.0,
        //     y: 0.0,
        //     width: 100.0,
        //     height: 100.0,
        // });
        //
        // let connection1 = graph.connect(clear_node, root_node, 0);
        // assert!(connection1.is_some());
        //
        // let connection2 = graph.connect(rectangle_node, root_node, 1);
        // assert!(connection2.is_some())
    }
}
