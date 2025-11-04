use crate::camera_controller::CameraController;
use crate::camera_uniform::CameraUniform;
use crate::instance::{Instance, InstanceRaw};
use crate::vertex::{VERTEX_INDEX_LIST, VERTEX_LIST};
use crate::{
    camera,
    model::{DrawModel, Model, ModelVertex, Vertex},
    resources, texture,
};

use std::sync::Arc;

use wgpu::MemoryHints::Performance;
use wgpu::Trace;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
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
    diffuse_textures: texture::Texture,
    //  texture_image: RgbaImg,
    // texture_size: wgpu::Extent3d,
    //sample: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
    camera: camera::Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    pub camera_controller: CameraController,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    depth_texture: texture::Texture,
    obj_model: Model,
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

        // let img = RgbaImg::new("assets/example-img.png").unwrap();
        let diffuse_bytes = include_bytes!("../assets/example-img.png");
        // let texture_size = wgpu::Extent3d {
        //     width: img.width,
        //     height: img.height,
        //     depth_or_array_layers: 1, //1表示2d纹理
        // };
        // let texture = device.create_texture(&wgpu::wgt::TextureDescriptor {
        //     label: None,
        //     size: texture_size,
        //     mip_level_count: 1,
        //     sample_count: 1,
        //     dimension: wgpu::TextureDimension::D2,
        //     format: wgpu::TextureFormat::Rgba8UnormSrgb,
        //     usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        //     view_formats: &[],
        // });

        // let sample = device.create_sampler(&SamplerDescriptor {
        //     address_mode_u: wgpu::AddressMode::ClampToEdge,
        //     address_mode_v: wgpu::AddressMode::ClampToEdge,
        //     address_mode_w: wgpu::AddressMode::ClampToEdge,
        //     ..Default::default()
        // });

        let diffuse_textures =
            texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "example-img.png")
                .unwrap();

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
            label: Some("diffuse_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_textures.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_textures.sampler),
                },
            ],
        });

        let camera = camera::Camera {
            eye: (0.0, -2.0, 4.0).into(),
            // 摄像机看向原点
            target: (0.0, 0.0, 0.0).into(),
            // 定义哪个方向朝上
            up: glam::Vec3::Y,
            // aspect: config.width as f32 / config.height as f32,
            aspect: 4.0 / 3.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let camera_controller = CameraController::new(0.2);
        const NUM_INSTANCES_PRE_ROW: u32 = 10;
        const INSTANCE_DISPLACEMENT: glam::Vec3 = glam::Vec3::new(
            NUM_INSTANCES_PRE_ROW as f32 * 0.5,
            0.0,
            NUM_INSTANCES_PRE_ROW as f32 * 0.5,
        );

        const SPACE_BETWEEN: f32 = 3.0;
        let instances = (0..NUM_INSTANCES_PRE_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PRE_ROW).map(move |x| {
                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PRE_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PRE_ROW as f32 / 2.0);

                    let pos = glam::Vec3 { x: x, y: 0.0, z: z };

                    let rotation = if pos.length().abs() < f32::EPSILON {
                        glam::Quat::from_axis_angle(glam::Vec3::Z, 0.0)
                    } else {
                        glam::Quat::from_axis_angle(pos.normalize(), std::f32::consts::FRAC_PI_4)
                    };
                    Instance { pos, rotation }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &surface_config, "depth_texture");

        let obj_model =
            resources::load_model("cube.obj", &device, &queue, &camera_bind_group_layout)
                .await
                .unwrap();
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline =
            Self::create_pipeline(&device, surface_config.format, &render_pipeline_layout);

        WgpuCtx {
            surface,
            surface_config,
            adapter,
            device,
            queue,
            render_pipeline,
            vertex_buffer,
            vertex_index_buffer,
            // texture_size,
            // texture_image: img,
            diffuse_textures,
            // sample,
            bind_group,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            instances,
            instance_buffer,
            depth_texture,
            obj_model,
        }
    }

    pub fn new(window: Arc<Window>) -> WgpuCtx<'window> {
        pollster::block_on(WgpuCtx::new_async(window))
    }

    fn create_pipeline(
        device: &wgpu::Device,
        swap_chain_format: wgpu::TextureFormat,
        pipeline_layout: &wgpu::PipelineLayout,
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
                buffers: &[ModelVertex::desc(), InstanceRaw::desc()],
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
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
            let mut r_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(
                    (wgpu::RenderPassColorAttachment {
                        view: &texutre_view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                )],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            r_pass.set_bind_group(0, &self.bind_group, &[]);
            r_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            r_pass.set_pipeline(&self.render_pipeline);
            r_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            r_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            r_pass.set_index_buffer(
                self.vertex_index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            // r_pass.draw_indexed(
            //     0..VERTEX_INDEX_LIST.len() as u32,
            //     0,
            //     0..self.instances.len() as _,
            // );

            r_pass.draw_mesh_instanced(&self.obj_model.meshes[0], 0..self.instances.len() as u32);
        }

        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.surface_config.width = size.width.max(1);
        self.surface_config.height = size.height.max(1);
        self.surface.configure(&self.device, &self.surface_config);

        self.depth_texture = texture::Texture::create_depth_texture(
            &self.device,
            &self.surface_config,
            "depth_texture",
        );
    }

    pub fn update(&mut self /*, _dt: instant::Duration */) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
}
