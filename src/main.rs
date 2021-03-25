mod window;

use crate::window::winit::event::{ElementState, MouseButton};
use demoloops_ui::*;
use glutin::dpi::PhysicalPosition;
use nodes::*;
use solstice_2d::{solstice, Draw};

fn main() {
    let (width, height) = (1920., 1080.);
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Demoloops UI")
        .with_inner_size(glutin::dpi::PhysicalSize::new(width, height));
    let (ctx, window) = window::init_ctx(wb, &event_loop);
    let mut ctx = solstice::Context::new(ctx);
    let mut ctx_2d = solstice_2d::Graphics::new(&mut ctx, width, height).unwrap();

    let resources_folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("resources");
    let graph_path = resources_folder.join("graph.json");

    let font = ab_glyph::FontVec::try_from_vec({
        let path = resources_folder.join("Roboto-Regular.ttf");
        std::fs::read(path).unwrap()
    })
    .unwrap();
    let font = ctx_2d.add_font(font);

    let mut graph = {
        std::fs::read(&graph_path)
            .map_err(eyre::Error::from)
            .and_then(|data| serde_json::from_slice(&data).map_err(eyre::Error::from))
            .unwrap_or_else(|err| {
                eprintln!("{}", err);

                let mut graph = UIGraph::new(font, ScreenNode, 500., 610.);
                let draw = graph.add_node(DrawNode, 500., 500.);
                let count = graph.add_node(ConstantNode::Unsigned(6), 100., 100.);
                let range = graph.add_node(RangeNode, 100., 210.);
                let d = graph.add_node(ConstantNode::Float(100.), 100., 500.);
                let y = graph.add_node(ConstantNode::Float(10.), 100., 610.);
                let offset = graph.add_node(ConstantNode::Float(900.), 500., 100.);
                let ratio = graph.add_node(RatioNode, 300., 320.);
                let multiply = graph.add_node(MultiplyNode, 500., 210.);
                let rect = graph.add_node(RectangleNode, 300., 500.);

                graph.connect(count, range, 0);

                graph.connect(count, ratio, 0);
                graph.connect(range, ratio, 1);

                graph.connect(offset, multiply, 0);
                graph.connect(ratio, multiply, 1);

                graph.connect(multiply, rect, 0);
                graph.connect(y, rect, 1);
                graph.connect(d, rect, 2);
                graph.connect(d, rect, 3);
                graph.connect(rect, draw, 0);

                graph.connect(draw, graph.root(), 0);

                graph
            })
    };
    let mut resources_cache = command::ResourcesCache::default();

    let mut ui_state = UIState::None;
    let mut mouse_position = PhysicalPosition::new(0., 0.);
    // let mut selected_nodes = std::collections::HashSet::new();

    let mut show_graph = true;
    let mut times = std::collections::VecDeque::with_capacity(60);

    let _execution_thread_handle = std::thread::spawn({
        let graph = graph.clone();
        move || {
            loop {
                let _r = std::panic::catch_unwind(|| {
                    // let mut guard = graph.lock().unwrap();
                    match graph.clone().execute() {
                        Ok(_) => {}
                        Err(_) => {}
                    }
                });
            }
        }
    });

    event_loop.run(move |event, _target, control_flow| {
        use glutin::{event::*, event_loop::*};
        match event {
            Event::WindowEvent { window_id, event } => {
                if window_id == window.id() {
                    match event {
                        WindowEvent::Resized(size) => {
                            ctx.set_viewport(0, 0, size.width as _, size.height as _);
                            ctx_2d.set_width_height(size.width as _, size.height as _);
                        }
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state,
                                    virtual_keycode: Some(key_code),
                                    ..
                                },
                            ..
                        } => {
                            ui_state = ui_state.clone().handle_event(
                                UIEvent::KeyboardInput { state, key_code },
                                UIContext {
                                    mouse_position,
                                    graph: &mut graph,
                                },
                            );
                            if state == ElementState::Pressed {
                                match key_code {
                                    VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                                    VirtualKeyCode::Grave => show_graph = !show_graph,
                                    _ => (),
                                }
                            }
                        }
                        WindowEvent::MouseInput { state, button, .. } => {
                            ui_state = ui_state.clone().handle_event(
                                UIEvent::MouseInput { state, button },
                                UIContext {
                                    mouse_position,
                                    graph: &mut graph,
                                },
                            )
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            ui_state = ui_state.clone().handle_event(
                                UIEvent::MouseMoved(position),
                                UIContext {
                                    mouse_position,
                                    graph: &mut graph,
                                },
                            );
                            mouse_position = position;
                        }
                        _ => {}
                    }
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::LoopDestroyed => match std::fs::File::create(&graph_path) {
                Ok(writer) => {
                    if let Err(err) = serde_json::to_writer_pretty(writer, &graph) {
                        eprintln!("{}", err);
                    }
                }
                Err(err) => {
                    eprintln!("{}", err);
                }
            },
            Event::RedrawRequested(_) => {
                ctx.clear_color(0.1, 0.1, 0.1, 1.);
                ctx.clear();
                {
                    let start = std::time::Instant::now();
                    let result = graph.execute();
                    let elapsed = start.elapsed();
                    while times.len() > 60 {
                        times.pop_front();
                    }
                    times.push_back(elapsed);
                    match result {
                        Ok(output) => {
                            let commands = output.downcast::<One<Vec<command::Command>>>().unwrap();
                            command::Command::batch_execute(
                                &mut ctx,
                                &mut ctx_2d,
                                &mut resources_cache,
                                &*commands,
                            );
                        }
                        Err(_err) => {
                            // eprintln!("{:?}", err);
                        }
                    }
                }

                if show_graph {
                    let mut g = ctx_2d.lock(&mut ctx);
                    let average_elapsed =
                        times.iter().sum::<std::time::Duration>() / times.len() as u32;
                    g.print(
                        format!("Eval time: {:?}", average_elapsed),
                        font,
                        16.,
                        solstice_2d::Rectangle::new(width / 2., 0., width / 2., 50.),
                    );
                    graph.render(&mut g);
                    ui_state.render(
                        &mut g,
                        UIContext {
                            mouse_position,
                            graph: &mut graph,
                        },
                    );
                }

                window.swap_buffers().expect("terrible, terrible damage");
                ::nodes::GlobalNode::incr();
            }
            _ => {}
        }
    });
}

