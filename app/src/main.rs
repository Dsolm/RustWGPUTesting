use std::time::Instant;

use pasts::Executor;
use renderer::{create_wgpu_renderer_winit, RenderError};
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    {
        let mut renderer = create_wgpu_renderer_winit(&window).await;
        // let _model = renderer.load_model("cube.obj").await.expect("Error while loading model");
        let mut last_render_time = Instant::now();

        event_loop
            .run(|event, control_flow| match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    if !renderer.input(event) {
                        match event {
                            WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                                    ..
                                },
                            ..
                        } => control_flow.exit(),
                            WindowEvent::Resized(physical_size) => {
                                renderer.resize(physical_size.width, physical_size.height);
                            }
                            WindowEvent::RedrawRequested => {
                                window.request_redraw();

                                // if !surface_configured {
                                //     return;
                                // }

                                let now = Instant::now();
                                let dt = now - last_render_time;
                                last_render_time = now;
                                renderer.update(&dt);
                                match renderer.render() {
                                    Ok(_) => {}
                                    Err(RenderError::OutOfMemory) => {
                                        log::error!("OutOfMemory");
                                        control_flow.exit();
                                    }

                                    Err(RenderError::Timeout) => {
                                        log::warn!("Surface timeout")
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion{ delta, },
                    .. // We're not using device_id currently
                } => if renderer.mouse_pressed() {
                    renderer.camera_controller().process_mouse(delta.0, delta.1)
                }
                _ => {}
            })
            .expect("WHAAAAAAAAAAAAT");
    }
}

fn main() {
    let executor = Executor::default();
    executor.block_on(run());
}
