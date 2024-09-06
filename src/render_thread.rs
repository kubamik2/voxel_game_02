use std::sync::{mpsc::{channel, Receiver, Sender}, Arc};

use egui_wgpu::ScreenDescriptor;

use crate::gui::egui_renderer::EguiRenderer;

pub type RenderCommand = Box<dyn FnOnce(&wgpu::Queue, &mut wgpu::CommandEncoder, &wgpu::TextureView) + Send>; 

pub struct RenderThread {
    event_sender: Arc<Sender<RenderEvent>>,
    work_done_receiver: Receiver<()>,
    commands: Vec<RenderCommand>,
}

pub enum RenderEvent {
    Render((RenderArgs, Box<[RenderCommand]>)),
}

pub struct RenderArgs {
    pub surface: Arc<wgpu::Surface<'static>>,
    pub window: Arc<winit::window::Window>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub egui_context: egui::Context,
    pub egui_full_output: egui::FullOutput,
}

impl RenderThread {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, surface_config: wgpu::SurfaceConfiguration) -> Self {
        let (event_sender, event_receiver) = channel();
        let (work_done_sender, work_done_receiver) = channel();
        std::thread::spawn(move || Self::run(event_receiver,  device, queue, surface_config, work_done_sender));
        
        Self { event_sender: Arc::new(event_sender), work_done_receiver, commands: vec![] }
    }

    pub fn push_render<F: FnOnce(&wgpu::Queue, &mut wgpu::CommandEncoder, &wgpu::TextureView) + Send + 'static>(&mut self, f: F) {
        self.commands.push(Box::new(f));
    }

    pub fn execute_queued_renders(&mut self, render_args: RenderArgs) {
        let mut commands = vec![];
        std::mem::swap(&mut self.commands, &mut commands);
        self.event_sender.send(RenderEvent::Render((render_args, commands.into_boxed_slice()))).expect("render_thread.render failed");
    }

    fn run(event_receiver: Receiver<RenderEvent>, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, surface_config: wgpu::SurfaceConfiguration, work_done_sender: Sender<()>) {
        let mut render_thread = RenderThreadInner::new(device, queue, surface_config, work_done_sender);
        for event in event_receiver.iter() {
            match event {
                RenderEvent::Render((args, commands)) => render_thread.render(args, commands),
            }
        }
    }

    pub fn is_work_done(&self) -> bool {
        self.work_done_receiver.try_recv().is_ok()
    }
}

struct RenderThreadInner {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    egui_renderer: EguiRenderer,
    work_done_sender: Sender<()>,
}

impl RenderThreadInner {
    fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, surface_config: wgpu::SurfaceConfiguration, work_done_sender: Sender<()>) -> Self {
        let egui_renderer = EguiRenderer::new(&surface_config, &device);
        Self { device, queue, work_done_sender, egui_renderer }
    }

    fn render(&mut self, args: RenderArgs, commands: Box<[RenderCommand]>) {
        let Ok(output) = args.surface.get_current_texture() else {
            self.work_done_sender.send(()).expect("render_thread work_done_sender.send() failed");
            println!("fail");
            return;
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render_thread command encoder") });

        for command in commands {
            command(&self.queue, &mut encoder, &view);
        }

        let screen_descriptor = ScreenDescriptor {
            pixels_per_point: args.window.scale_factor() as f32,
            size_in_pixels: [args.surface_config.width, args.surface_config.height],
        };

        self.egui_renderer.draw(&self.device, &self.queue, &mut encoder, &view, screen_descriptor, &args.egui_context, args.egui_full_output);
        
        self.queue.submit(std::iter::once(encoder.finish()));
        args.window.pre_present_notify();
        output.present();
        self.work_done_sender.send(()).expect("render_thread work_done_sender.send() failed");
    }
}
