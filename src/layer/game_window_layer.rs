use winit::{event::{DeviceEvent, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};

use crate::{event::{EventReader, Events}, game_window::{GameWindowEvent, KeyboardInputEvent, MouseInputEvent, MouseMoveEvent}};

use super::Layer;

pub struct GameWindowLayer {
    winit_event_reader: EventReader<winit::event::Event<()>>,
}

impl Layer for GameWindowLayer {
    fn on_update(&mut self, events: &mut crate::event::Events, application: &mut crate::application::Application) {
        if application.render_thread.is_work_done() {
            application.game_window.window().request_redraw();
        }

        for event in self.winit_event_reader.read(events).cloned().collect::<Vec<winit::event::Event<()>>>() {
            match event {
                winit::event::Event::WindowEvent { event, .. } => {
                    let _ = application.egui_winit_state.on_window_event(application.game_window.window(), &event);

                    match event {
                        WindowEvent::KeyboardInput { event, .. } => {
                            if let PhysicalKey::Code(key_code) = event.physical_key {
                                // TODO temp
                                if let KeyCode::Escape = key_code {
                                    application.quit = true;
                                }
                                events.send(KeyboardInputEvent { key_code, pressed: event.state.is_pressed(), repeat: event.repeat });
                            }
                        },
                        WindowEvent::MouseInput { button, state, .. } => {
                            events.send(MouseInputEvent { button, pressed: state.is_pressed() });
                        },
                        WindowEvent::RedrawRequested => {
                            events.send(GameWindowEvent::RedrawRequested);
                            application.is_render_frame = true;
                        },
                        WindowEvent::CloseRequested => {
                            application.quit = true;
                        },
                        WindowEvent::Resized(new_size) => {
                            application.resize_window(new_size);
                        },
                        _ => ()
                    }
                },
                winit::event::Event::DeviceEvent { event, .. } => {
                    match event {
                        DeviceEvent::MouseMotion { delta } => {
                            application.egui_winit_state.on_mouse_motion(delta);
                            events.send(MouseMoveEvent { delta: delta.into() });

                            // TODO temp
                            let x = application.surface_config.width / 2;
                            let y = application.surface_config.height / 2;
                            let _ = application.game_window.window().set_cursor_position(winit::dpi::PhysicalPosition::new(x, y));
                        },
                        _ => ()
                    }
                }
                _ => ()
            }
        }
    }

    fn on_render(&mut self, events: &mut Events, application: &mut crate::application::Application) {
    }
}

impl GameWindowLayer {
    pub fn new(events: &Events) -> Self {
        Self {
            winit_event_reader: EventReader::new(events),
        }
    }
}