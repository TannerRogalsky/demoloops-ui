mod nodes;

pub use self::nodes::*;
use serde::{Deserialize, Serialize};
use slotmap::SlotMap;
use std::any::Any;

#[derive(Debug, Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct One<T>(T);

impl<T> One<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }

    pub fn inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for One<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait ManyTrait<T>: Iterator<Item = T> + dyn_clone::DynClone + std::fmt::Debug {}
dyn_clone::clone_trait_object!(<T> ManyTrait<T>);
impl<I, T> ManyTrait<T> for I where I: Iterator<Item = T> + Clone + std::fmt::Debug {}

// If it turns out that there are only so many different types of iterator then this could
// be replaced internally with an enum and the From implementation restricted
#[derive(Debug, Clone)]
pub struct Many<T>(Box<dyn ManyTrait<T>>);

impl<T> Many<T> {
    pub fn inner(self) -> Box<dyn ManyTrait<T>> {
        self.0
    }
}

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
    <I as IntoIterator>::IntoIter: ManyTrait<T>,
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

pub trait FromAny {
    fn from_any(inputs: &mut Vec<Box<dyn std::any::Any>>) -> Result<Self, ()>
    where
        Self: Sized;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Pair<A, B> {
    lhs: A,
    rhs: B,
}

impl<A, B> Pair<A, B>
where
    A: 'static,
    B: 'static,
{
    fn can_match(inputs: &[Box<dyn Any>]) -> bool {
        if inputs.len() == 2 {
            inputs[0].is::<A>() && inputs[1].is::<B>()
        } else {
            false
        }
    }
}

impl<A, B> FromAny for Pair<A, B>
where
    A: 'static,
    B: 'static,
{
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
        if let Some(rhs) = inputs.pop() {
            if let Some(lhs) = inputs.pop() {
                let lhs = lhs.downcast::<A>();
                let rhs = rhs.downcast::<B>();
                match (lhs, rhs) {
                    (Ok(lhs), Ok(rhs)) => Ok(Self {
                        lhs: *lhs,
                        rhs: *rhs,
                    }),
                    (Ok(lhs), Err(rhs)) => {
                        inputs.push(lhs);
                        inputs.push(rhs);
                        Err(())
                    }
                    (Err(lhs), Ok(rhs)) => {
                        inputs.push(lhs);
                        inputs.push(rhs);
                        Err(())
                    }
                    (Err(lhs), Err(rhs)) => {
                        inputs.push(lhs);
                        inputs.push(rhs);
                        Err(())
                    }
                }
            } else {
                inputs.push(rhs);
                Err(())
            }
        } else {
            Err(())
        }
    }
}

pub trait NodeInput {
    fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool;
    fn is_terminator(&self) -> bool {
        false
    }
}

pub trait NodeOutput {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()>;
}

#[typetag::serde(tag = "type")]
pub trait Node: std::fmt::Debug + dyn_clone::DynClone + NodeInput + NodeOutput {
    fn name(&self) -> &'static str;
}
dyn_clone::clone_trait_object!(Node);

