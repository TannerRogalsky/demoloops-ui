pub mod command;
mod nodes;

pub use self::nodes::*;
use ::nodes::{ConnectionState, Graph, NodeID};
use solstice_2d::{Color, Draw, FontId, LineVertex, Rectangle};
use std::any::Any;

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

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct Metadata {
    pub position: Position,
    pub dimensions: Dimensions,
}

pub fn rect_contains(rect: &Rectangle, x: f32, y: f32) -> bool {
    let x1 = rect.x;
    let y1 = rect.y;
    let x2 = x1 + rect.width;
    let y2 = y1 + rect.height;
    x > x1 && x < x2 && y > y1 && y < y2
}

pub fn rects_collide(a: &Rectangle, b: &Rectangle) -> bool {
    rect_contains(a, b.x, b.y)
        || rect_contains(a, b.x + b.width, b.y)
        || rect_contains(a, b.x + b.width, b.y + b.height)
        || rect_contains(a, b.x, b.y + b.height)
}

pub fn rect_center(rect: &Rectangle) -> Position {
    Position {
        x: rect.x + rect.width / 2.,
        y: rect.y + rect.height / 2.,
    }
}

impl Metadata {
    const TOP_BAR_HEIGHT: f32 = 16.;
    const OUTPUT_WIDTH: f32 = 16.;

    pub fn contains(&self, x: f32, y: f32) -> bool {
        let rect = self.into();
        rect_contains(&rect, x, y)
    }

    pub fn top_bar(&self) -> Rectangle {
        Rectangle {
            x: self.position.x,
            y: self.position.y,
            width: self.dimensions.width,
            height: Self::TOP_BAR_HEIGHT,
        }
    }

    pub fn input(&self, index: usize) -> Rectangle {
        let y = self.position.y + Self::TOP_BAR_HEIGHT;
        let width = self.dimensions.width - Self::OUTPUT_WIDTH * 2.;
        Rectangle {
            x: self.position.x,
            y: y + (index as f32 + 1.) * Self::TOP_BAR_HEIGHT,
            width,
            height: Self::TOP_BAR_HEIGHT,
        }
    }

    pub fn output(&self) -> Rectangle {
        Rectangle {
            x: self.position.x + self.dimensions.width - Self::OUTPUT_WIDTH,
            y: self.position.y + Self::TOP_BAR_HEIGHT,
            width: Self::OUTPUT_WIDTH,
            height: self.dimensions.height - Self::TOP_BAR_HEIGHT - Self::OUTPUT_WIDTH,
        }
    }

