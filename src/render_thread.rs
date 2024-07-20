use std::{collections::VecDeque, sync::{mpsc::{channel, Receiver, Sender}, Arc, Condvar, Mutex}};
use wgpu::util::DeviceExt;
use cgmath::{Matrix4, Vector2};

use crate::{block::quad_buffer::QuadBuffer, camera::{Camera, CameraTemp, ViewProjection}, game_window::GameWindow, texture::{Texture, TextureAtlas}, world::{chunk::{chunk_mesh_map::ChunkMeshMap, chunk_part::chunk_part_mesher::MeshingOutput, dynamic_chunk_mesh::DynamicChunkMesh}, PARTS_PER_CHUNK}, QUADS};

pub struct RenderThread {
    event_sender: Arc<Sender<RenderEvent>>,
    needed_chunk_parts_receiver: Receiver<(Vector2<i32>, usize)>,
    pub work_done_receiver: Receiver<()>,
    pub rendering_condvar_pair: Arc<(Mutex<bool>, Condvar)>,
    pub can_render_condvar_pair: Arc<(Mutex<bool>, Condvar)>,
}

pub enum RenderEvent {
    Render(RenderArgs),
    UpdateViewProjection(Matrix4<f32>),
}

pub struct RenderArgs {
    pub surface: Arc<wgpu::Surface<'static>>,
    pub game_window: GameWindow,
    pub aspect: f32,
    pub meshes: Box<[DynamicChunkMesh]>,
}

impl RenderThread {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, surface_config: wgpu::SurfaceConfiguration) -> Self {
        let (event_sender, event_receiver) = channel();
        let (needed_chunk_parts_sender, needed_chunk_parts_receiver) = channel();
        let (work_done_sender, work_done_receiver) = channel();
        let rendering_condvar_pair = Arc::new((Mutex::new(false), Condvar::new()));
        let rendering_condvar_pair_clone = rendering_condvar_pair.clone();
        let can_render_condvar_pair = Arc::new((Mutex::new(false), Condvar::new()));
        let can_render_condvar_pair_clone = rendering_condvar_pair.clone();
        rayon::spawn(move || Self::run(event_receiver, needed_chunk_parts_sender, device, queue, surface_config, work_done_sender, rendering_condvar_pair_clone, can_render_condvar_pair_clone));
        Self { event_sender: Arc::new(event_sender), needed_chunk_parts_receiver, work_done_receiver, rendering_condvar_pair, can_render_condvar_pair }
    }

    pub fn render(&self, render_args: RenderArgs) -> Result<(), std::sync::mpsc::SendError<RenderEvent>> {
        self.event_sender.send(RenderEvent::Render(render_args))
    }

    pub fn update_view_projection(&self, camera: &dyn Camera, aspect: f32) -> Result<(), std::sync::mpsc::SendError<RenderEvent>> {
        self.event_sender.send(RenderEvent::UpdateViewProjection(camera.build_view_projection_matrix(aspect)))
    }

    fn run(event_receiver: Receiver<RenderEvent>, needed_chunk_parts_sender: Sender<(Vector2<i32>, usize)>, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, surface_config: wgpu::SurfaceConfiguration, work_done_sender: Sender<()>, rendering_condvar_pair: Arc<(Mutex<bool>, Condvar)>, can_render_condvar_pair: Arc<(Mutex<bool>, Condvar)>) {
        let mut render_thread = RenderThreadInner::new(device, queue, surface_config, needed_chunk_parts_sender, work_done_sender, rendering_condvar_pair, can_render_condvar_pair);
        for event in event_receiver.iter() {
            match event {
                RenderEvent::Render(args) => render_thread.render(args),
                RenderEvent::UpdateViewProjection(matrix) => render_thread.view_projection.update_from_matrix(matrix),
            }
        }
    }
}

struct RenderThreadInner {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    depth_texture: Texture,
    texture_atlas: TextureAtlas,
    index_buffer: wgpu::Buffer,
    quad_buffer: QuadBuffer,
    view_projection: ViewProjection,
    render_pipeline: wgpu::RenderPipeline,
    needed_chunk_parts_sender: Sender<(Vector2<i32>, usize)>,
    work_done_sender: Sender<()>,
    rendering_condvar_pair: Arc<(Mutex<bool>, Condvar)>,
    can_render_condvar_pair: Arc<(Mutex<bool>, Condvar)>,
}

impl RenderThreadInner {
    fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, surface_config: wgpu::SurfaceConfiguration, needed_chunk_parts_sender: Sender<(Vector2<i32>, usize)>, work_done_sender: Sender<()>, rendering_condvar_pair: Arc<(Mutex<bool>, Condvar)>, can_render_condvar_pair: Arc<(Mutex<bool>, Condvar)>) -> Self {
        let depth_texture = Texture::create_depth_texture(&device, &surface_config, "depth texture");
        let texture_atlas = TextureAtlas::new("./assets/atlases/block_01.png", &device, &queue);

        const INDICES: &[u32] = &[0, 1, 2,  1, 3, 2];
        let mut indices = vec![];
        for i in 0..128 * 128 * 128 * 6 as u32 {
            indices.extend(INDICES.iter().cloned().map(|f| f + i * 4))
        }
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index_buffer"),
            usage: wgpu::BufferUsages::INDEX,
            contents: bytemuck::cast_slice(&indices)
        });

        let quad_buffer = QuadBuffer::new(&device, &QUADS);

        let view_projection = ViewProjection::new(&device);

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render_pipeline_layout"),
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

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
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
            layout: Some(&render_pipeline_layout),
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

        Self { device, queue, depth_texture, texture_atlas, index_buffer, quad_buffer, view_projection, render_pipeline, needed_chunk_parts_sender, work_done_sender, rendering_condvar_pair, can_render_condvar_pair }
    }

    fn render(&mut self, args: RenderArgs) {
        {
            let mut is_rendering = self.rendering_condvar_pair.0.lock().unwrap();
            *is_rendering = true;
        }

        let output = args.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render_encoder")} );
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
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
            
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, self.view_projection.bind_group(), &[]);
            render_pass.set_bind_group(2, self.quad_buffer.bind_group(), &[]);
            render_pass.set_bind_group(3, self.texture_atlas.bind_group(), &[]);
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            for mesh in args.meshes.iter() {
                if !mesh.parts_meshed.iter().cloned().all(|f| f) { continue; }
                
                render_pass.set_bind_group(1, mesh.face_buffer_bind_group(), &[]);
                render_pass.set_bind_group(4, mesh.translation().bind_group(), &[]);

                render_pass.multi_draw_indexed_indirect(mesh.indirect_buffer(), 0, PARTS_PER_CHUNK as u32);
            }
        }

        self.view_projection.update_buffer(&self.queue);
        
        self.queue.submit(std::iter::once(encoder.finish()));

        let rendering_condvar_pair = self.rendering_condvar_pair.clone();
        self.queue.on_submitted_work_done(move || {
            let mut is_rendering = rendering_condvar_pair.0.lock().unwrap();
            *is_rendering = false;
            rendering_condvar_pair.1.notify_all();
        });

        args.game_window.window().pre_present_notify();
        output.present();
        self.work_done_sender.send(());
    }
}