const POSSIBLE_NODES: once_cell::sync::Lazy<Vec<Box<dyn Node>>> =
    once_cell::sync::Lazy::new(|| {
        vec![
            Box::new(::nodes::ConstantNode::Unsigned(0)),
            Box::new(::nodes::GlobalNode),
            Box::new(::nodes::AddNode),
            Box::new(::nodes::MultiplyNode),
            Box::new(::nodes::DivisionNode),
            Box::new(::nodes::ModuloNode),
            Box::new(::nodes::RangeNode),
            Box::new(::nodes::RepeatNode),
            Box::new(::nodes::RatioNode),
            Box::new(::nodes::SineNode),
            Box::new(::nodes::CosNode),
            Box::new(::nodes::ToFloatNode),
            Box::new(ColorNode),
            Box::new(HSLNode),
            Box::new(RectangleNode),
            Box::new(RegularPolygonNode),
            Box::new(NoiseTextureNode),
            Box::new(DrawNode),
        ]
    });

#[derive(Debug, Clone)]
enum Action {
    Move,
    Resize,
    Edit { buffer: String },
}
#[derive(Debug, Clone)]
struct ActionContext {
    node_id: NodeID,
    action: Action,
}
#[derive(Debug, Copy, Clone)]
struct NewNodeContext {
    origin: PhysicalPosition<f64>,
}

