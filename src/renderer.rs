use std::{future::Future, pin::Pin, time::Duration};

use winit::event::WindowEvent;
use crate::camera::camera_controller::CameraController;

pub enum RenderError {
    Timeout,
    OutOfMemory,
}

pub struct ModelHandle(u16); 

pub trait Renderer {
    fn resize(&mut self, width: u32, height: u32);
    fn input(&mut self, event: &WindowEvent) -> bool; // TODO: Remove
    fn update(&mut self, dt: &Duration);
    fn render(&mut self) -> Result<(), RenderError>;

    fn load_model<'a>(&'a mut self, file_path: &'a str) -> Pin<Box<dyn Future<Output = anyhow::Result<ModelHandle>> + Send + '_>>;

    fn mouse_pressed(&self) -> bool; //TODO: Remove
    fn camera_controller(&mut self) -> &mut CameraController; //TODO: Remove
}

mod wgpu_renderer;
pub use wgpu_renderer::create_wgpu_renderer_winit;
