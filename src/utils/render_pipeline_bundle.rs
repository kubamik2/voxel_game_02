use std::sync::Arc;
 
#[derive(Clone)]
pub struct RenderPipelineBundle {
    render_pipeline: Arc<wgpu::RenderPipeline>,
    layout: Arc<wgpu::PipelineLayout>,
}

impl RenderPipelineBundle {
    pub fn new(render_pipeline: wgpu::RenderPipeline, layout: wgpu::PipelineLayout) -> Self {
        Self { render_pipeline: Arc::new(render_pipeline), layout: Arc::new(layout) }
    }

    pub fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.render_pipeline
    }

    pub fn layout(&self) -> &wgpu::PipelineLayout {
        &self.layout
    }
}