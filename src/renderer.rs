use std::{future::Future, pin::Pin, time::Duration};

use winit::event::WindowEvent;
use crate::camera::camera_controller::CameraController;

pub enum RenderError {
    Timeout,
    OutOfMemory,
}

pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

#[derive(Copy, Clone)]
pub struct ModelHandle(u16); 

pub struct InstanceHandle(ModelHandle, u16);

#[allow(dead_code)]
pub trait Renderer {
    fn resize(&mut self, width: u32, height: u32);
    fn input(&mut self, event: &WindowEvent) -> bool; // TODO: Remove
    fn update(&mut self, dt: &Duration);
    fn render(&mut self) -> Result<(), RenderError>;

    // Will allocate space for upto max_instances upfront.
    fn load_model<'a>(&'a mut self, file_path: &'a str, max_instances: u16) -> Pin<Box<dyn Future<Output = anyhow::Result<ModelHandle>> + Send + 'a>>;

    fn add_instance(&mut self, model: ModelHandle, instance: &Instance) -> InstanceHandle; 
    fn update_instance(&mut self, model: InstanceHandle, instance: &Instance);  // TODO: Make a drop() for InstanceHandle instead instead.
    fn remove_instance(&mut self, model: InstanceHandle);  // TODO: Make a drop() for InstanceHandle instead instead.

    fn mouse_pressed(&self) -> bool; //TODO: Remove
    fn camera_controller(&mut self) -> &mut CameraController; //TODO: Remove
}

mod wgpu_renderer;
pub use wgpu_renderer::create_wgpu_renderer_winit;
