use cgmath::Vector2;

use crate::{block::model::{Face, FacePacked}, world::PARTS_PER_CHUNK};

use super::chunk_part::{chunk_part_mesher::MeshingOutput, CHUNK_SIZE};

static FACE_BUFFER_BIND_GROUP_LAYOUT: std::sync::OnceLock<wgpu::BindGroupLayout> = std::sync::OnceLock::new();

// Model mesh that is divided into buckets containing each chunk's faces
pub struct DynamicChunkModelMesh {
    pub face_buffer: wgpu::Buffer,
    pub face_buffer_bind_group: wgpu::BindGroup,
    pub indirect_buffer: wgpu::Buffer,
    pub face_bucket_elements: [u32; PARTS_PER_CHUNK as usize]
}

impl DynamicChunkModelMesh {
    pub const MIN_BUCKET_SIZE: u32 = Self::MIN_BUCKET_ELEMENTS * std::mem::size_of::<Face>() as u32;
    pub const MIN_BUCKET_ELEMENTS: u32 = 200_000;

    pub fn new(device: &wgpu::Device) -> Self {
        let face_buffer = Self::create_face_buffer(device, (Self::MIN_BUCKET_SIZE as usize * PARTS_PER_CHUNK) as u64);
        let indirect_buffer = Self::create_indirect_buffer(device);
        
        let face_bucket_elements = std::array::from_fn(|_| Self::MIN_BUCKET_ELEMENTS);
        let face_buffer_bind_group_layout = Self::get_or_init_face_buffer_bind_group_layout(device);
        let face_buffer_bind_group = Self::create_bind_group(device, face_buffer_bind_group_layout, &face_buffer);

        Self { face_buffer, indirect_buffer, face_bucket_elements, face_buffer_bind_group }
    }

    pub fn get_or_init_face_buffer_bind_group_layout(device: &wgpu::Device) -> &wgpu::BindGroupLayout {
        FACE_BUFFER_BIND_GROUP_LAYOUT.get_or_init(|| {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("DynamicChunkModelMesh_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        count: None,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        visibility: wgpu::ShaderStages::VERTEX,
                    }
                ]
            })
        })
    }

    fn create_face_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("DynamicRegionModelMesh_face_buffer"),
            mapped_at_creation: false,
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn create_indirect_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("DynamicRegionModelMesh_indirect_buffer"),
            mapped_at_creation: false,
            size: (std::mem::size_of::<wgpu::util::DrawIndexedIndirectArgs>() * PARTS_PER_CHUNK) as u64,
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn create_bind_group(device: &wgpu::Device, face_buffer_bind_group_layout: &wgpu::BindGroupLayout, face_buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("DynamicChunkModelMesh_bind_group_layout"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(face_buffer.as_entire_buffer_binding()),
                }
            ],
            layout: face_buffer_bind_group_layout,
        })
    }

    pub fn resize(&mut self, device: &wgpu::Device) {
        let new_face_buffer_size = (self.face_bucket_elements.iter().sum::<u32>() * std::mem::size_of::<FacePacked>() as u32) as u64;

        self.face_buffer.destroy();
        self.face_buffer = Self::create_face_buffer(device, new_face_buffer_size);
        self.face_buffer_bind_group = Self::create_bind_group(device, Self::get_or_init_face_buffer_bind_group_layout(device), &self.face_buffer);

        self.indirect_buffer.destroy();

        self.indirect_buffer = Self::create_indirect_buffer(device);
    }

    pub fn create_indirect_args(&self, faces_num: usize, chunk_part_index: usize) -> wgpu::util::DrawIndexedIndirectArgs {
        wgpu::util::DrawIndexedIndirectArgs {
            base_vertex: (self.face_bucket_elements[0..chunk_part_index].iter().sum::<u32>() * Face::VERTICES_PER_FACE as u32) as i32,
            first_index: 0,
            first_instance: (chunk_part_index * CHUNK_SIZE) as u32,
            index_count: (faces_num * Face::INDICES_PER_FACE) as u32,
            instance_count: 1
        }
    }

    pub fn insert_meshed_chunk(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, meshing_data: MeshingOutput, mesh_queue: &mut std::collections::VecDeque<(Vector2<i32>, usize)>) {
        let faces_size = meshing_data.faces.len() as u32;
        let chunk_part_index = meshing_data.chunk_part_index;
        let chunk_position = meshing_data.chunk_position;

        let mut needs_resizing = false;
        while self.face_bucket_elements[chunk_part_index] < faces_size {
            self.face_bucket_elements[chunk_part_index] *= 2;
            needs_resizing = true;
        }

        if needs_resizing {
            self.resize(device);
            for i in 0..PARTS_PER_CHUNK {
                if i == chunk_part_index { continue; }
                if !mesh_queue.contains(&(chunk_position, chunk_part_index)) {
                    mesh_queue.push_front((chunk_position, chunk_part_index));
                }
            }
        }

        let indirect_buffer_offset = (chunk_part_index * std::mem::size_of::<wgpu::util::DrawIndexedIndirectArgs>()) as u64;
        let face_buffer_offset = ((0..chunk_part_index).map(|i| self.face_bucket_elements[i]).sum::<u32>() * std::mem::size_of::<FacePacked>() as u32) as u64;
        if meshing_data.faces_num > 0 {
            queue.write_buffer(&self.face_buffer, face_buffer_offset, bytemuck::cast_slice(&meshing_data.faces));
        }
        queue.write_buffer(&self.indirect_buffer, indirect_buffer_offset, self.create_indirect_args(meshing_data.faces_num, chunk_part_index).as_bytes())
    }
}

impl Drop for DynamicChunkModelMesh {
    fn drop(&mut self) {
        self.face_buffer.destroy();
        self.indirect_buffer.destroy();
    }
}