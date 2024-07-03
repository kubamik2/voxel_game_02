use crate::{block::model::{Face, IndexFormat, INDICES_PER_FACE}, world::CHUNK_PARTS_PER_REGION};

pub type Item = Face;

// Model mesh that is divided into buckets containing each chunk's faces
pub struct DynamicRegionModelMesh {
    face_buffer: wgpu::Buffer,
    indirect_buffer: wgpu::Buffer,
    face_bucket_sizes: [u32; CHUNK_PARTS_PER_REGION as usize],
}

impl DynamicRegionModelMesh {
    pub const MIN_BUCKET_SIZE: u32 = Self::MIN_BUCKET_ELEMENTS * std::mem::size_of::<Face>() as u32;
    pub const MIN_BUCKET_ELEMENTS: u32 = 64;

    pub fn new(device: &wgpu::Device) -> Self {
        let face_buffer = Self::create_face_buffer(device, (Self::MIN_BUCKET_SIZE as usize * CHUNK_PARTS_PER_REGION) as u64);
        let indirect_buffer = Self::create_indirect_buffer(device);
        
        let face_bucket_sizes = std::array::from_fn(|_| Self::MIN_BUCKET_SIZE);

        Self { face_buffer, indirect_buffer, face_bucket_sizes }
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
            size: (std::mem::size_of::<wgpu::util::DrawIndexedIndirectArgs>() * CHUNK_PARTS_PER_REGION) as u64,
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub fn resize(&mut self, device: &wgpu::Device) {
        let new_face_buffer_size = self.face_bucket_sizes.iter().sum::<u32>() as u64;

        if new_face_buffer_size != self.face_buffer.size() {
            self.face_buffer.destroy();
            self.face_buffer = Self::create_face_buffer(device, new_face_buffer_size);
        }
    }
}

impl Drop for DynamicRegionModelMesh {
    fn drop(&mut self) {
        self.face_buffer.destroy();
        self.indirect_buffer.destroy();
    }
}