impl NewNodeContext {
    pub const ELEMENT_HEIGHT: f32 = 24.;

    pub fn bounds(&self) -> solstice_2d::Rectangle {
        let ox = self.origin.x as f32;
        let oy = self.origin.y as f32;
        solstice_2d::Rectangle::new(
            ox,
            oy,
            100.,
            POSSIBLE_NODES.len() as f32 * Self::ELEMENT_HEIGHT,
        )
    }

    pub fn item_bounds(&self, index: usize) -> solstice_2d::Rectangle {
        let ox = self.origin.x as f32;
        let oy = self.origin.y as f32;
        solstice_2d::Rectangle::new(
            ox,
            oy + index as f32 * Self::ELEMENT_HEIGHT,
            100.,
            Self::ELEMENT_HEIGHT,
        )
    }
}

#[derive(Debug, Copy, Clone)]
struct NewConnectionContext {
    from: NodeID,
}

#[derive(Debug, Copy, Clone)]
struct MultiSelectContext {
    origin: PhysicalPosition<f64>,
}

#[derive(Debug, Clone)]
struct MultiMoveContext {
    selected: Vec<NodeID>,
    moving: bool,
}

#[derive(Debug, Copy, Clone)]
enum UIEvent {
    KeyboardInput {
        state: glutin::event::ElementState,
        key_code: glutin::event::VirtualKeyCode,
    },
    MouseInput {
        state: glutin::event::ElementState,
        button: glutin::event::MouseButton,
    },
    MouseMoved(PhysicalPosition<f64>),
}

struct UIContext<'a> {
    mouse_position: PhysicalPosition<f64>,
    graph: &'a mut UIGraph,
}

#[derive(Clone, Debug)]
enum UIState {
    None,
    NodeAction(ActionContext),
    NewNode(NewNodeContext),
    NewConnection(NewConnectionContext),
    MultiSelect(MultiSelectContext),
    MultiMove(MultiMoveContext),
}

impl UIState {
    pub fn render(&self, g: &mut solstice_2d::GraphicsLock, context: UIContext) {
        const SELECTION_COLOR: solstice_2d::Color = solstice_2d::Color {
            red: 1.,
            green: 0.,
            blue: 1.,
            alpha: 0.5,
        };
        match self {
            UIState::None => {}
            UIState::NodeAction(ctx) => {
                if let Some(metadata) = context.graph.metadata().get(ctx.node_id) {
                    let bounds: solstice_2d::Rectangle = metadata.into();
                    g.stroke_with_color(bounds, SELECTION_COLOR);
                }
            }
            UIState::NewNode(ctx) => {
                let bounds = ctx.bounds();
                g.draw_with_color(bounds, [0.3, 0.3, 0.3, 1.]);
                let mx = context.mouse_position.x as f32;
                let my = context.mouse_position.y as f32;
                for (index, node) in POSSIBLE_NODES.iter().enumerate() {
                    let bounds = ctx.item_bounds(index);
                    g.print(node.name(), solstice_2d::FontId::default(), 16., bounds);
                    if rect_contains(&bounds, mx, my) {
                        g.stroke_with_color(bounds, [1., 1., 0., 0.75]);
                    }
                }
            }
            UIState::NewConnection(ctx) => {
                if let Some(metadata) = context.graph.metadata().get(ctx.from) {
                    let from = rect_center(&metadata.output());
                    let to = Position {
                        x: context.mouse_position.x as f32,
                        y: context.mouse_position.y as f32,
                    };
                    g.line_2d(std::array::IntoIter::new([from, to]).map(|p| {
                        solstice_2d::LineVertex {
                            position: [p.x, p.y, 0.],
                            width: 5.0,
                            color: [1., 1., 1., 1.],
                        }
                    }))
                }
            }
            UIState::MultiSelect(ctx) => {
                let x = ctx.origin.x as f32;
                let y = ctx.origin.y as f32;
                let x2 = context.mouse_position.x as f32;
                let y2 = context.mouse_position.y as f32;
                let width = x2 - x;
                let height = y2 - y;
                let bounds = solstice_2d::Rectangle {
                    x,
                    y,
                    width,
                    height,
                };

                for metadata in context.graph.metadata().values() {
                    let node_bounds: solstice_2d::Rectangle = metadata.into();
                    if rects_collide(&bounds, &node_bounds) {
                        let bounds: solstice_2d::Rectangle = metadata.into();
                        g.stroke_with_color(bounds, SELECTION_COLOR);
                    }
                }

                g.stroke_with_color(bounds, solstice_2d::Color::new(1., 1., 1., 0.75));
            }
            UIState::MultiMove(ctx) => {
                for metadata in ctx
                    .selected
                    .iter()
                    .copied()
                    .filter_map(|node| context.graph.metadata().get(node))
                {
                    let bounds: solstice_2d::Rectangle = metadata.into();
                    g.stroke_with_color(bounds, SELECTION_COLOR);
                }
            }
        }
    }

