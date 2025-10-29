use crate::wgpu_ctx::{self, WgpuCtx};

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

#[derive(Default)]
pub struct App<'window> {
    window: Option<Arc<Window>>,
    wgpu_ctx: Option<WgpuCtx<'window>>,
}

impl<'window> App<'window> {
    /// 请求重绘    
    fn request_redraw(&self) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes().with_title("title");
            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("create window err."),
            );

            let wgpu_ctx = WgpuCtx::new(window.clone());
            self.wgpu_ctx = Some(wgpu_ctx);
            self.window = Some(window);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(new_size) => {
                if let (Some(window), Some(wgpu_ctx)) =
                    (self.window.as_ref(), self.wgpu_ctx.as_mut())
                {
                    wgpu_ctx.resize(new_size);
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => {
                if event.physical_key == PhysicalKey::Code(KeyCode::Space) {
                    if event.state.is_pressed() {
                        println!("space entre");
                    }
                }
                self.wgpu_ctx
                    .as_mut()
                    .unwrap()
                    .camera_controller
                    .process_events(&event);
            }
            WindowEvent::RedrawRequested => {
                if let Some(wgpu_ctx) = self.wgpu_ctx.as_mut() {
                    wgpu_ctx.update();
                    wgpu_ctx.draw();
                }
                self.request_redraw();
            }

            _ => (),
        }
    }
}
