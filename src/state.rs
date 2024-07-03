use std::sync::Arc;

use cgmath::{Point3, Vector2, Vector3};
use wgpu::{util::DeviceExt, Device, Features, Queue};
use winit::{event::{DeviceEvent, Event, WindowEvent}, event_loop::EventLoop};

use crate::{block::quad_buffer::QuadBuffer, game_window::GameWindow, setttings::Settings, texture::Texture, BLOCK_MODEL_VARIANTS, QUADS};
const S: usize = 2;
pub struct State<'a> {
    game_window: GameWindow,
    settings: Settings,
    settings_last_modified: std::time::SystemTime,
    surface: wgpu::Surface<'a>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    surface_config: wgpu::SurfaceConfiguration,
    aspect_ratio: f32,
    settings_path: std::path::PathBuf,
    pipeline: wgpu::RenderPipeline,
    face_buffer: wgpu::Buffer,
    quad_buffer: QuadBuffer,
    face_buffer_bind_group: wgpu::BindGroup,
    model_buffer_bind_group: wgpu::BindGroup,
    view_projection_uniform: crate::camera::ViewProjectionUniform,
    view_projection_bind_group: wgpu::BindGroup,
    view_projection_buffer: wgpu::Buffer,
    camera: crate::camera::CameraTemp,
    index_buffer: wgpu::Buffer,
    texture_atlas: Texture,
    texture_atlas_bind_group: wgpu::BindGroup,
    faces_num: usize,
    depth_texture: Texture,
    translation_bind_groups: Vec<wgpu::BindGroup>,
}

