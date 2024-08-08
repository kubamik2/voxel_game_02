use std::{collections::VecDeque, sync::{Arc, Condvar, Mutex}};

use cgmath::{Point3, Vector2, Vector3};
use wgpu::{util::DeviceExt, Device, Features, Queue};
use winit::{event::{DeviceEvent, Event, WindowEvent}, event_loop::EventLoop};

use crate::{block::quad_buffer::QuadBuffer, camera::Camera, game_window::GameWindow, global_vector::GlobalVecF, gui::{Gui}, interval::{Interval, IntervalThread}, render_thread::{RenderArgs, RenderEvent, RenderThread}, settings::Settings, texture::{Texture, TextureAtlas}, world::{chunk::{area::Area, chunk_manager::ChunkManager, chunk_mesh_map::ChunkMeshMap, chunk_part::{chunk_part_mesher::ChunkPartMesher, expanded_chunk_part::ExpandedChunkPart, CHUNK_SIZE, CHUNK_SIZE_I32}, dynamic_chunk_mesh::DynamicChunkMesh}, PARTS_PER_CHUNK}, BLOCK_MAP, BLOCK_MODEL_VARIANTS, QUADS};

pub struct Application {
    game_window: GameWindow,

    settings: Settings,
    settings_last_modified: std::time::SystemTime,

    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Arc<wgpu::Surface<'static>>,

    surface_config: wgpu::SurfaceConfiguration,
    aspect_ratio: f32,
    settings_path: std::path::PathBuf,

    camera: crate::camera::Player,

    chunk_manager: ChunkManager,

    alt_pressed: bool,
    egui_winit_state: egui_winit::State,
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
        
        let camera = crate::camera::Player::new();

        let device = Arc::new(device);
        let queue = Arc::new(queue);
        
        let chunk_manager = ChunkManager::new(10, 12, 12);
        let ctx = egui::Context::default();
        let viewport_id = ctx.viewport_id();
        let egui_winit_state = egui_winit::State::new(ctx, viewport_id, &game_window.window(), None, None);
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
                camera,
                chunk_manager,
                alt_pressed: false,
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
        
        let mut last_render_instant = std::time::Instant::now();

        let render_thread = RenderThread::new(app.device.clone(), app.queue.clone(), app.surface_config.clone(), app.game_window.window_arc().clone());
        let mut interval_300hz = Interval::new(std::time::Duration::from_secs_f32(1.0 / 300.0));
        let mut interval_60hz = Interval::new(std::time::Duration::from_secs_f32(1.0 / 60.0));
        let mut interval_2hz = Interval::new(std::time::Duration::from_secs_f32(1.0 / 2.0));

