use std::sync::Arc;

#[derive(Clone)]
pub struct BindGroupBundle {
    bind_group: Arc<wgpu::BindGroup>,
    layout: Arc<wgpu::BindGroupLayout>,
}

impl BindGroupBundle {
    pub fn new(bind_group: wgpu::BindGroup, layout: wgpu::BindGroupLayout) -> Self {
        Self { bind_group: Arc::new(bind_group), layout: Arc::new(layout) }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }
}