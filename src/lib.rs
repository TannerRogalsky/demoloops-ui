mod nodes;

pub use self::nodes::*;
use ::nodes::{Graph, NodeID};
use solstice_2d::{Color, Draw, FontId, LineVertex, Rectangle};

#[derive(Debug, Copy, Clone)]
struct Position {
    x: f32,
    y: f32,
}

struct Node {
    position: Position,
}

pub struct UIGraph {
    inner: Graph,
    nodes: slotmap::SecondaryMap<NodeID, Node>,
    font: FontId,
}

impl UIGraph {
    pub fn new(inner: Graph, font: FontId) -> Self {
        let nodes = inner
            .nodes()
            .keys()
            .enumerate()
            .map(|(index, id)| {
                (
                    id,
                    Node {
                        position: Position {
                            x: index as f32 * 125. + 50.,
                            y: if index % 2 == 0 { 300. } else { 400. },
                        },
                    },
                )
            })
            .collect();
        Self { inner, nodes, font }
    }

    pub fn inner(&self) -> &Graph {
        &self.inner
    }

    pub fn render(&self, mut g: solstice_2d::GraphicsLock) {
        for (id, metadata) in self.nodes.iter() {
            if let Some(node) = self.inner.nodes().get(id) {
                let Position { x, y } = metadata.position;
                g.draw_with_color(
                    Rectangle::new(x, y, 100., 100.),
                    Color::new(1., 0., 0., 1.),
                );
                g.print(node.name(), self.font, 16., Rectangle::new(x, y, 100., 100.));
            }
        }

        for connection in self.inner.connections() {
            let to = self.nodes.get(connection.to);
            let from = self.nodes.get(connection.from);
            if let (Some(from), Some(to)) = (from, to) {
                g.line_2d(
                    vec![from.position, to.position]
                        .into_iter()
                        .map(|p| LineVertex {
                            position: [p.x, p.y, 0.],
                            width: 5.0,
                            ..LineVertex::default()
                        }),
                )
            }
        }
    }
}
