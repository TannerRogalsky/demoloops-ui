mod nodes;

pub use self::nodes::*;
use ::nodes::{Graph, NodeID};
use solstice_2d::{Color, Draw, FontId, LineVertex, Rectangle};

#[derive(Debug, Copy, Clone)]
struct Position {
    x: f32,
    y: f32,
}

struct Metadata {
    position: Position,
}

pub struct UIGraph {
    inner: Graph,
    nodes: slotmap::SecondaryMap<NodeID, Metadata>,
    font: FontId,
}

impl UIGraph {
    pub fn new<N>(font: FontId, root: N, x: f32, y: f32) -> Self
    where
        N: ::nodes::Node + 'static,
    {
        let inner = Graph::with_root(root);
        let mut nodes = slotmap::SecondaryMap::new();
        nodes.insert(
            inner.root(),
            Metadata {
                position: Position { x, y },
            },
        );
        Self { inner, nodes, font }
    }

    pub fn inner(&self) -> &Graph {
        &self.inner
    }

    pub fn root(&self) -> NodeID {
        self.inner.root()
    }

    pub fn add_node<N>(&mut self, node: N, x: f32, y: f32) -> NodeID
    where
        N: ::nodes::Node + 'static,
    {
        let id = self.inner.add_node(node);
        self.nodes.insert(
            id,
            Metadata {
                position: Position { x, y },
            },
        );
        id
    }

    pub fn connect(&mut self, from: NodeID, to: NodeID, input: usize) {
        self.inner.connect(from, to, input)
    }

    pub fn render(&self, mut g: solstice_2d::GraphicsLock) {
        for (id, metadata) in self.nodes.iter() {
            if let Some(node) = self.inner.nodes().get(id) {
                let Position { x, y } = metadata.position;
                g.draw_with_color(Rectangle::new(x, y, 100., 100.), Color::new(1., 0., 0., 1.));
                g.print(
                    node.name(),
                    self.font,
                    16.,
                    Rectangle::new(x, y, 100., 100.),
                );
                if let Some(input_group) = node.inputs().iter().next() {
                    for (index, input) in input_group.iter().enumerate() {
                        let y = y + (index + 2) as f32 * 16.;
                        g.print(input.name, self.font, 16., Rectangle::new(x, y, 100., 100.));
                    }
                }
            }
        }

        for connection in self.inner.connections() {
            let to = self.nodes.get(connection.to);
            let from = self.nodes.get(connection.from);
            if let (Some(from), Some(to)) = (from, to) {
                let from_pos = Position {
                    x: from.position.x + 100.,
                    y: from.position.y + 50.,
                };
                let to_pos = Position {
                    x: to.position.x,
                    y: to.position.y + (connection.input as f32 + 2.5) * 16.
                };
                let points =
                    std::array::IntoIter::new([from_pos, to_pos]).map(|p| LineVertex {
                        position: [p.x, p.y, 0.],
                        width: 5.0,
                        ..LineVertex::default()
                    });
                g.line_2d(points);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::nodes::*;

    #[test]
    fn serialize() {
        let mut graph = Graph::with_root(DrawNode);
        graph.add_node(ConstantNode::Float(123.213));
        graph.add_node(RatioNode);
        graph.add_node(MultiplyNode);
        println!("{}", serde_json::to_string_pretty(&graph).unwrap());
    }
}
