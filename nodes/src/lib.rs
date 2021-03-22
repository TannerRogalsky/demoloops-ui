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

pub trait FromAny {
    fn from_any(inputs: &mut Vec<Box<dyn std::any::Any>>) -> Result<Self, ()>
    where
        Self: Sized;
}

#[derive(Debug, Clone)]
pub enum OneOrMany<T> {
    One(One<T>),
    Many(Many<T>),
}

impl<T: 'static> FromAny for OneOrMany<T> {
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()>
    where
        Self: Sized,
    {
        if let Some(input) = inputs.get(0) {
            let input = &**input;
            if input.is::<OneOrMany<T>>() {
                Ok(*inputs.remove(0).downcast::<OneOrMany<T>>().unwrap())
            } else if input.is::<One<T>>() {
                let v = *inputs.remove(0).downcast::<One<T>>().unwrap();
                Ok(OneOrMany::One(v))
            } else if input.is::<Many<T>>() {
                let v = *inputs.remove(0).downcast::<Many<T>>().unwrap();
                Ok(OneOrMany::Many(v))
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

impl<T: 'static> OneOrMany<T> {
    pub fn is(v: &dyn Any) -> bool {
        let v = v.type_id();
        std::array::IntoIter::new(Self::type_ids()).any(|t| t == v)
    }

    pub fn downcast(v: Box<dyn Any>) -> Result<Self, Box<dyn Any>> {
        fn take<T>(b: Box<T>) -> T {
            *b
        }

        if v.is::<Self>() {
            v.downcast::<Self>().map(take)
        } else if v.is::<Many<T>>() {
            v.downcast::<Many<T>>().map(take).map(OneOrMany::Many)
        } else if v.is::<One<T>>() {
            v.downcast::<One<T>>().map(take).map(OneOrMany::One)
        } else {
            Err(v)
        }
    }

    pub fn type_ids() -> [std::any::TypeId; 3] {
        use std::any::TypeId;
        [
            TypeId::of::<Self>(),
            TypeId::of::<One<T>>(),
            TypeId::of::<Many<T>>(),
        ]
    }

    pub fn into_boxed_inner(self) -> Box<dyn Any> {
        match self {
            OneOrMany::One(inner) => Box::new(inner),
            OneOrMany::Many(inner) => Box::new(inner),
        }
    }
}

impl<T: Clone> Iterator for OneOrMany<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            OneOrMany::One(v) => Some(v.0.clone()),
            OneOrMany::Many(v) => v.0.next(),
        }
    }
}

pub mod one_many {
    use super::{Many, One, OneOrMany};

    pub fn op1<A, O, FUNC>(a: OneOrMany<A>, op: FUNC) -> OneOrMany<O>
    where
        A: Clone + std::fmt::Debug + 'static,
        O: Clone + std::fmt::Debug + 'static,
        FUNC: Fn(A) -> O + 'static + Clone,
    {
        match a {
            OneOrMany::One(a) => OneOrMany::One(One(op(a.0))),
            OneOrMany::Many(a) => OneOrMany::Many(Many::from(a.0.map(move |a| op(a)))),
        }
    }

    pub fn op2<A, B, O, FUNC>(a: OneOrMany<A>, b: OneOrMany<B>, op: FUNC) -> OneOrMany<O>
    where
        A: Clone + std::fmt::Debug + 'static,
        B: Clone + std::fmt::Debug + 'static,
        O: Clone + std::fmt::Debug + 'static,
        FUNC: Fn(A, B) -> O + 'static + Clone,
    {
        match (a, b) {
            (OneOrMany::One(a), OneOrMany::One(b)) => OneOrMany::One(One(op(a.0, b.0))),
            (a, b) => OneOrMany::Many(Many::from(a.zip(b).map(move |(a, b)| op(a, b)))),
        }
    }

    pub fn op3<A, B, C, O, FUNC>(
        a: OneOrMany<A>,
        b: OneOrMany<B>,
        c: OneOrMany<C>,
        op: FUNC,
    ) -> OneOrMany<O>
    where
        A: Clone + std::fmt::Debug + 'static,
        B: Clone + std::fmt::Debug + 'static,
        C: Clone + std::fmt::Debug + 'static,
        O: Clone + std::fmt::Debug + 'static,
        FUNC: Fn(A, B, C) -> O + 'static + Clone,
    {
        match (a, b, c) {
            (OneOrMany::One(a), OneOrMany::One(b), OneOrMany::One(c)) => {
                OneOrMany::One(One(op(a.0, b.0, c.0)))
            }
            (a, b, c) => OneOrMany::Many(Many::from(
                a.zip(b).zip(c).map(move |((a, b), c)| op(a, b, c)),
            )),
        }
    }