impl<'a> State<'a> {
    pub async fn new(settings_path: &std::path::Path) -> anyhow::Result<(Self, EventLoop<()>)> {
        let settings = Settings::from_file(settings_path)?;
        let settings_last_modified = std::fs::metadata(settings_path)?.modified()?;

        let (game_window, event_loop) = GameWindow::new(&settings)?;
        let size = settings.resolution;

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        let surface = instance.create_surface(game_window.window_arc())?;

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
            compatible_surface: Some(&surface),
            ..Default::default()
        }).await.expect("Couldn't create adapter");

        let required_limits = wgpu::Limits {
            max_bind_groups: 8,
            max_buffer_size: i32::MAX as u64,
            ..Default::default()
        };
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("device"),
            required_features: Features::BUFFER_BINDING_ARRAY | Features::STORAGE_RESOURCE_BINDING_ARRAY,
            required_limits,
            ..Default::default()
        }, None).await?;

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|p| p.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]); // TODO change

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.0,
            height: size.1,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_capabilities.alpha_modes[0], // TODO change
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };

        surface.configure(&device, &surface_config);
        let aspect_ratio = surface_config.width as f32 / surface_config.height as f32;
        

        

        let model_buffer_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: Some(std::num::NonZeroU32::new(1).unwrap()),
                    ty: wgpu::BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: None,
                        ty: wgpu::BufferBindingType::Storage { read_only: true }
                    },
                    visibility: wgpu::ShaderStages::VERTEX
                }
            ]
        });
        // let mut baked_block_models = crate::block::asset_loader::load_models("./assets/models").unwrap();
        // let (block_map, block_list, block_models, quads) = crate::block::asset_loader::load_blocks("./assets/blocks", &mut baked_block_models).unwrap();
        let models = BLOCK_MODEL_VARIANTS.lock().unwrap().get_quad_block_models(&crate::block::Block { id: 0, name: "cobblestone".to_string(), block_state: crate::block::block_state::BlockState::new() }).unwrap();
        let quad_buffer = QuadBuffer::new(&device, &QUADS.lock().unwrap());

        let mut faces = vec![];
        let translation_buffer_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0, 
                count: None,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                visibility: wgpu::ShaderStages::VERTEX
            }]
        });
        for y in 0..32 {
            for z in 0..32 {
                for x in 0..32 {
                    for model in models.iter() {
                        assert!(model.quad_indices.len() == model.texture_indices.len());
                        for (quad_i, texture_i) in model.quad_indices.iter().zip(model.texture_indices.iter()) {
                            faces.push(crate::block::model::Face {
                                block_position: [x, y, z],
                                lighting: [crate::block::light::LightLevel::new(0).unwrap(); 4],
                                quad_index: *quad_i,
                                texture_index: *texture_i
                            }.pack());
                        }
                    }
                }
            }
        }
        let mut translation_bind_groups = vec![];
        for tz in 0..S as i32 {
            for tx in 0..S as i32 {   
                let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&[tx * 32, tz * 32]),
                    usage: wgpu::BufferUsages::UNIFORM
                });

                translation_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding()
                    }],
                    layout: &translation_buffer_bind_group_layout
                }));
            }
        }


        let faces_num = faces.len();

        const INDICES: &[u32] = &[0, 1, 2,  1, 3, 2];
        let mut indices = vec![];
        for i in 0..128 * 128 * 128 * 6 as u32 {
            indices.extend(INDICES.iter().cloned().map(|f| f + i * 4))
        }
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            usage: wgpu::BufferUsages::INDEX,
            contents: bytemuck::cast_slice(&indices)
        });
        
        let face_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            usage: wgpu::BufferUsages::STORAGE,
            contents: bytemuck::cast_slice(&faces)
        });

        let face_buffer_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: None,
                        ty: wgpu::BufferBindingType::Storage { read_only: true }
                    },
                    visibility: wgpu::ShaderStages::VERTEX
                }
            ]
        });

        let face_buffer_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &face_buffer_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(face_buffer.as_entire_buffer_binding())
                }
            ]
        });

        let quad_buffer_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &model_buffer_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(quad_buffer.buffer().as_entire_buffer_binding())
                }
            ]
        });

        let camera = crate::camera::CameraTemp::new();
        let mut view_projection_uniform = crate::camera::ViewProjectionUniform::new();
        view_projection_uniform.update(&camera, aspect_ratio);

        let view_projection_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&[view_projection_uniform])
        });

        let view_projection_bind_group_layout = crate::camera::ViewProjectionUniform::create_bind_group_layout(&device);

        let view_projection_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &view_projection_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(view_projection_buffer.as_entire_buffer_binding())
                }
            ]
        });
        let texture_atlas = crate::texture::Texture::from_bytes(&device, &queue, include_bytes!("../assets/atlases/debug.png"), "texture_atlas")?;
        let texture_atlas_bind_group_layout = Texture::texture_atlas_bind_group_layout(&device);
        let texture_atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture_atlas_bind_group"),
            layout: &texture_atlas_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_atlas.view)
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_atlas.sampler)
                }
            ]
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &view_projection_bind_group_layout,
                &face_buffer_bind_group_layout,
                &model_buffer_bind_group_layout,
                &texture_atlas_bind_group_layout,
                &translation_buffer_bind_group_layout
            ],
            push_constant_ranges: &[],
        });

        let shaders = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/model.wgsl").into())
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            depth_stencil: Some(wgpu::DepthStencilState {
                bias: wgpu::DepthBiasState::default(),
                depth_compare: wgpu::CompareFunction::Less,
                depth_write_enabled: true,
                format: Texture::DEPTH_FORMAT,
                stencil: wgpu::StencilState::default()
            }),
            vertex: wgpu::VertexState {
                buffers: &[],
                entry_point: "vs_main",
                module: &shaders
            },
            fragment: Some(wgpu::FragmentState {
                entry_point: "fs_main",
                module: &shaders,
                targets: &[Some(
                    wgpu::ColorTargetState {
                        format: surface_config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::all()
                    }
                )]
            }),
            layout: Some(&pipeline_layout),
            multisample: wgpu::MultisampleState {
                alpha_to_coverage_enabled: false,
                count: 1,
                mask: !0
            },
            multiview: None,
            primitive: wgpu::PrimitiveState {
                conservative: false,
                cull_mode: Some(wgpu::Face::Back),
                front_face: wgpu::FrontFace::Ccw,
                polygon_mode: wgpu::PolygonMode::Fill,
                strip_index_format: None,
                topology: wgpu::PrimitiveTopology::TriangleList,
                unclipped_depth: false
            },
        });
        
        let depth_texture = Texture::create_depth_texture(&device, &surface_config, "depth_texture");

        Ok((
            Self {
                game_window,
                settings,
                settings_last_modified,
                device: Arc::new(device),
                queue: Arc::new(queue),
                surface,
                surface_config,
                aspect_ratio,
                settings_path: settings_path.to_owned(),
                pipeline,
                face_buffer,
                quad_buffer,
                face_buffer_bind_group,
                model_buffer_bind_group: quad_buffer_bind_group,
                view_projection_uniform,
                view_projection_bind_group,
                view_projection_buffer,
                camera,
                index_buffer,
                texture_atlas,
                texture_atlas_bind_group,
                faces_num,
                depth_texture,
                translation_bind_groups
            }, 
            event_loop
        ))
    }

    pub fn resize_window(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.height > 0 && new_size.width > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
            self.aspect_ratio = self.width() as f32 / self.height() as f32;
        }
    }

    pub fn height(&self) -> u32 {
        self.surface_config.height
    }

    pub fn width(&self) -> u32 {
        self.surface_config.width
    }

    pub fn run<T: Into<std::path::PathBuf>>(settings_path: T) -> anyhow::Result<()> {
        let settings_path = settings_path.into();
        let (mut state, event_loop) = pollster::block_on(Self::new(&settings_path))?;
        let mut last_render_instant = std::time::Instant::now();
        event_loop.run(move |event, elwt| {
            elwt.set_control_flow(winit::event_loop::ControlFlow::Poll);
            if let Ok(settings_file_metadata) = std::fs::metadata(&settings_path) {
                if let Ok(settings_last_modified_new) = settings_file_metadata.modified() {
                    if state.settings_last_modified != settings_last_modified_new {
                        if let Ok(new_settings) = Settings::from_file(&settings_path) {
                            state.settings = new_settings;
                            state.settings_last_modified = settings_last_modified_new;
                        }
                    }
                }
            }
 
            match event {
                Event::WindowEvent { window_id, event } => {
                    match event {
                        WindowEvent::CloseRequested | WindowEvent::KeyboardInput { event: winit::event::KeyEvent { physical_key: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape), .. }, .. } => {
                            elwt.exit();
                        },
                        WindowEvent::RedrawRequested => {
                            // state.game_window.window().set_cursor_position(winit::dpi::PhysicalPosition::new(state.width() / 2, state.height() / 2)).unwrap();
                            println!("{:.1?} fps", 1.0 / last_render_instant.elapsed().as_secs_f64());
                            last_render_instant = std::time::Instant::now();
                            state.render();
                        }
                        WindowEvent::Resized(new_size) => {
                            state.resize_window(new_size);
                        },
                        WindowEvent::KeyboardInput { event, .. } => {
                            if let winit::keyboard::PhysicalKey::Code(key_code) = event.physical_key {
                                state.camera.handle_keyboard_input(key_code, event.state);
                            }
                        }
                        _ => ()
                    }
                },
                Event::DeviceEvent { event, .. } => {
                    if let DeviceEvent::MouseMotion { delta: (delta_x, delta_y) } = event {
                        state.camera.handle_mouse_movement(delta_x as f32, delta_y as f32);
                    }
                }
                Event::AboutToWait => {
                    state.game_window.window().request_redraw();
                }
                _ => ()
            }
        })?;
        Ok(())
    }

    pub fn render(&mut self) {
        let output = self.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render_encoder")} );
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("model_translucent_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store
                    }
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                    view: &self.depth_texture.view
                }),
                timestamp_writes: None,
                occlusion_query_set: None
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.view_projection_bind_group, &[]);
            render_pass.set_bind_group(1, &self.face_buffer_bind_group, &[]);
            render_pass.set_bind_group(2, &self.model_buffer_bind_group, &[]);
            render_pass.set_bind_group(3, &self.texture_atlas_bind_group, &[]);

            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            
            for tz in 0..S {
                for tx in 0..S {
                    render_pass.set_bind_group(4, &self.translation_bind_groups[tx + tz * S], &[]);
                    render_pass.draw_indexed(0..self.faces_num as u32 * 6, 0, 0..1);
                }
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        self.game_window.window().pre_present_notify();
        output.present();


        self.view_projection_uniform.update(&self.camera, self.aspect_ratio);

        self.queue.write_buffer(&self.view_projection_buffer, 0, bytemuck::cast_slice(&[self.view_projection_uniform]));
        self.camera.update();
    }
}