mod window;

use demoloops_ui::*;
use solstice_2d::{solstice, Draw};

fn main() {
    let (width, height) = (1920., 1080.);
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size(glutin::dpi::PhysicalSize::new(width, height));
    let (ctx, window) = window::init_ctx(wb, &event_loop);
    let mut ctx = solstice::Context::new(ctx);
    let mut ctx2d = solstice_2d::Graphics::new(&mut ctx, width, height).unwrap();

    let graph = Graph::new();

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
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => {}
                    }
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                {
                    let mut g = ctx2d.lock(&mut ctx);
                    // for command in graph.root.inputs.iter() {
                    //     match command {
                    //         Command::Clear(Clear { color }) => {
                    //             g.clear(solstice_2d::Color {
                    //                 red: color.red,
                    //                 green: color.green,
                    //                 blue: color.blue,
                    //                 alpha: color.alpha,
                    //             });
                    //         }
                    //         Command::Rectangle(rectangle) => g.draw_with_color(
                    //             solstice_2d::Rectangle {
                    //                 x: rectangle.x,
                    //                 y: rectangle.y,
                    //                 width: rectangle.width,
                    //                 height: rectangle.height,
                    //             },
                    //             solstice_2d::Color {
                    //                 red: rectangle.color.red,
                    //                 green: rectangle.color.green,
                    //                 blue: rectangle.color.blue,
                    //                 alpha: rectangle.color.alpha,
                    //             },
                    //         ),
                    //     }
                    // }
                }

                window.swap_buffers().expect("terrible, terrible damage");
            }
            _ => {}
        }
    });
}