    pub fn op4<A, B, C, D, O, FUNC>(
        a: OneOrMany<A>,
        b: OneOrMany<B>,
        c: OneOrMany<C>,
        d: OneOrMany<D>,
        op: FUNC,
    ) -> OneOrMany<O>
    where
        A: Clone + std::fmt::Debug + 'static,
        B: Clone + std::fmt::Debug + 'static,
        C: Clone + std::fmt::Debug + 'static,
        D: Clone + std::fmt::Debug + 'static,
        O: Clone + std::fmt::Debug + 'static,
        FUNC: Fn(A, B, C, D) -> O + 'static + Clone,
    {
        match (a, b, c, d) {
            (OneOrMany::One(a), OneOrMany::One(b), OneOrMany::One(c), OneOrMany::One(d)) => {
                OneOrMany::One(One(op(a.0, b.0, c.0, d.0)))
            }
            (a, b, c, d) => OneOrMany::Many(Many::from(
                a.zip(b)
                    .zip(c)
                    .zip(d)
                    .map(move |(((a, b), c), d)| op(a, b, c, d)),
            )),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct InputInfo {
    pub name: &'static str,
    pub ty_name: &'static str,
    pub type_id: std::any::TypeId,
}

// Success is matching any of these
#[derive(Debug, Copy, Clone)]
pub struct PossibleInputs<'a> {
    pub groups: &'a [InputGroup<'a>],
}

impl PossibleInputs<'_> {
    pub fn best_match(&self, inputs: &[Box<dyn Any>]) -> Option<&InputGroup> {
        self.groups
            .iter()
            .max_by(|a, b| a.score(inputs).cmp(&b.score(inputs)))
    }
}

// Success is matching all of these
#[derive(Debug, Clone)]
pub struct InputGroup<'a> {
    pub info: std::borrow::Cow<'a, [InputInfo]>,
}

impl InputGroup<'_> {
    pub fn score(&self, inputs: &[Box<dyn Any>]) -> usize {
        self.info
            .iter()
            .zip(inputs.iter())
            .fold(0, |score, (info, input)| {
                let input = &**input;
                if info.type_id == input.type_id() {
                    score + 1
                } else {
                    score
                }
            })
    }
}

pub trait NodeInput {
    // fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> Option<InputMatchError>;
    fn inputs_match(&self, inputs: &[Box<dyn Any>]) -> bool {
        self.inputs().groups.iter().any(|group| {
            group.info.len() == inputs.len()
                && group
                    .info
                    .iter()
                    .zip(inputs.iter())
                    .all(|(info, input)| info.type_id == (**input).type_id())
        })
    }
    fn is_terminator(&self) -> bool {
        false
    }
    fn inputs(&self) -> PossibleInputs;
}

pub trait NodeOutput {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()>;
}

#[typetag::serde(tag = "type")]
pub trait Node:
    std::fmt::Debug
    + dyn_clone::DynClone
    + NodeInput
    + NodeOutput
    + Any
    + Send
    + std::panic::RefUnwindSafe
{
    fn name(&self) -> &'static str;
}
dyn_clone::clone_trait_object!(Node);

impl dyn Node {
    pub fn is<T: Node>(&self) -> bool {
        // Get `TypeId` of the type this function is instantiated with.
        let t = std::any::TypeId::of::<T>();

        // Get `TypeId` of the type in the trait object (`self`).
        let concrete = self.type_id();

        // Compare both `TypeId`s on equality.
        t == concrete
    }

    pub fn downcast_ref<T: Node>(&self) -> Option<&T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(&*(self as *const dyn Node as *const T)) }
        } else {
            None
        }
    }

    pub fn downcast_mut<T: Node>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(&mut *(self as *mut dyn Node as *mut T)) }
        } else {
            None
        }
    }
}

