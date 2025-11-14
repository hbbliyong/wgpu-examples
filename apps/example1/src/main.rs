use crate::app::App;
use std::fs::OpenOptions;
use std::io::Write;
use winit::error::EventLoopError;
use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod camera;
mod camera_controller;
mod camera_uniform;
mod img_utils;
mod instance;
mod model;
mod resources;
mod texture;
mod vertex;
mod wgpu_ctx;

fn main() -> Result<(), EventLoopError> {
    // 尝试创建或打开日志文件（以追加模式）
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("my_app.log")
        .expect("无法打开日志文件");

    env_logger::builder()
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {}] {}",
                record.level(),
                record.target(), // 这会显示模块路径（如 wgpu_core）
                record.args()
            )
        })
        .filter_level(log::LevelFilter::Debug)
        .filter_module("wgpu_core", log::LevelFilter::Debug)
        .filter_module("wgpu_hal", log::LevelFilter::Debug)
        .filter_module("naga", log::LevelFilter::Error)
        .parse_default_env()
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app)
}