slotmap::new_key_type! {
    pub struct NodeID;
    pub struct InputID;
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connection {
    pub from: NodeID,
    pub to: NodeID,
    pub input: usize,
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
    pub fn with_root<N>(root: N) -> Self
    where
        N: Node + 'static,
    {
        let mut nodes: SlotMap<NodeID, Box<dyn Node>> = SlotMap::with_key();
        let root = nodes.insert(Box::new(root));
        Self {
            root,
            nodes,
            connections: vec![],
        }
    }

    pub fn root(&self) -> NodeID {
        self.root
    }
    pub fn nodes(&self) -> &SlotMap<NodeID, Box<dyn Node>> {
        &self.nodes
    }
    pub fn connections(&self) -> &[Connection] {
        self.connections.as_slice()
    }

    pub fn add_node<N>(&mut self, node: N) -> NodeID
    where
        N: Node + 'static,
    {
        self.nodes.insert(Box::new(node))
    }

    pub fn connect(&mut self, from: NodeID, to: NodeID, input: usize) {
        self.connections.push(Connection { from, to, input })
    }

    pub fn execute(&self) -> Result<Box<dyn Any>, NodeID> {
        self.execute_node(self.root)
    }

    fn execute_node(&self, node_id: NodeID) -> Result<Box<dyn Any>, NodeID> {
        let to = self.nodes.get(node_id).unwrap();
        let mut connections = self
            .connections
            .iter()
            .filter(|connection| connection.to == node_id)
            .collect::<Vec<_>>();
        connections.sort_unstable_by(|a, b| a.input.cmp(&b.input));
        let mut input = connections
            .into_iter()
            .map(|connection| {
                let input = self.nodes.get(connection.from).unwrap();
                if input.is_terminator() {
                    let mut buf = vec![];
                    input.op(&mut buf).map_err(|_| connection.from)
                } else {
                    self.execute_node(connection.from)
                }
            })
            .collect::<Result<Vec<Box<dyn Any>>, NodeID>>()?;
        to.op(&mut input).map_err(|_| node_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn many_one() {
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
            assert!(MultiplyNode.inputs_match(&buffer));

            let output = MultiplyNode.op(&mut buffer).unwrap();
            assert!(buffer.is_empty());
            buffer.push(output);
            buffer.push(Box::new(Into::<Many<u32>>::into(vec![3u32, 4, 5])));
            let output = MultiplyNode.op(&mut buffer).unwrap();

            let output = output.downcast::<Many<u32>>().unwrap();
            assert_eq!(vec![12u32, 24, 40], output.collect::<Vec<_>>());
        }

        {
            let mut buffer: Vec<Box<dyn Any>> = Vec::new();
            buffer.push(Box::new(One(3u32)));
            buffer.push(Box::new(Into::<Many<u32>>::into(0u32..3)));
            assert!(RatioNode.inputs_match(&buffer));

            let output = RatioNode.op(&mut buffer).unwrap();
            assert!(buffer.is_empty());

            let output = output.downcast::<Many<f32>>().unwrap();
            assert_eq!(vec![0f32, 1. / 3., 2. / 3.], output.collect::<Vec<_>>());
        }

        {
            let mut buffer: Vec<Box<dyn Any>> = vec![];

            let constant = ConstantNode::Unsigned(3);
            let range = RangeNode;
            let ratio = RatioNode;
            let multiply = MultiplyNode;

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

        {
            let mut graph = Graph::with_root(RangeNode);
            let constant = graph.add_node(ConstantNode::Unsigned(3));
            graph.connect(constant, graph.root, 0);
            let output = graph.execute().unwrap().downcast::<Many<u32>>().unwrap();
            assert_eq!(vec![0, 1, 2], output.collect::<Vec<_>>());
        }

        {
            const WIDTH: u32 = 3;
            const HEIGHT: u32 = 5;

            let mut graph = Graph::with_root(RatioNode);
            let width = graph.add_node(ConstantNode::Unsigned(WIDTH));
            let height = graph.add_node(ConstantNode::Unsigned(HEIGHT));
            let index = graph.add_node(Range2DNode);
            let total = graph.add_node(MultiplyNode);

            graph.connect(width, index, 0);
            graph.connect(height, index, 1);

            graph.connect(width, total, 0);
            graph.connect(height, total, 1);

            graph.connect(total, graph.root, 0);
            graph.connect(index, graph.root, 1);

            let output = graph.execute().unwrap().downcast::<Many<f32>>().unwrap();
            let control = (0..WIDTH)
                .flat_map(|vx| (0..HEIGHT).map(move |vy| vy + vx * HEIGHT))
                .map(|i| ((i % (WIDTH * HEIGHT)) as f64 / (WIDTH * HEIGHT) as f64) as f32);
            assert_eq!(control.collect::<Vec<_>>(), output.collect::<Vec<_>>());
        }

        {
            let width = 2;
            let height = 3;

            let output1 = (0..width)
                .flat_map(|y| (0..height).map(move |x| x + y * height))
                .collect::<Vec<_>>();
            assert_eq!(vec![0, 1, 2, 3, 4, 5], output1);

            let output2 = (0..width).flat_map(|_y| (0..height)).collect::<Vec<_>>();
            assert_eq!(vec![0, 1, 2, 0, 1, 2], output2);
        }
    }
}
