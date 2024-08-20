use crate::{block::quad_buffer::QuadBuffer, camera::ViewProjection, render_thread::RenderThread, shader::Shader, texture::Texture, utils::{bind_group_bundle::BindGroupBundle, index_buffer::IndexBuffer, render_pipeline_bundle::RenderPipelineBundle}, world::PARTS_PER_CHUNK, QUADS};

use super::{chunk_manager::ChunkManager, dynamic_chunk_mesh::DynamicChunkMesh, ChunkTranslation};

pub struct ChunkRenderer {
    textures_bind_group_bundle: BindGroupBundle,
    texture_atlas: Texture,
    light_map: Texture,
    depth_texture: Texture,
    block_render_pipeline_bundle: RenderPipelineBundle,
    index_buffer: IndexBuffer,
    quad_buffer: QuadBuffer,
    pub view_projection: ViewProjection,
    quad_buffer_bind_group_bundle: BindGroupBundle,
    view_projection_bind_group_bundle: BindGroupBundle,
}

impl ChunkRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, surface_config: &wgpu::SurfaceConfiguration) -> anyhow::Result<Self> {
        let texture_atlas = Texture::from_file(device, queue, "./assets/atlases/block_01.png")?;
        let light_map = Texture::from_file(device, queue, "./assets/atlases/light_map.png")?;
        let depth_texture = Texture::create_depth_texture(device, surface_config, "ChunkRenderer_depth_texture");

        let textures_bind_group_bundle = {
            let textures_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ChunkRenderer_textures_bind_group_layout"),
                entries:
                    texture_atlas.bind_group_layout_entries(0, 1).into_iter()
                    .chain(light_map.bind_group_layout_entries(2, 3).into_iter())
                    .collect::<Box<[wgpu::BindGroupLayoutEntry]>>().as_ref()
            });

            let textures_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ChunkRenderer_textures_bind_group_layout"),
                layout: &textures_bind_group_layout,
                entries: 
                    texture_atlas.bind_group_entries(0, 1).into_iter()
                    .chain(light_map.bind_group_entries(2, 3).into_iter())
                    .collect::<Box<[wgpu::BindGroupEntry]>>().as_ref()
            });

            BindGroupBundle::new(textures_bind_group, textures_bind_group_layout)
        };

        let index_buffer = {
            const INDICES: &[u32] = &[0, 1, 2,  1, 3, 2];
            let mut indices = vec![];
            for i in 0..128 * 128 * 128 * 6 as u32 {
                indices.extend(INDICES.iter().cloned().map(|f| f + i * 4))
            }
            IndexBuffer::new_init(device, &indices)
        };

        let quad_buffer = QuadBuffer::new(&device, &QUADS);
        let view_projection = ViewProjection::new(&device);

        let quad_buffer_bind_group_bundle = {
            let quad_buffer_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ChunkRenderer_quad_buffer_bind_group_layout"),
                entries: &[quad_buffer.bind_group_layout_entry(0)],
            });
            
            let quad_buffer_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ChunkRenderer_quad_buffer_bind_group"),
                entries: &[quad_buffer.bind_group_entry(0)],
                layout: &quad_buffer_bind_group_layout,
            });

            BindGroupBundle::new(quad_buffer_bind_group, quad_buffer_bind_group_layout)
        };

        let view_projection_bind_group_bundle = {
            let view_projection_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ChunkRenderer_view_projection_bind_group_layout"),
                entries: &[view_projection.bind_group_layout_entry(0)],
            });

            let view_projection_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ChunkRenderer_view_projection_bind_group"),
                entries: &[view_projection.bind_group_entry(0)],
                layout: &view_projection_bind_group_layout,
            });

            BindGroupBundle::new(view_projection_bind_group, view_projection_bind_group_layout)
        };

        let block_render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ChunkRenderer_block_render_pipeline_layout"),
            push_constant_ranges: &[],
            bind_group_layouts: &[
                textures_bind_group_bundle.layout(),
                quad_buffer_bind_group_bundle.layout(),
                view_projection_bind_group_bundle.layout(),
                DynamicChunkMesh::get_or_init_face_buffer_bind_group_layout(device),
                ChunkTranslation::get_or_init_bind_group_layout(device),
            ]
        });

        let block_render_pipeline_shader = Shader::from_file(device, "./src/shaders/model.wgsl")?;

        let block_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ChunkRenderer_block_render_pipeline"),
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
                module: block_render_pipeline_shader.module(),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                entry_point: "fs_main",
                module: block_render_pipeline_shader.module(),
                targets: &[Some(
                    wgpu::ColorTargetState {
                        format: surface_config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::all()
                    }
                )],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            layout: Some(&block_render_pipeline_layout),
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

        let block_render_pipeline_bundle = RenderPipelineBundle::new(block_render_pipeline, block_render_pipeline_layout);

        Ok(Self { textures_bind_group_bundle, texture_atlas, light_map, block_render_pipeline_bundle, index_buffer, quad_buffer, view_projection, quad_buffer_bind_group_bundle, view_projection_bind_group_bundle, depth_texture })
    }

    pub fn render(&self, device: &wgpu::Device, queue: &wgpu::Queue, chunk_manager: &mut ChunkManager, render_thread: &mut RenderThread) {
        chunk_manager.collect_meshing_outputs(device, queue);

        let meshes = chunk_manager.get_ready_meshes();
        let view_projection = self.view_projection.clone();
        let textures_bind_group_bundle = self.textures_bind_group_bundle.clone();
        let quad_buffer_bind_group_bundle = self.quad_buffer_bind_group_bundle.clone();
        let view_projection_bind_group_bundle = self.view_projection_bind_group_bundle.clone();
        let depth_texture = self.depth_texture.clone();
        let render_pipeline_bundle = self.block_render_pipeline_bundle.clone();
        let index_buffer = self.index_buffer.clone();

        render_thread.push_render(move |queue, encoder, view| {
            view_projection.update_buffer(queue);

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ChunkRenderBundle_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 123.0 / 255.0, g: 164.0 / 255.0, b: 1.0, a: 1.0 }),
                        store: wgpu::StoreOp::Store
                    }
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                    view: depth_texture.view()
                }),
                timestamp_writes: None,
                occlusion_query_set: None
            });

            render_pass.set_pipeline(render_pipeline_bundle.render_pipeline());
            render_pass.set_bind_group(0, textures_bind_group_bundle.bind_group(), &[]);
            render_pass.set_bind_group(1, quad_buffer_bind_group_bundle.bind_group(), &[]);
            render_pass.set_bind_group(2, view_projection_bind_group_bundle.bind_group(), &[]);
            render_pass.set_index_buffer(index_buffer.buffer().slice(..), IndexBuffer::FORMAT);

            for mesh in meshes.iter() {
                if !mesh.parts_meshed.iter().cloned().all(|f| f) { continue; }

                render_pass.set_bind_group(3, mesh.face_buffer_bind_group(), &[]);
                render_pass.set_bind_group(4, mesh.translation().bind_group(), &[]);

                render_pass.multi_draw_indexed_indirect(mesh.indirect_buffer(), 0, PARTS_PER_CHUNK as u32);
            }
        });
    }
}