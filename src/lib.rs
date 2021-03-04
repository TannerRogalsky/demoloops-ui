#![feature(const_type_id)]

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

    fn set_input(&mut self, _index: usize, _v: &dyn Any) -> Result<(), SetInputError> {
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
    use super::{Any, Connection, NodeID, SlotMap};

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
            fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool {
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
            fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool {
                self.one_one.inputs_match(inputs)
                    || self.one_many.inputs_match(inputs)
                    || self.many_many.inputs_match(inputs)
            }
        }

        impl<T> NodeOutputProto for MultiplyGroup<T>
        where
            T: std::ops::Mul<Output = T> + 'static + Copy,
        {
            fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
                if self.one_one.inputs_match(inputs) {
                    let rhs = inputs.remove(1).downcast::<One<T>>().unwrap();
                    let lhs = inputs.remove(0).downcast::<One<T>>().unwrap();
                    Ok(Box::new(*lhs * *rhs))
                } else if self.one_many.inputs_match(inputs) {
                    let rhs = inputs.remove(1).downcast::<Many<T>>().unwrap();
                    let lhs = inputs.remove(0).downcast::<One<T>>().unwrap();
                    Ok(Box::new(*lhs * *rhs))
                } else if self.many_many.inputs_match(inputs) {
                    let rhs = inputs.remove(1).downcast::<Many<T>>().unwrap();
                    let lhs = inputs.remove(0).downcast::<Many<T>>().unwrap();
                    Ok(Box::new(*lhs * *rhs))
                } else {
                    Err(())
                }
            }
        }

        impl NodeInputProto for MultiplyNode {
            fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool {
                self.f32.inputs_match(inputs) || self.u32.inputs_match(inputs)
            }
        }

        impl NodeOutputProto for MultiplyNode {
            fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
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
            fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool {
                self.one_one.inputs_match(inputs) || self.one_many.inputs_match(inputs)
            }
        }

        impl<T> NodeOutputProto for RatioGroup<T>
        where
            T: std::ops::Rem<Output = T> + Into<f64> + Copy + 'static,
        {
            fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
                fn ratio<T>(length: T, count: T) -> f32
                where
                    T: std::ops::Rem<Output = T> + Into<f64> + Copy,
                {
                    (count.rem(length).into() / length.into()) as f32
                }

                if self.one_one.inputs_match(inputs) {
                    let count = inputs.remove(1).downcast::<One<T>>().unwrap();
                    let length = inputs.remove(0).downcast::<One<T>>().unwrap();
                    Ok(Box::new(One(ratio(length.0, count.0))))
                } else if self.one_many.inputs_match(inputs) {
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
            fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool {
                self.f32.inputs_match(inputs) || self.u32.inputs_match(inputs)
            }
        }

        impl NodeOutputProto for RatioNode {
            fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
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
            fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool {
                if inputs.len() == 1 {
                    inputs[0].is::<One<u32>>()
                } else {
                    false
                }
            }
        }

        impl NodeOutputProto for RangeNode {
            fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
                if self.inputs_match(&inputs) {
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
            fn inputs_match(&self, _inputs: &[Box<dyn Any>]) -> bool {
                false
            }

            fn is_terminator(&self) -> bool {
                true
            }
        }

        impl NodeOutputProto for ConstantNode {
            fn op(&self, _inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
                Ok(match self {
                    ConstantNode::Unsigned(output) => Box::new(One(*output)),
                    ConstantNode::Float(output) => Box::new(One(*output)),
                })
            }
        }

        trait NodeInputProto {
            fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool;
            fn is_terminator(&self) -> bool {
                false
            }
        }

        trait NodeOutputProto {
            fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()>;
        }

        trait NodeProto: NodeInputProto + NodeOutputProto {}
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
            assert!(<Pair<One<u32>, Many<u32>>>::default().inputs_match(&buffer));
            assert!(MultiplyGroup::<u32>::default().inputs_match(&buffer));
            assert!(MultiplyNode::default().inputs_match(&buffer));

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
            assert!(RatioNode::default().inputs_match(&buffer));

            let output = RatioNode::default().op(&mut buffer).unwrap();
            assert!(buffer.is_empty());

            let output = output.downcast::<Many<f32>>().unwrap();
            assert_eq!(vec![0f32, 1. / 3., 2. / 3.], output.collect::<Vec<_>>());
        }

        {
            let mut buffer: Vec<Box<dyn Any>> = vec![];

            let constant = ConstantNode::Unsigned(3);
            let range = RangeNode::default();
            let ratio = RatioNode::default();
            let multiply = MultiplyNode::default();

            let constant_output = constant.op(&mut buffer).unwrap();

            buffer.push(constant_output);
            let range_output = range.op(&mut buffer).unwrap();

            let constant_output = constant.op(&mut buffer).unwrap();

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

        struct Graph {
            root: NodeID,
            nodes: SlotMap<NodeID, Box<dyn NodeProto>>,
            connections: Vec<Connection>,
        }

        impl Graph {
            fn with_root<N>(root: N) -> Self
            where
                N: NodeProto + 'static,
            {
                let mut nodes: SlotMap<NodeID, Box<dyn NodeProto>> = SlotMap::with_key();
                let root = nodes.insert(Box::new(root));
                Self {
                    root,
                    nodes,
                    connections: vec![],
                }
            }

            fn add_node<N>(&mut self, node: N) -> NodeID
            where
                N: NodeProto + 'static,
            {
                self.nodes.insert(Box::new(node))
            }

            fn connect(&mut self, from: NodeID, to: NodeID, input: usize) {
                self.connections.push(Connection { from, to, input })
            }

            fn execute(&self) -> Result<Box<dyn Any>, ()> {
                self.execute_node(self.root)
            }

            fn execute_node(&self, node_id: NodeID) -> Result<Box<dyn Any>, ()> {
                let to = self.nodes.get(node_id).unwrap();
                let mut connections = self
                    .connections
                    .iter()
                    .filter(|connection| connection.connects(node_id))
                    .collect::<Vec<_>>();
                connections.sort_unstable_by(|a, b| a.input.cmp(&b.input));
                let mut input = connections
                    .into_iter()
                    .map(|connection| {
                        let input = self.nodes.get(connection.from).unwrap();
                        if input.is_terminator() {
                            let mut buf = vec![];
                            input.op(&mut buf)
                        } else {
                            self.execute_node(connection.from)
                        }
                    })
                    .collect::<Result<Vec<Box<dyn Any>>, ()>>()
                    .unwrap();
                to.op(&mut input)
            }
        }

        let mut graph = Graph::with_root(RangeNode::default());
        let constant = graph.add_node(ConstantNode::Unsigned(3));
        graph.connect(constant, graph.root, 0);
        let output = graph.execute().unwrap().downcast::<Many<u32>>().unwrap();
        assert_eq!(vec![0, 1, 2], output.collect::<Vec<_>>());
    }
}
