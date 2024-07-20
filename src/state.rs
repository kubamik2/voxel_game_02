use std::{collections::VecDeque, sync::{Arc, Condvar, Mutex}};

use cgmath::{Point3, Vector2, Vector3};
use wgpu::{util::DeviceExt, Device, Features, Queue};
use winit::{event::{DeviceEvent, Event, WindowEvent}, event_loop::EventLoop};

use crate::{block::quad_buffer::QuadBuffer, camera::Camera, game_window::GameWindow, interval::{Interval, IntervalThread}, render_thread::{RenderArgs, RenderEvent, RenderThread}, settings::Settings, texture::{Texture, TextureAtlas}, world::{chunk::{chunk_manager::ChunkManager, chunk_mesh_map::ChunkMeshMap, chunk_part::{chunk_part_mesher::ChunkPartMesher, expanded_chunk_part::ExpandedChunkPart}, dynamic_chunk_mesh::DynamicChunkMesh}, PARTS_PER_CHUNK}, BLOCK_MODEL_VARIANTS, QUADS};

const S: usize = 1;

pub struct State {
    game_window: GameWindow,

    settings: Settings,
    settings_last_modified: std::time::SystemTime,

    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Arc<wgpu::Surface<'static>>,

    surface_config: wgpu::SurfaceConfiguration,
    aspect_ratio: f32,
    settings_path: std::path::PathBuf,
    pipeline: wgpu::RenderPipeline,

    quad_buffer: QuadBuffer,

    view_projection: crate::camera::ViewProjection,

    camera: crate::camera::CameraTemp,

    index_buffer: wgpu::Buffer,

    texture_atlas: TextureAtlas,
    depth_texture: Texture,

    chunk_manager: ChunkManager,

    now: std::time::Instant,
}

impl State {
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
            required_features: Features::BUFFER_BINDING_ARRAY | Features::STORAGE_RESOURCE_BINDING_ARRAY | Features::MULTI_DRAW_INDIRECT,
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
        
        let quad_buffer = QuadBuffer::new(&device, &QUADS);


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
        
        let camera = crate::camera::CameraTemp::new();
        let mut view_projection = crate::camera::ViewProjection::new(&device);
        view_projection.update(&camera, aspect_ratio);

        let texture_atlas = TextureAtlas::new("./assets/atlases/block_01.png", &device, &queue);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                crate::camera::ViewProjection::get_or_init_bind_group_layout(&device),
                crate::world::chunk::dynamic_chunk_mesh::DynamicChunkMesh::get_or_init_face_buffer_bind_group_layout(&device),
                QuadBuffer::get_or_init_bind_group_layout(&device),
                TextureAtlas::get_or_init_bind_group_layout(&device),
                crate::world::chunk::ChunkTranslation::get_or_init_bind_group_layout(&device),
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
                module: &shaders,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
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
                )],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
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
        let device = Arc::new(device);
        let queue = Arc::new(queue);
        
        let chunk_manager = ChunkManager::new(device.clone(), queue.clone(), 10, 12, 12);

        Ok((
            Self {
                game_window,
                settings,
                settings_last_modified,
                device,
                queue,
                surface: Arc::new(surface),
                surface_config,
                aspect_ratio,
                settings_path: settings_path.to_owned(),
                pipeline,
                quad_buffer,
                view_projection,
                camera,
                index_buffer,
                texture_atlas,
                depth_texture,
                chunk_manager,
                now: std::time::Instant::now(),
            }, 
            event_loop,
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

    fn update_60hz(&mut self) {

    }

    pub fn run<T: Into<std::path::PathBuf>>(settings_path: T) -> anyhow::Result<()> {
        let settings_path = settings_path.into();
        let (mut state, event_loop) = pollster::block_on(Self::new(&settings_path))?;
        
        let mut last_render_instant = std::time::Instant::now();

        let render_thread = RenderThread::new(state.device.clone(), state.queue.clone(), state.surface_config.clone());
        let rendering_condvar_pair = render_thread.rendering_condvar_pair.clone();
        let mut interval_300hz = Interval::new(std::time::Duration::from_secs_f32(1.0 / 300.0));
        let mut interval_60hz = Interval::new(std::time::Duration::from_secs_f32(1.0 / 60.0));

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

            if render_thread.work_done_receiver.try_recv().is_ok() {
                state.game_window.window().request_redraw();
            }

            interval_300hz.tick(|| {
                state.chunk_manager.update();
            });

            interval_60hz.tick(|| {
                state.chunk_manager.insert_chunks_around_player(Vector2::new(0, 0));
            });

            match event {
                Event::WindowEvent { window_id, event } => {
                    match event {
                        WindowEvent::CloseRequested | WindowEvent::KeyboardInput { event: winit::event::KeyEvent { physical_key: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape), .. }, .. } => {
                            elwt.exit();
                        },
                        WindowEvent::RedrawRequested => {
                            if state.game_window.window().has_focus() {
                                state.game_window.window().set_cursor_position(winit::dpi::PhysicalPosition::new(state.width() / 2, state.height() / 2)).unwrap();
                            }
                            state.camera.update();
                            render_thread.update_view_projection(&state.camera, state.aspect_ratio);
                            state.chunk_manager.collect_meshing_outputs();
                            let meshes = state.chunk_manager.get_ready_meshes();
                            if meshes.len() == (2 * state.chunk_manager.render_distance() as usize - 3) * (2 * state.chunk_manager.render_distance() as usize - 3) {
                                dbg!(state.now.elapsed());
                            }
                            // state.chunk_manager.print_chunk_generation_stages();
                            render_thread.render(RenderArgs { surface: state.surface.clone(), game_window: state.game_window.clone(), aspect: state.aspect_ratio, meshes });
                            // println!("{:.1?} fps", 1.0 / last_render_instant.elapsed().as_secs_f64());
                            last_render_instant = std::time::Instant::now();
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
                },
                _ => ()
            }
        })?;
        Ok(())
    }
}
