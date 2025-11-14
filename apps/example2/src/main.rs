use std::fs::OpenOptions;
use std::io::Write;
use winit::event_loop::{self, EventLoop};

use crate::bezier_app::BezierApp;

mod bezier_app;
mod data;

fn main() {
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
    let mut app = BezierApp {
        window: None,
        surface: None,
        device: None,
        queue: None,
        config: None,
        render_pipeline: None,
        control_points_buffer: None,
        bind_group: None,
    };

    event_loop.run_app(&mut app).unwrap();
}