    pub fn handle_event(mut self, event: UIEvent, context: UIContext) -> Self {
        let UIContext {
            mouse_position,
            graph,
        } = context;
        match self {
            UIState::None => match event {
                UIEvent::KeyboardInput { .. } => self,
                UIEvent::MouseInput { state, button } => match state {
                    ElementState::Pressed => {
                        let x = mouse_position.x as f32;
                        let y = mouse_position.y as f32;
                        let clicked = graph
                            .metadata()
                            .iter()
                            .find(|(_id, metadata)| metadata.contains(x, y));
                        match button {
                            MouseButton::Left => {
                                if let Some((node_id, metadata)) = clicked {
                                    if rect_contains(&metadata.top_bar(), x, y) {
                                        Self::NodeAction(ActionContext {
                                            node_id,
                                            action: Action::Move,
                                        })
                                    } else if rect_contains(&metadata.resize(), x, y) {
                                        Self::NodeAction(ActionContext {
                                            node_id,
                                            action: Action::Resize,
                                        })
                                    } else if rect_contains(&metadata.output(), x, y) {
                                        Self::NewConnection(NewConnectionContext { from: node_id })
                                    } else if let Some(node) = graph
                                        .inner()
                                        .nodes()
                                        .get(node_id)
                                        .and_then(|n| n.downcast_ref::<ConstantNode>())
                                    {
                                        let buffer = match node {
                                            ConstantNode::Unsigned(v) => v.to_string(),
                                            ConstantNode::Float(v) => v.to_string(),
                                        };
                                        Self::NodeAction(ActionContext {
                                            node_id,
                                            action: Action::Edit { buffer },
                                        })
                                    } else {
                                        self
                                    }
                                } else {
                                    Self::MultiSelect(MultiSelectContext {
                                        origin: mouse_position,
                                    })
                                }
                            }
                            MouseButton::Right => {
                                if clicked.is_none() {
                                    Self::NewNode(NewNodeContext {
                                        origin: mouse_position,
                                    })
                                } else {
                                    self
                                }
                            }
                            _ => self,
                        }
                    }
                    ElementState::Released => self,
                },
                UIEvent::MouseMoved(_) => self,
            },
            UIState::NodeAction(mut action) => match event {
                UIEvent::KeyboardInput { state, key_code } => {
                    match &mut action.action {
                        Action::Edit { buffer } => match state {
                            ElementState::Pressed => {
                                use glutin::event::VirtualKeyCode;
                                match key_code {
                                    VirtualKeyCode::Key1 => buffer.push('1'),
                                    VirtualKeyCode::Key2 => buffer.push('2'),
                                    VirtualKeyCode::Key3 => buffer.push('3'),
                                    VirtualKeyCode::Key4 => buffer.push('4'),
                                    VirtualKeyCode::Key5 => buffer.push('5'),
                                    VirtualKeyCode::Key6 => buffer.push('6'),
                                    VirtualKeyCode::Key7 => buffer.push('7'),
                                    VirtualKeyCode::Key8 => buffer.push('8'),
                                    VirtualKeyCode::Key9 => buffer.push('9'),
                                    VirtualKeyCode::Key0 => buffer.push('0'),
                                    VirtualKeyCode::Period => buffer.push('.'),
                                    VirtualKeyCode::Minus => buffer.push('-'),
                                    VirtualKeyCode::Back => {
                                        buffer.pop();
                                    }
                                    VirtualKeyCode::Return | VirtualKeyCode::Escape => {
                                        return UIState::None;
                                    }
                                    _ => {
                                        return UIState::NodeAction(action);
                                    }
                                };
                                let node = graph
                                    .node_mut(action.node_id)
                                    .and_then(|n| n.downcast_mut::<ConstantNode>());
                                if let Some(node) = node {
                                    if let Ok(v) = buffer.parse::<u32>() {
                                        *node = ConstantNode::Unsigned(v);
                                    } else if let Ok(v) = buffer.parse::<f32>() {
                                        *node = ConstantNode::Float(v);
                                    } else {
                                        *node = ConstantNode::Unsigned(0);
                                    }
                                }
                            }
                            ElementState::Released => {}
                        },
                        _ => {}
                    }
                    UIState::NodeAction(action)
                }
                UIEvent::MouseInput { state, .. } => match state {
                    ElementState::Pressed => UIState::NodeAction(action),
                    ElementState::Released => match action.action {
                        Action::Move | Action::Resize => UIState::None,
                        Action::Edit { .. } => UIState::NodeAction(action),
                    },
                },
                UIEvent::MouseMoved(position) => {
                    let delta_x = position.x - mouse_position.x;
                    let delta_y = position.y - mouse_position.y;
                    let node_id = action.node_id;
                    match &action.action {
                        Action::Move => {
                            if let Some(selected) = graph.metadata_mut(node_id) {
                                selected.position.x += delta_x as f32;
                                selected.position.y += delta_y as f32;
                            }
                        }
                        Action::Resize => {
                            if let Some(selected) = graph.metadata_mut(node_id) {
                                selected.dimensions.width += delta_x as f32;
                                selected.dimensions.height += delta_y as f32;
                            }
                        }
                        Action::Edit { .. } => {}
                    }
                    UIState::NodeAction(action)
                }
            },
            UIState::NewNode(ctx) => match event {
                UIEvent::KeyboardInput { .. } => self,
                UIEvent::MouseMoved(_) => self,
                UIEvent::MouseInput { state, button } => match (state, button) {
                    (ElementState::Pressed, MouseButton::Left) => {
                        let bounds = ctx.bounds();
                        let mx = mouse_position.x as f32;
                        let my = mouse_position.y as f32;
                        if rect_contains(&bounds, mx, my) {
                            let clicked =
                                POSSIBLE_NODES.iter().enumerate().find_map(|(index, node)| {
                                    let bounds = ctx.item_bounds(index);
                                    if rect_contains(&bounds, mx, my) {
                                        Some(node.clone())
                                    } else {
                                        None
                                    }
                                });
                            if let Some(node) = clicked {
                                let x = ctx.origin.x as f32;
                                let y = ctx.origin.y as f32;
                                graph.add_boxed_node(node, x, y);
                            }
                        }
                        UIState::None
                    }
                    (ElementState::Pressed, MouseButton::Right) => {
                        UIState::NewNode(NewNodeContext {
                            origin: mouse_position,
                        })
                    }
                    _ => self,
                },
            },
            UIState::NewConnection(ctx) => match event {
                UIEvent::KeyboardInput { .. } => self,
                UIEvent::MouseMoved(_) => self,
                UIEvent::MouseInput { state, button } => match (state, button) {
                    (ElementState::Released, MouseButton::Left) => {
                        let mx = context.mouse_position.x as f32;
                        let my = context.mouse_position.y as f32;
                        let clicked = graph
                            .metadata()
                            .iter()
                            .find(|(_id, metadata)| {
                                let bounds: solstice_2d::Rectangle = (*metadata).into();
                                rect_contains(&bounds, mx, my)
                            })
                            .and_then(|(id, metadata)| {
                                graph
                                    .inner()
                                    .nodes()
                                    .get(id)
                                    .map(|node| (id, node, metadata))
                            });
                        if let Some((to, node, metadata)) = clicked {
                            if let Some(most) = node
                                .inputs()
                                .groups
                                .iter()
                                .max_by(|a, b| a.info.len().cmp(&b.info.len()))
                            {
                                let input =
                                    most.info.iter().enumerate().find_map(|(index, _info)| {
                                        if rect_contains(&metadata.input(index), mx, my) {
                                            Some(index)
                                        } else {
                                            None
                                        }
                                    });

                                if let Some(input) = input {
                                    graph.connect(ctx.from, to, input);
                                }
                            }
                        }
                        UIState::None
                    }
                    _ => UIState::None,
                },
            },
            UIState::MultiSelect(ctx) => match event {
                UIEvent::KeyboardInput { .. } => self,
                UIEvent::MouseMoved(_) => self,
                UIEvent::MouseInput { state, button } => match (state, button) {
                    (ElementState::Released, MouseButton::Left) => {
                        UIState::MultiMove(MultiMoveContext {
                            selected: {
                                let x = ctx.origin.x as f32;
                                let y = ctx.origin.y as f32;
                                let x2 = context.mouse_position.x as f32;
                                let y2 = context.mouse_position.y as f32;
                                let width = x2 - x;
                                let height = y2 - y;
                                let bounds = solstice_2d::Rectangle {
                                    x,
                                    y,
                                    width,
                                    height,
                                };

                                graph
                                    .metadata()
                                    .iter()
                                    .filter_map(|(id, metadata)| {
                                        let node_bounds: solstice_2d::Rectangle = metadata.into();
                                        if rects_collide(&bounds, &node_bounds) {
                                            Some(id)
                                        } else {
                                            None
                                        }
                                    })
                                    .collect()
                            },
                            moving: false,
                        })
                    }
                    _ => self,
                },
            },
            UIState::MultiMove(ref mut ctx) => match event {
                UIEvent::KeyboardInput { state, key_code } => match (state, key_code) {
                    (ElementState::Pressed, glutin::event::VirtualKeyCode::Delete) => {
                        for node in ctx.selected.iter().copied() {
                            graph.remove_node(node);
                        }
                        UIState::None
                    }
                    _ => self,
                },
                UIEvent::MouseMoved(position) => {
                    if ctx.moving {
                        let dx = mouse_position.x - position.x;
                        let dy = mouse_position.y - position.y;

                        for node in ctx.selected.iter().copied() {
                            if let Some(metadata) = graph.metadata_mut(node) {
                                metadata.position.x -= dx as f32;
                                metadata.position.y -= dy as f32;
                            }
                        }
                    }
                    self
                }
                UIEvent::MouseInput { state, button } => match (state, button) {
                    (ElementState::Pressed, MouseButton::Left) => {
                        let mx = mouse_position.x as f32;
                        let my = mouse_position.y as f32;
                        let should_move = ctx
                            .selected
                            .iter()
                            .copied()
                            .filter_map(|node| graph.metadata().get(node))
                            .any(|metadata| {
                                let bounds = metadata.into();
                                rect_contains(&bounds, mx, my)
                            });
                        if should_move {
                            ctx.moving = true;
                            self
                        } else {
                            UIState::None
                        }
                    }
                    (ElementState::Released, MouseButton::Left) => {
                        ctx.moving = false;
                        self
                    }
                    _ => self,
                },
            },
        }
    }
}
