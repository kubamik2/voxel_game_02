use std::io::Read;

use egui::Color32;

pub struct EguiRenderer {
    pub context: egui::Context,
    pub state: egui_winit::State,
    pub renderer: egui_wgpu::Renderer
}

impl EguiRenderer {
    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        window: &winit::window::Window,
        window_surface_view: &wgpu::TextureView,
        screen_descriptor: egui_wgpu::ScreenDescriptor,
        run_ui: impl FnOnce(&egui::Context),
    ) {
        let raw_input = self.state.take_egui_input(&window);
        let full_output = self.context.run(raw_input, |_| {
            run_ui(&self.context);
        });

        self.state
            .handle_platform_output(&window, full_output.platform_output);

        let tris = self
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(&device, &queue, *id, &image_delta);
        }
        self.renderer
            .update_buffers(&device, &queue, encoder, &tris, &screen_descriptor);
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        self.renderer.render(&mut rpass, &tris, &screen_descriptor);
        drop(rpass);
        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }
    }

    pub fn new(window: &winit::window::Window, config: &wgpu::SurfaceConfiguration, device: &wgpu::Device) -> Self {
        let visuals = egui::Visuals {
            window_fill: Color32::from_rgb(0, 0, 0),
            faint_bg_color: Color32::TRANSPARENT,
            extreme_bg_color: Color32::TRANSPARENT,
            panel_fill: Color32::TRANSPARENT,
            window_shadow: egui::epaint::Shadow::NONE,
            window_rounding: egui::Rounding::same(2.0),
            window_stroke: egui::Stroke::NONE,
            ..Default::default()
        };

        let ctx = egui::Context::default();
        ctx.set_visuals(visuals);

        // if let Ok(mut file) = std::fs::File::open("custom_font.ttf") {
        //     let mut bytes = vec![];
        //     if let Ok(_) = file.read_to_end(&mut bytes) {
        //         let mut fonts = egui::FontDefinitions::default();
        //         fonts.font_data.insert("custom_font".to_string(), egui::FontData::from_owned(bytes));
        //         fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "custom_font".to_string());
        //         ctx.set_fonts(fonts);
        //     }
        // }
        

        let viewport_id = ctx.viewport_id();
        let egui_state = egui_winit::State::new(ctx.clone(), viewport_id, &window, None, None);
        let renderer = egui_wgpu::Renderer::new(&device, config.format, None, 1);
        
        Self { context: ctx, state: egui_state, renderer }
    }
}