slotmap::new_key_type! {
    pub struct NodeID;
    pub struct InputID;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Valid,
    Invalid,
    Unevaluated,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Unevaluated
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connection {
    pub from: NodeID,
    pub to: NodeID,
    pub input: usize,
    #[serde(skip)]
    pub state: ConnectionState,
}

impl Connection {
    pub fn connects(&self, node: NodeID) -> bool {
        self.from == node || self.to == node
    }
}

#[derive(Debug)]
pub struct Error {
    pub executing_node: NodeID,
    pub inputs: Vec<Box<dyn Any>>,
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
    pub fn node_mut(&mut self, id: NodeID) -> Option<&mut dyn Node> {
        self.nodes.get_mut(id).map(Box::as_mut)
    }
    pub fn connections(&self) -> &[Connection] {
        self.connections.as_slice()
    }

    pub fn add_node<N>(&mut self, node: N) -> NodeID
    where
        N: Node + 'static,
    {
        self.add_boxed_node(Box::new(node))
    }

    pub fn add_boxed_node(&mut self, node: Box<dyn Node>) -> NodeID {
        self.nodes.insert(node)
    }

    pub fn remove_node(&mut self, id: NodeID) -> Option<Box<dyn Node>> {
        if id == self.root {
            None
        } else {
            self.connections.retain(|c| c.from != id && c.to != id);
            self.nodes.remove(id)
        }
    }

    pub fn connect(&mut self, from: NodeID, to: NodeID, input: usize) {
        let not_same_input = |c: &Connection| c.to != to || c.input != input;
        self.connections.retain(not_same_input);

        self.connections.push(Connection {
            from,
            to,
            input,
            state: ConnectionState::Unevaluated,
        })
    }

    pub fn execute(&mut self) -> Result<Box<dyn Any>, Error> {
        for connection in self.connections.iter_mut() {
            connection.state = ConnectionState::Unevaluated;
        }
        self.execute_node(self.root)
    }

    fn execute_node(&mut self, node_id: NodeID) -> Result<Box<dyn Any>, Error> {
        let mut connections = self
            .connections
            .iter()
            .filter(|connection| connection.to == node_id)
            .cloned()
            .collect::<Vec<_>>();
        connections.sort_unstable_by(|a, b| a.input.cmp(&b.input));
        let mut inputs = connections
            .into_iter()
            .map(|connection| {
                let from = self.nodes.get(connection.from).unwrap();
                if from.is_terminator() {
                    let mut inputs = vec![];
                    from.op(&mut inputs).map_err(|_| Error {
                        executing_node: connection.from,
                        inputs,
                    })
                } else {
                    self.execute_node(connection.from)
                }
            })
            .collect::<Result<Vec<Box<dyn Any>>, Error>>()?;
        let to = self.nodes.get(node_id).unwrap();
        let result = to.op(&mut inputs);

        let mut connections = self
            .connections
            .iter_mut()
            .filter(|connection| connection.to == node_id)
            .collect::<Vec<_>>();
        connections.sort_unstable_by(|a, b| a.input.cmp(&b.input));
        if result.is_ok() {
            for connection in connections {
                connection.state = ConnectionState::Valid;
            }
        } else {
            let possible_inputs = to.inputs();
            if let Some(best_match) = possible_inputs.best_match(&inputs) {
                let iter = best_match.info.iter().zip(inputs.iter()).zip(connections);
                for ((info, input), connection) in iter {
                    let input = &**input;
                    let state = if info.type_id == input.type_id() {
                        ConnectionState::Valid
                    } else {
                        ConnectionState::Invalid
                    };
                    connection.state = state;
                }
            } else {
                for connection in connections {
                    connection.state = ConnectionState::Invalid;
                }
            }
        }
        result.map_err(|_| Error {
            executing_node: node_id,
            inputs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_test() {
        let x = OneOrMany::Many(Many::from(0u32..3));
        let y = OneOrMany::Many(Many::from(3u32..6));

        fn repeat(count: u32, v: OneOrMany<u32>) -> Many<u32> {
            match v {
                OneOrMany::One(v) => Many::from((0..count).map(move |_| v.0)),
                OneOrMany::Many(v) => {
                    Many::from((0..count).flat_map(move |r| v.0.clone().map(move |_| r)))
                }
            }
        }

        fn range(count: OneOrMany<u32>) -> Many<u32> {
            match count {
                OneOrMany::One(count) => Many::from(0..count.0),
                OneOrMany::Many(count) => {
                    Many::from(count.0.clone().flat_map(move |_| count.0.clone()))
                }
            }
        }

        let x = repeat(3, x);
        let y = range(y);
        let v = x.inner().zip(y.inner()).collect::<Vec<_>>();

        let control = (0u32..3)
            .flat_map(|x| (3u32..6).map(move |y| (x, y)))
            .collect::<Vec<_>>();
        assert_eq!(control, v);
    }

    #[test]
    fn one_or_many() {
        {
            let a = OneOrMany::One(One::new(2u32));
            let b = OneOrMany::One(One::new(3u32));
            let c = one_many::op2(a, b, std::ops::Mul::mul);
            match c {
                OneOrMany::One(c) => assert_eq!(6u32, c.inner()),
                OneOrMany::Many(_) => panic!(),
            }
        }

        {
            let a = OneOrMany::One(One::new(2u32));
            let b = OneOrMany::Many(Many::from(3u32..5u32));
            let c = one_many::op2(a, b, std::ops::Mul::mul);
            match c {
                OneOrMany::One(_) => panic!(),
                OneOrMany::Many(c) => assert_eq!(vec![6, 8], c.collect::<Vec<_>>()),
            }
        }

        {
            let a = OneOrMany::One(One::new(2u32));
            let b = OneOrMany::Many(Many::from(3u32..5u32));
            let c = one_many::op2(a, b, std::ops::Mul::mul);
            match c {
                OneOrMany::One(_) => panic!(),
                OneOrMany::Many(c) => assert_eq!(vec![6, 8], c.collect::<Vec<_>>()),
            }
        }

        {
            let a = OneOrMany::Many(Many::from(3u32..5u32));
            let b = OneOrMany::Many(Many::from(3u32..5u32));
            let c = one_many::op2(a, b, std::ops::Mul::mul);
            match c {
                OneOrMany::One(_) => panic!(),
                OneOrMany::Many(c) => assert_eq!(vec![9, 16], c.collect::<Vec<_>>()),
            }
        }

        {
            let a = OneOrMany::One(One::new(2u32));
            let b = OneOrMany::One(One::new(3u32));
            let c = OneOrMany::One(One::new(4u32));
            let out = one_many::op3(a, b, c, |a, b, c| a + b + c);
            match out {
                OneOrMany::One(out) => assert_eq!(2 + 3 + 4, out.inner()),
                OneOrMany::Many(_) => panic!(),
            }
        }

        {
            let a = OneOrMany::Many(Many::from(3u32..5u32));
            let b = OneOrMany::One(One::new(0u32));
            let c = OneOrMany::Many(Many::from(3u32..5u32));
            let out = one_many::op3(a, b, c, |a, b, c| a + b + c);
            match out {
                OneOrMany::One(_) => panic!(),
                OneOrMany::Many(out) => assert_eq!(vec![6, 8], out.collect::<Vec<_>>()),
            }
        }

        {
            let a = OneOrMany::Many(Many::from(3u32..5u32));
            let b = OneOrMany::Many(Many::from(3u32..5u32));
            let c = OneOrMany::Many(Many::from(3u32..5u32));
            let out = one_many::op3(a, b, c, |a, b, c| a + b + c);
            match out {
                OneOrMany::One(_) => panic!(),
                OneOrMany::Many(out) => assert_eq!(vec![9, 12], out.collect::<Vec<_>>()),
            }
        }
    }

    #[test]
    fn many_one() {
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
            buffer.push(Box::new(Into::<Many<u32>>::into(0u32..3)));
            buffer.push(Box::new(One(3u32)));
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

            buffer.push(range_output);
            buffer.push(constant_output);
            let ratio_output = ratio.op(&mut buffer).unwrap();

            let constant_output = ConstantNode::Float(2.).op(&mut buffer).unwrap();

            buffer.push(ratio_output);
            buffer.push(constant_output);
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
            let index = graph.add_node(RangeNode);
            let total = graph.add_node(MultiplyNode);

            graph.connect(width, total, 0);
            graph.connect(height, total, 1);

            graph.connect(total, index, 0);

            graph.connect(index, graph.root, 0);
            graph.connect(total, graph.root, 1);

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
