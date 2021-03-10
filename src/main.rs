mod window;

use demoloops_ui::*;
use glutin::dpi::PhysicalPosition;
use nodes::*;
use solstice_2d::{solstice, DrawList};

fn main() {
    let (width, height) = (1920., 1080.);
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size(glutin::dpi::PhysicalSize::new(width, height));
    let (ctx, window) = window::init_ctx(wb, &event_loop);
    let mut ctx = solstice::Context::new(ctx);
    let mut ctx2d = solstice_2d::Graphics::new(&mut ctx, width, height).unwrap();

    let resources_folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("resources");
    let graph_path = resources_folder.join("graph.json");

    let font = ab_glyph::FontVec::try_from_vec({
        let path = resources_folder.join("Roboto-Regular.ttf");
        std::fs::read(path).unwrap()
    })
    .unwrap();
    let font = ctx2d.add_font(font);

    let mut graph = {
        std::fs::read(&graph_path)
            .map_err(eyre::Error::from)
            .and_then(|data| serde_json::from_slice(&data).map_err(eyre::Error::from))
            .unwrap_or_else(|err| {
                eprintln!("{}", err);

                let mut graph = UIGraph::new(font, DrawNode, 500., 500.);
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
                graph.connect(rect, graph.root(), 0);

                graph
            })
    };

    enum ClickState {
        Down(PhysicalPosition<f64>),
        Up,
    }
    struct MouseState {
        position: PhysicalPosition<f64>,
        left: ClickState,
    }
    let mut mouse_state = MouseState {
        position: PhysicalPosition::new(0., 0.),
        left: ClickState::Up,
    };
    let mut selected_node: Option<NodeID> = None;

    let mut show_graph = true;
    let mut times = std::collections::VecDeque::with_capacity(60);

    event_loop.run(move |event, _target, control_flow| {
        use glutin::{event::*, event_loop::*};

        match event {
            Event::WindowEvent { window_id, event } => {
                if window_id == window.id() {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(keycode),
                                    ..
                                },
                            ..
                        } => match keycode {
                            VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                            VirtualKeyCode::Grave => show_graph = !show_graph,
                            _ => (),
                        },
                        WindowEvent::MouseInput { state, button, .. } => match button {
                            MouseButton::Left => match state {
                                ElementState::Pressed => {
                                    mouse_state.left = ClickState::Down(mouse_state.position);
                                    let x = mouse_state.position.x as f32;
                                    let y = mouse_state.position.y as f32;
                                    let selected = graph
                                        .metadata()
                                        .iter()
                                        .find(|(_id, metadata)| metadata.contains(x, y));
                                    if let Some((node, _metadata)) = selected {
                                        selected_node = Some(node);
                                    }
                                }
                                ElementState::Released => {
                                    mouse_state.left = ClickState::Up;
                                    selected_node = None;
                                }
                            },
                            _ => {}
                        },
                        WindowEvent::CursorMoved { position, .. } => {
                            let delta_x = position.x - mouse_state.position.x;
                            let delta_y = position.y - mouse_state.position.y;
                            mouse_state.position = position;
                            if let Some(selected) = selected_node {
                                if let Some(selected) = graph.metadata_mut(selected) {
                                    selected.position.x += delta_x as f32;
                                    selected.position.y += delta_y as f32;
                                }
                            }
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
                ctx.clear();
                {
                    let start = std::time::Instant::now();
                    let result = graph.inner().execute();
                    let elapsed = start.elapsed();
                    while times.len() > 60 {
                        times.pop_front();
                    }
                    times.push_back(elapsed);
                    match result {
                        Ok(output) => {
                            let dl = output.downcast::<One<DrawList>>().unwrap();
                            ctx2d.process(&mut ctx, &dl.inner());
                        }
                        Err(problem) => {
                            if let Some(problem) = graph.inner().nodes().get(problem) {
                                eprintln!("error with {}", problem.name());
                            }
                        }
                    }
                }

                if show_graph {
                    let mut g = ctx2d.lock(&mut ctx);
                    let average_elapsed =
                        times.iter().sum::<std::time::Duration>() / times.len() as u32;
                    g.print(
                        format!("Eval time: {:?}", average_elapsed),
                        font,
                        16.,
                        solstice_2d::Rectangle::new(width / 2., 0., width / 2., 50.),
                    );
                    graph.render(g);
                }

                window.swap_buffers().expect("terrible, terrible damage");
            }
            _ => {}
        }
    });
}