        event_loop.run(move |event, elwt| {
            elwt.set_control_flow(winit::event_loop::ControlFlow::Poll);

            if let Ok(settings_file_metadata) = std::fs::metadata(&settings_path) {
                if let Ok(settings_last_modified_new) = settings_file_metadata.modified() {
                    if app.settings_last_modified != settings_last_modified_new {
                        if let Ok(new_settings) = Settings::from_file(&settings_path) {
                            app.settings = new_settings;
                            app.settings_last_modified = settings_last_modified_new;
                        }
                    }
                }
            }

            if let Ok(full_output) = render_thread.work_done_receiver.try_recv() {
                app.egui_winit_state.handle_platform_output(&app.game_window.window(), full_output.platform_output);
                app.game_window.window().request_redraw();
            }

            interval_300hz.tick(|| {
                app.chunk_manager.update(&app.device);
            });

            interval_60hz.tick(|| {
                while let Some(changed_block_position) = app.chunk_manager.changed_blocks.pop() {
                    let mut afflicted_chunk_parts = hashbrown::HashSet::new();
                    let Some(chunk_part) = app.chunk_manager.chunk_map.get_mut_chunk_part(changed_block_position.chunk) else { continue; };
                    
                    let mut removed_light_emitters = std::mem::take(&mut chunk_part.removed_light_emitters);
                    let mut added_light_emitters = std::mem::take(&mut chunk_part.added_light_emitters);

                    let Some(mut area) = Area::new(&mut app.chunk_manager.chunk_map, changed_block_position.chunk.xz()) else { continue; };
                    area.update_sky_light_at(changed_block_position.local().map(|f| f as i32) + Vector3::new(0, changed_block_position.chunk.y * CHUNK_SIZE_I32, 0), &mut afflicted_chunk_parts);
                    area.update_block_light_at(changed_block_position.local().map(|f| f as i32) + Vector3::new(0, changed_block_position.chunk.y * CHUNK_SIZE_I32, 0), &mut afflicted_chunk_parts);
                    
                    while let Some(light_local_position) = removed_light_emitters.pop() {
                        let area_position = light_local_position.map(|f| f as i32) + Vector3::new(0, changed_block_position.chunk.y * CHUNK_SIZE_I32, 0);
                        area.remove_block_light_at(area_position, &mut afflicted_chunk_parts);
                    }

                    while let Some(light_local_position) = added_light_emitters.pop() {
                        let area_position = light_local_position.map(|f| f as i32) + Vector3::new(0, changed_block_position.chunk.y * CHUNK_SIZE_I32, 0);
                        area.propagate_block_light_at(area_position, &mut afflicted_chunk_parts);
                    }

                    for chunk in area.chunks {
                        app.chunk_manager.chunk_map.insert_arc(chunk.position, chunk);
                    }

                    for (chunk_position, chunk_part_index) in afflicted_chunk_parts {
                        let Some(mesh) = app.chunk_manager.chunk_mesh_map.get_mut(chunk_position) else { continue; };
                        mesh.parts_need_meshing[chunk_part_index] = true;
                    }
                }
                app.chunk_manager.insert_chunks_around_player(Vector2::new(0, 0));
                app.camera.modify_block(&mut app.chunk_manager, BLOCK_MAP.get("torch").unwrap().clone().into());
            });

            match event {
                Event::WindowEvent { event, .. } => {
                    if app.alt_pressed {
                        app.egui_winit_state.on_window_event(&app.game_window.window(), &event);
                    }
                    match event {
                        WindowEvent::CloseRequested | WindowEvent::KeyboardInput { event: winit::event::KeyEvent { physical_key: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape), .. }, .. } => {
                            elwt.exit();
                        },
                        WindowEvent::RedrawRequested => {
                            app.camera.update();
                            render_thread.update_view_projection(&app.camera, app.aspect_ratio);
                            app.chunk_manager.collect_meshing_outputs(&app.device, &app.queue);
                            let meshes = app.chunk_manager.get_ready_meshes();
                            render_thread.render(RenderArgs {
                                surface: app.surface.clone(),
                                game_window: app.game_window.clone(),
                                meshes,
                                surface_config: app.surface_config.clone(),
                                gui: Gui::new(&app.camera, &app.chunk_manager.chunk_map, last_render_instant.elapsed()),
                                raw_input: app.egui_winit_state.take_egui_input(&app.game_window.window()),
                            }).unwrap();
                            last_render_instant = std::time::Instant::now();
                        }
                        WindowEvent::Resized(new_size) => {
                            app.resize_window(new_size);
                        },
                        WindowEvent::KeyboardInput { event, .. } => {
                            if let winit::keyboard::PhysicalKey::Code(key_code) = event.physical_key {
                                app.camera.handle_keyboard_input(key_code, event.state);
                                if key_code == winit::keyboard::KeyCode::AltLeft {
                                    app.alt_pressed = event.state.is_pressed();
                                    app.game_window.window().set_cursor_visible(event.state.is_pressed());
                                }
                            }
                        },
                        WindowEvent::MouseInput { button, state: elem_state, .. } => {
                            if app.alt_pressed {
                                return;
                            }
                            app.camera.handle_mouse_input(button, elem_state);
                            app.camera.modify_block(&mut app.chunk_manager, BLOCK_MAP.get("torch").unwrap().clone().into());
                        },
                        _ => ()
                    }
                },
                Event::DeviceEvent { event, .. } => {
                    if let DeviceEvent::MouseMotion { delta: (delta_x, delta_y) } = event {
                        if !app.alt_pressed {
                            app.camera.handle_mouse_movement(delta_x as f32, delta_y as f32);
                            if app.game_window.window().has_focus() {
                                app.game_window.window().set_cursor_position(winit::dpi::PhysicalPosition::new(app.width() / 2, app.height() / 3)).unwrap();
                            }
                        }
                    }
                },
                _ => ()
            }
        })?;
        Ok(())
    }
}
