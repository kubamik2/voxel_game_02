use std::sync::Arc;

use wgpu::{Device, Features, Queue};
use winit::{event::Event, event_loop::EventLoop};

use crate::{game_window::{GameWindow, GameWindowEvent, KeyboardInputEvent, MouseInputEvent, MouseMoveEvent}, layer::{game_logic_layer::{GameLogicLayer, ChunkUpdateRenderMesh}, chunk_rendering_layer::ChunkRenderingLayer, game_window_layer::GameWindowLayer, LayerStack}, render_thread::RenderThread, settings::Settings};

pub struct Application {
    pub game_window: GameWindow,

    pub settings: Settings,
    pub settings_last_modified: std::time::SystemTime,
    pub settings_path: std::path::PathBuf,

    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface: Arc<wgpu::Surface<'static>>,

    pub surface_config: wgpu::SurfaceConfiguration,
    pub aspect_ratio: f32,

    pub render_thread: RenderThread,
    pub quit: bool,
    pub is_render_frame: bool,

    pub egui_winit_state: egui_winit::State,
}

impl Application {
    pub async fn new(settings_path: &std::path::Path) -> anyhow::Result<(Self, EventLoop<()>)> {
        let settings = Settings::from_file(settings_path)?;
        let settings_last_modified = std::fs::metadata(settings_path)?.modified()?;

        let (game_window, event_loop) = GameWindow::new(&settings)?;
        let size = settings.resolution;

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        let surface = instance.create_surface(game_window.window_arc())?;

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
            compatible_surface: Some(&surface),
            ..Default::default()
        }).await.expect("Couldn't create adapter");

        let required_limits = wgpu::Limits {
            max_bind_groups: 6,
            max_buffer_size: i32::MAX as u64,
            ..Default::default()
        };
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("device"),
            required_features: Features::BUFFER_BINDING_ARRAY | Features::STORAGE_RESOURCE_BINDING_ARRAY | Features::MULTI_DRAW_INDIRECT,
            required_limits,
            ..Default::default()
        }, None).await?;

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|p| p.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]); // TODO change

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.0,
            height: size.1,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_capabilities.alpha_modes[0], // TODO change
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };

        surface.configure(&device, &surface_config);
        let aspect_ratio = surface_config.width as f32 / surface_config.height as f32;
        
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let render_thread = RenderThread::new(device.clone(), queue.clone(), surface_config.clone());

        let egui_context = egui::Context::default();
        let viewport_id = egui_context.viewport_id();
        let egui_winit_state = egui_winit::State::new(egui_context, viewport_id, game_window.window(), None, None);
        
        Ok((
            Self {
                game_window,
                settings,
                settings_last_modified,
                device,
                queue,
                surface: Arc::new(surface),
                surface_config,
                aspect_ratio,
                settings_path: settings_path.to_owned(),
                render_thread,
                quit: false,
                is_render_frame: false,
                egui_winit_state,
            }, 
            event_loop,
        ))
    }

    pub fn resize_window(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.height > 0 && new_size.width > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
            self.aspect_ratio = self.width() as f32 / self.height() as f32;
        }
    }

    pub fn height(&self) -> u32 {
        self.surface_config.height
    }

    pub fn width(&self) -> u32 {
        self.surface_config.width
    }

    pub fn run<T: Into<std::path::PathBuf>>(settings_path: T) -> anyhow::Result<()> {
        let settings_path = settings_path.into();
        let (mut app, event_loop) = pollster::block_on(Self::new(&settings_path))?;
        
        let mut layers = LayerStack::new();

        layers.register_event_type::<GameWindowEvent>();
        layers.register_event_type::<ChunkUpdateRenderMesh>();
        layers.register_event_type::<KeyboardInputEvent>();
        layers.register_event_type::<MouseInputEvent>();
        layers.register_event_type::<MouseMoveEvent>();
        layers.register_event_type::<Event<()>>();

        layers.push_layer(Box::new(GameWindowLayer::new(&layers.events)));
        layers.push_layer(Box::new(GameLogicLayer::new(&layers.events, &app.settings)));
        layers.push_layer(Box::new(ChunkRenderingLayer::new(&layers.events)));

        event_loop.run(move |event, elwt| {
            elwt.set_control_flow(winit::event_loop::ControlFlow::Poll);
            layers.events.send(event);
            layers.update(&mut app);
            if app.quit {
                elwt.exit();
            }
        })?;
        Ok(())
    }
}