use crate::wgpu_ctx::{self, WgpuCtx};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window,WindowId};
use std::sync::Arc;


#[derive(Default)]
pub struct App<'window>{
    window:Option<Arc<Window>>,
    wgpu_ctx:Option<WgpuCtx<'window>>
}

impl<'window> ApplicationHandler for App<'window>{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
       if self.window.is_none(){
        let win_attr=Window::default_attributes().with_title("title");
        let window=Arc::new(event_loop.create_window(win_attr)
        .expect("create window err."),);
    
    let wgpu_ctx=WgpuCtx::new(window.clone());
    self.wgpu_ctx=Some(wgpu_ctx);
    self.window=Some(window);
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
            WindowEvent::Resized(new_size)=>{
                if let (Some(window),Some(wgpu_ctx))=
                (self.window.as_ref(),self.wgpu_ctx.as_mut()){
                    wgpu_ctx.resize(new_size);
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested=>{
                if let Some(wgpu_ctx)=self.wgpu_ctx.as_mut(){
                    wgpu_ctx.draw();
                }
            }
            _ => (),
        }
    }
}