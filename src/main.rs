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
                        _ => {}
                    }
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
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
