use crate::app::App;
use winit::error::EventLoopError;
use winit::event_loop::{ControlFlow, EventLoop};

mod instance;
mod app;
mod camera;
mod camera_controller;
mod camera_uniform;
mod img_utils;
mod texture;
mod vertex;
mod wgpu_ctx;
fn main() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app)
}
