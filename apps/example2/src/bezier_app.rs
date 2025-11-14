use std::{default, sync::Arc};

use crate::data::ControlPoints;
use wgpu::{RequestAdapterOptions, util::DeviceExt};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};
pub struct BezierApp {
    pub window: Option<Arc<Window>>,
    pub surface: Option<wgpu::Surface<'static>>,
    pub device: Option<wgpu::Device>,
    pub queue: Option<wgpu::Queue>,
    pub config: Option<wgpu::SurfaceConfiguration>,
    pub render_pipeline: Option<wgpu::RenderPipeline>,
    pub control_points_buffer: Option<wgpu::Buffer>,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl ApplicationHandler for BezierApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("Bezier Curve");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(&window).unwrap();
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("Failed to find an appropriate adapter.");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::wgt::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::default(),
                required_limits: wgpu::Limits::defaults(),
                memory_hints: wgpu::MemoryHints::Performance,
                ..Default::default()
            }))
            .expect("Failed to create device.");

        let mut size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0];
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 3,
        };
        surface.configure(&device, &surface_config);

        let control_points = ControlPoints {
            points: [
                [-0.5, -0.5], // P0: 起点
                [-0.2, 0.8],  // P1: 控制点
                [0.5, -0.7],  // P2: 控制点
                [0.7, 0.6],   // P3: 终点
            ],
        };

        let control_points_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Control Points Buffer"),
            contents: bytemuck::cast_slice(&[control_points]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        // 着色器代码 (WGSL)
        const BEZIER_SHADER_SRC: &str = r#"
// 定义与控制点结构体匹配的Uniform Buffer
struct ControlPoints {
    points: array<vec2f, 4>,
};
@group(0) @binding(0)
var<uniform> ctrl_pts: ControlPoints;

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec3f,
};

// 三次贝塞尔曲线公式
fn bezier(t: f32) -> vec2f {
    let one_minus_t = 1.0 - t;
    let t2 = t * t;
    let t3 = t2 * t;
    let omt2 = one_minus_t * one_minus_t;
    let omt3 = omt2 * one_minus_t;

    return omt3 * ctrl_pts.points[0]
         + 3.0 * one_minus_t * t * ctrl_pts.points[1]
         + 3.0 * omt2 * t2 * ctrl_pts.points[2]
         + t3 * ctrl_pts.points[3];
}

@vertex
fn vs_main(@builtin(vertex_index) vert_index: u32) -> VertexOutput {
    let t = f32(vert_index) / 99.0; // 将顶点索引映射到 [0, 1] 区间
    let position = bezier(t);

    var output: VertexOutput;
    output.clip_position = vec4f(position, 0.0, 1.0);
    // 简单的颜色渐变
    output.color = vec3f(t, 1.0 - t, 0.5);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4f {
    return vec4f(input.color, 1.0);
}
"#;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Bezier Shader"),
            source: wgpu::ShaderSource::Wgsl(BEZIER_SHADER_SRC.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Control Points Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Control Points Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: control_points_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Bezier Curve Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineStrip, //使用线带绘制曲线
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),

            multiview: None,
            cache: None,
        });

        // 将创建的资源存入App结构体
        self.window = Some(window.clone());
        // 使用 `wgpu::Surface<'static>` 来满足生命周期要求
        self.surface = Some(unsafe { std::mem::transmute(surface) });
        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(surface_config);
        self.render_pipeline = Some(render_pipeline);
        self.control_points_buffer = Some(control_points_buffer);
        self.bind_group = Some(bind_group);
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
            WindowEvent::RedrawRequested => {
                self.render().expect("Render failed");
            }
            _ => {}
        }
    }
}

impl BezierApp {
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let surface = self.surface.as_ref().unwrap();
        let device = self.device.as_ref().unwrap();
        let queue = self.queue.as_ref().unwrap();
        let render_pipeline = self.render_pipeline.as_ref().unwrap();
        let config = self.config.as_ref().unwrap();

        let frame = surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::wgt::TextureViewDescriptor::default());

        let mut encoder =
            device.create_command_encoder(&wgpu::wgt::CommandEncoderDescriptor::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.05,
                            b: 0.05,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_pipeline(render_pipeline);
            render_pass.draw(0..100, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        Ok(())
    }
}
