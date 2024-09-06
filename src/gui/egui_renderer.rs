use egui::FullOutput;

pub struct EguiRenderer {
    renderer: egui_wgpu::Renderer
}

impl EguiRenderer {
    pub fn draw(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, window_surface_view: &wgpu::TextureView, screen_descriptor: egui_wgpu::ScreenDescriptor, context: &egui::Context, full_output: FullOutput) {
        let tris = context.tessellate(full_output.shapes.clone(), full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer.update_texture(&device, &queue, *id, &image_delta);
        }
        self.renderer.update_buffers(&device, &queue, encoder, &tris, &screen_descriptor);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &window_surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                label: Some("egui_render_pass"),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            self.renderer.render(&mut render_pass, &tris, &screen_descriptor);
        }

        for texture_id in &full_output.textures_delta.free {
            self.renderer.free_texture(texture_id)
        }
    }

    pub fn new(config: &wgpu::SurfaceConfiguration, device: &wgpu::Device) -> Self {
        let renderer = egui_wgpu::Renderer::new(&device, config.format, None, 1);
        Self { renderer }
    }
}