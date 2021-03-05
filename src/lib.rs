mod nodes;

use nodes::*;
use serde::{Deserialize, Serialize};
use slotmap::SlotMap;
use std::any::Any;

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Pair<A, B> {
    lhs: std::marker::PhantomData<A>,
    rhs: std::marker::PhantomData<B>,
}

trait NodeInput {
    fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool;
    fn is_terminator(&self) -> bool {
        false
    }
}

trait NodeOutput {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()>;
}

#[typetag::serde(tag = "type")]
trait Node: std::fmt::Debug + dyn_clone::DynClone + NodeInput + NodeOutput {}
dyn_clone::clone_trait_object!(Node);

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
    fn with_root<N>(root: N) -> Self
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

    fn add_node<N>(&mut self, node: N) -> NodeID
    where
        N: Node + 'static,
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
            .filter(|connection| connection.to == node_id)
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

        {
            let mut graph = Graph::with_root(RangeNode::default());
            let constant = graph.add_node(ConstantNode::Unsigned(3));
            graph.connect(constant, graph.root, 0);
            let output = graph.execute().unwrap().downcast::<Many<u32>>().unwrap();
            assert_eq!(vec![0, 1, 2], output.collect::<Vec<_>>());
        }

        {
            const WIDTH: u32 = 3;
            const HEIGHT: u32 = 5;

            let mut graph = Graph::with_root(RatioNode::default());
            let width = graph.add_node(ConstantNode::Unsigned(WIDTH));
            let height = graph.add_node(ConstantNode::Unsigned(HEIGHT));
            let index = graph.add_node(Range2DNode::default());
            let total = graph.add_node(MultiplyNode::default());

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