    pub fn resize(&self) -> Rectangle {
        let x = self.position.x + self.dimensions.width - Self::OUTPUT_WIDTH;
        let y = self.position.y + self.dimensions.height - Self::OUTPUT_WIDTH;
        Rectangle {
            x,
            y,
            width: Self::OUTPUT_WIDTH,
            height: Self::OUTPUT_WIDTH,
        }
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

    pub fn execute(&mut self) -> Result<Box<dyn Any>, ::nodes::Error> {
        self.inner.execute()
    }

    pub fn inner(&self) -> &Graph {
        &self.inner
    }

    pub fn node_mut(&mut self, id: NodeID) -> Option<&mut dyn ::nodes::Node> {
        self.inner.node_mut(id)
    }

    pub fn remove_node(&mut self, id: NodeID) -> Option<Box<dyn ::nodes::Node>> {
        let removed = self.inner.remove_node(id);
        if removed.is_some() {
            self.metadata.remove(id);
        }
        removed
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
        self.add_boxed_node(Box::new(node), x, y)
    }

    pub fn add_boxed_node(&mut self, node: Box<dyn ::nodes::Node>, x: f32, y: f32) -> NodeID {
        let id = self.inner.add_boxed_node(node);
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

    pub fn render(&self, g: &mut solstice_2d::GraphicsLock) {
        let black = Color::new(0., 0., 0., 1.);
        for (id, metadata) in self.metadata.iter() {
            if let Some(node) = self.inner.nodes().get(id) {
                let background: Rectangle = metadata.into();
                g.draw_with_color(background, Color::new(1., 0., 0., 1.));
                g.stroke_with_color(background, black);

                g.draw_with_color(metadata.top_bar(), Color::new(0.3, 0.3, 0.3, 1.));
                g.stroke_with_color(metadata.top_bar(), black);
                let text_bounds = Rectangle {
                    x: background.x + 5.,
                    ..background
                };
                g.print(node.name(), self.font, 16., text_bounds);

                g.draw_with_color(metadata.output(), Color::new(1., 1., 0., 1.));
                g.stroke_with_color(metadata.output(), black);

                g.draw_with_color(metadata.resize(), Color::new(1., 0., 1., 1.));
                g.stroke_with_color(metadata.resize(), black);

                if let Some(input_group) = node.inputs().groups.iter().next() {
                    let mut draw_input = |info: &::nodes::InputInfo, index: usize| {
                        let rect: Rectangle = metadata.input(index);
                        g.draw_with_color(rect, Color::new(0., 0., 1., 1.));
                        g.stroke_with_color(rect, black);
                        let text_bounds = Rectangle {
                            x: rect.x + 5.,
                            ..rect
                        };
                        g.print(info.name, self.font, 16., text_bounds);
                    };

                    if node.variadic() {
                        if let Some(info) = input_group.info.first() {
                            let connections = self
                                .inner
                                .connections()
                                .iter()
                                .filter(|c| c.to == id)
                                .count();
                            for index in 0..=connections {
                                draw_input(info, index);
                            }
                        }
                    } else {
                        for (index, info) in input_group.info.iter().enumerate() {
                            draw_input(info, index);
                        }
                    }
                }

                {
                    use ::nodes::ConstantNode;
                    if let Some(constant) = node.downcast_ref::<ConstantNode>() {
                        let text = match constant {
                            ConstantNode::Unsigned(v) => {
                                format!("{}", v)
                            }
                            ConstantNode::Float(v) => {
                                format!("{:.2}", v)
                            }
                        };
                        let bounds = Rectangle {
                            x: metadata.position.x + 5.,
                            y: metadata.position.y + Metadata::TOP_BAR_HEIGHT * 2.,
                            width: metadata.dimensions.width - Metadata::OUTPUT_WIDTH,
                            height: metadata.dimensions.height
                                - 10.
                                - Metadata::TOP_BAR_HEIGHT * 2.,
                        };
                        g.print(text, self.font, 32., bounds);
                    }
                }

                {
                    use ::nodes::GlobalNode;
                    if node.is::<GlobalNode>() {
                        let text = GlobalNode::load().to_string();
                        let bounds = Rectangle {
                            x: metadata.position.x + 5.,
                            y: metadata.position.y + Metadata::TOP_BAR_HEIGHT * 2.,
                            width: metadata.dimensions.width - Metadata::OUTPUT_WIDTH,
                            height: metadata.dimensions.height
                                - 10.
                                - Metadata::TOP_BAR_HEIGHT * 2.,
                        };
                        g.print(text, self.font, 32., bounds);
                    }
                }
            }
        }

        for connection in self.inner.connections() {
            let to = self.metadata.get(connection.to);
            let from = self.metadata.get(connection.from);
            if let (Some(from), Some(to)) = (from, to) {
                let from_pos = rect_center(&from.output());
                let to_pos = {
                    let rect = to.input(connection.input);
                    Position {
                        x: rect.x,
                        y: rect.y + rect.height / 2.,
                    }
                };

                let color = match connection.state {
                    ConnectionState::Valid => [0., 1., 0., 1.],
                    ConnectionState::Invalid => [1., 0., 0., 1.],
                    ConnectionState::Unevaluated => [1., 1., 1., 1.],
                };

                let points = std::array::IntoIter::new([from_pos, to_pos]);
                let points = points.map(move |p| LineVertex {
                    position: [p.x, p.y, 0.],
                    width: 5.0,
                    color,
                });
                g.line_2d(points);
            }
        }
    }
}

#[cfg(test)]
mod tests {}
