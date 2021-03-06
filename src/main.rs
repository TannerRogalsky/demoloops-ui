mod window;

use demoloops_ui::*;
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

    let font = ab_glyph::FontVec::try_from_vec({
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("resources");
        let path = folder.join("Roboto-Regular.ttf");
        std::fs::read(path).unwrap()
    })
    .unwrap();
    let font = ctx2d.add_font(font);

    let mut graph = Graph::with_root(DrawNode::default());
    let d = graph.add_node(ConstantNode::Float(100.));
    let rect = graph.add_node(RectangleNode::default());
    graph.connect(d, rect, 0);
    graph.connect(d, rect, 1);
    graph.connect(d, rect, 2);
    graph.connect(d, rect, 3);
    graph.connect(rect, graph.root(), 0);

    let graph = UIGraph::new(graph, font);

    let mut show_graph = true;

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
                        _ => {}
                    }
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                {
                    match graph.inner().execute() {
                        Ok(output) => {
                            let dl = output.downcast::<One<DrawList>>().unwrap();
                            ctx2d.process(&mut ctx, &*dl);
                        }
                        Err(_) => eprintln!("graph execution failed"),
                    }
                }

                if show_graph {
                    graph.render(ctx2d.lock(&mut ctx))
                }

                window.swap_buffers().expect("terrible, terrible damage");
            }
            _ => {}
        }
    });
}
