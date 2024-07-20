pub trait Renderable {
    fn render_pipeline(device: &wgpu::Device);
}