mod nodes;

pub use self::nodes::*;
use ::nodes::{Graph, NodeID};
use solstice_2d::{Color, Draw, FontId, LineVertex, Rectangle};

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct Dimensions {
    pub width: f32,
    pub height: f32,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Metadata {
    pub position: Position,
    pub dimensions: Dimensions,
}

impl Metadata {
    pub fn contains(&self, x: f32, y: f32) -> bool {
        let Position { x: x1, y: y1 } = self.position;
        let x2 = x1 + self.dimensions.width;
        let y2 = y1 + self.dimensions.height;
        x > x1 && x < x2 && y > y1 && y < y2
    }
}

impl Into<Rectangle> for &Metadata {
    fn into(self) -> Rectangle {
        Rectangle::new(
            self.position.x,
            self.position.y,
            self.dimensions.width,
            self.dimensions.height,
        )
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UIGraph {
    inner: Graph,
    metadata: slotmap::SecondaryMap<NodeID, Metadata>,
    #[serde(skip)]
    font: FontId,
}

impl UIGraph {
    pub fn new<N>(font: FontId, root: N, x: f32, y: f32) -> Self
    where
        N: ::nodes::Node + 'static,
    {
        let inner = Graph::with_root(root);
        let mut metadata = slotmap::SecondaryMap::new();
        metadata.insert(
            inner.root(),
            Metadata {
                position: Position { x, y },
                dimensions: Dimensions {
                    width: 100.,
                    height: 100.,
                },
            },
        );
        Self {
            inner,
            metadata,
            font,
        }
    }

    pub fn inner(&self) -> &Graph {
        &self.inner
    }

    pub fn root(&self) -> NodeID {
        self.inner.root()
    }

    pub fn metadata(&self) -> &slotmap::SecondaryMap<NodeID, Metadata> {
        &self.metadata
    }

    pub fn metadata_mut(&mut self, node: NodeID) -> Option<&mut Metadata> {
        self.metadata.get_mut(node)
    }

    pub fn add_node<N>(&mut self, node: N, x: f32, y: f32) -> NodeID
    where
        N: ::nodes::Node + 'static,
    {
        let id = self.inner.add_node(node);
        self.metadata.insert(
            id,
            Metadata {
                position: Position { x, y },
                dimensions: Dimensions {
                    width: 100.,
                    height: 100.,
                },
            },
        );
        id
    }

    pub fn connect(&mut self, from: NodeID, to: NodeID, input: usize) {
        self.inner.connect(from, to, input)
    }

    pub fn render(&self, mut g: solstice_2d::GraphicsLock) {
        for (id, metadata) in self.metadata.iter() {
            if let Some(node) = self.inner.nodes().get(id) {
                let background = metadata.into();
                g.draw_with_color(background, Color::new(1., 0., 0., 1.));
                g.print(node.name(), self.font, 16., background);
                if let Some(input_group) = node.inputs().iter().next() {
                    let Position { x, y } = metadata.position;
                    for (index, input) in input_group.iter().enumerate() {
                        let y = y + (index + 2) as f32 * 16.;
                        g.print(input.name, self.font, 16., Rectangle::new(x, y, 100., 100.));
                    }
                }
            }
        }

        for connection in self.inner.connections() {
            let to = self.metadata.get(connection.to);
            let from = self.metadata.get(connection.from);
            if let (Some(from), Some(to)) = (from, to) {
                let from_pos = Position {
                    x: from.position.x + 100.,
                    y: from.position.y + 50.,
                };
                let to_pos = Position {
                    x: to.position.x,
                    y: to.position.y + (connection.input as f32 + 2.5) * 16.,
                };
                let points = std::array::IntoIter::new([from_pos, to_pos]).map(|p| LineVertex {
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
