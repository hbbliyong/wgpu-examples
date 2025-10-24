use crate::img_utils::RgbaImg;
use crate::vertex::{VERTEX_INDEX_LIST, VERTEX_LIST, create_vertex_buffer_layout};
use std::sync::Arc;
use wgpu::MemoryHints::Performance;
use wgpu::Trace;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::wgt::SamplerDescriptor;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct WgpuCtx<'window> {
    surface: wgpu::Surface<'window>,
    surface_config: wgpu::SurfaceConfiguration,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_index_buffer: wgpu::Buffer,
    texture: wgpu::Texture,
    texture_image: RgbaImg,
    texture_size: wgpu::Extent3d,
    sample: wgpu::Sampler,
    bind_group:wgpu::BindGroup
}

impl<'window> WgpuCtx<'window> {
    pub async fn new_async(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                experimental_features: Default::default(),
                memory_hints: Performance,
                trace: Trace::Off,
            })
            .await
            .expect("Failed to create device");

        let mut size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let surface_config = surface.get_default_config(&adapter, width, height).unwrap();
        surface.configure(&device, &surface_config);

       

        let bytes: &[u8] = bytemuck::cast_slice(&VERTEX_LIST);
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytes,
            usage: wgpu::BufferUsages::VERTEX,
        });

        let vertex_index_bytes = bytemuck::cast_slice(&VERTEX_INDEX_LIST);
        let vertex_index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: vertex_index_bytes,
            usage: wgpu::BufferUsages::INDEX,
        });

        let img = RgbaImg::new("assets/example-img.png").unwrap();
        let texture_size = wgpu::Extent3d {
            width: img.width,
            height: img.height,
            depth_or_array_layers: 1, //1表示2d纹理
        };
        let texture = device.create_texture(&wgpu::wgt::TextureDescriptor {
            label: None,
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let sample = device.create_sampler(&SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &texture.create_view(&wgpu::wgt::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sample),
                },
            ],
        });

        let render_pipeline_layout=
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                 label: None,
                  bind_group_layouts: &[&bind_group_layout], 
                  push_constant_ranges: &[] });

         let render_pipeline = Self::create_pipeline(&device, surface_config.format,&render_pipeline_layout);

        WgpuCtx {
            surface,
            surface_config,
            adapter,
            device,
            queue,
            render_pipeline,
            vertex_buffer,
            vertex_index_buffer,
            texture_size,
            texture,
            texture_image: img,
            sample,
            bind_group,
        }
    }

    pub fn new(window: Arc<Window>) -> WgpuCtx<'window> {
        pollster::block_on(WgpuCtx::new_async(window))
    }

    fn create_pipeline(
        device: &wgpu::Device,
        swap_chain_format: wgpu::TextureFormat,
        pipeline_layout:&wgpu::PipelineLayout,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "shader.wgsl"
            ))),
        });

        return device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[create_vertex_buffer_layout()],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(swap_chain_format.into())],
            }),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
    }

    pub fn draw(&mut self) {
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let texutre_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut _r_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(
                    (wgpu::RenderPassColorAttachment {
                        view: &texutre_view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                )],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            _r_pass.set_bind_group(0, &self.bind_group, &[]);
            _r_pass.set_pipeline(&self.render_pipeline);
            _r_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            _r_pass.set_index_buffer(
                self.vertex_index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            _r_pass.draw_indexed(0..VERTEX_INDEX_LIST.len() as u32, 0, 0..1);
        }

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfoBase {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.texture_image.bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.texture_image.width),
                rows_per_image: Some(self.texture_image.height),
            },
            self.texture_size,
        );

        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.surface_config.width = size.width.max(1);
        self.surface_config.height = size.height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
    }
}
