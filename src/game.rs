use std::{io::Read, sync::Arc};

use egui::Color32;
use wgpu::{Device, Features, Queue};
use winit::{event::Event, event_loop::EventLoop};

use crate::{event::EventManager, game_window::{GameWindow, GameWindowEvent, KeyboardInputEvent, MouseInputEvent, MouseMoveEvent}, layer::{chunk_rendering_layer::ChunkRenderingLayer, game_logic_layer::{ChunkUpdateRenderMesh, GameLogicLayer}, game_window_layer::GameWindowLayer, LayerStack}, render_thread::RenderThread, settings::Settings, GLOBAL_RESOURCES};

pub struct Game {
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
    pub egui_full_output: egui::FullOutput,
    pub last_render_instant: std::time::Instant,
    pub last_update_time: std::time::Duration,
}

impl Game {
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

        if let Ok(mut file) = std::fs::File::open("./assets/fonts/minecraft.ttf") {
            let mut bytes = vec![];
            if let Ok(_) = file.read_to_end(&mut bytes) {
                let mut fonts = egui::FontDefinitions::default();
                fonts.font_data.insert("custom_font".to_string(), egui::FontData::from_owned(bytes));
                fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "custom_font".to_string());
                egui_winit_state.egui_ctx().set_fonts(fonts);
            }
        }

        egui_winit_state.egui_ctx().set_visuals(visuals);
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
                egui_full_output: egui::FullOutput::default(),
                last_render_instant: std::time::Instant::now(),
                last_update_time: std::time::Duration::ZERO,
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
        let (mut game, event_loop) = pollster::block_on(Self::new(&settings_path))?;
        let event_manager = (*GLOBAL_RESOURCES).get::<EventManager>().unwrap();
        
        let mut layers = LayerStack::new();


        layers.push_layer(Box::new(GameWindowLayer::new()));
        layers.push_layer(Box::new(GameLogicLayer::new(&game.device, &game.queue, &game.surface_config, &game.settings).unwrap()));
        layers.push_layer(Box::new(ChunkRenderingLayer::new()));

        event_loop.run(move |event, elwt| {
            elwt.set_control_flow(winit::event_loop::ControlFlow::Poll);
            event_manager.send(event);
            layers.update(&mut game);
            if game.quit {
                elwt.exit();
            }
        })?;
        Ok(())
    }
}
