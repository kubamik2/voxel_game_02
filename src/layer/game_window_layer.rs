use winit::{event::{DeviceEvent, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};

use crate::{event::{EventReader, EventManager}, game_window::{GameWindowEvent, KeyboardInputEvent, MouseInputEvent, MouseMoveEvent}};

use super::Layer;

pub struct GameWindowLayer {
    winit_event_reader: EventReader<winit::event::Event<()>>,
}

impl Layer for GameWindowLayer {
    fn on_update(&mut self, events: &mut crate::event::EventManager, game: &mut crate::game::Game) {
        if game.render_thread.is_work_done() {
            game.game_window.window().request_redraw();
        }

        for event in self.winit_event_reader.read().cloned().collect::<Vec<winit::event::Event<()>>>() {
            match event {
                winit::event::Event::WindowEvent { event, .. } => {
                    let _ = game.egui_winit_state.on_window_event(game.game_window.window(), &event);

                    match event {
                        WindowEvent::KeyboardInput { event, .. } => {
                            if let PhysicalKey::Code(key_code) = event.physical_key {
                                // TODO temp
                                if let KeyCode::Escape = key_code {
                                    game.quit = true;
                                }
                                events.send(KeyboardInputEvent { key_code, pressed: event.state.is_pressed(), repeat: event.repeat });
                            }
                        },
                        WindowEvent::MouseInput { button, state, .. } => {
                            events.send(MouseInputEvent { button, pressed: state.is_pressed() });
                        },
                        WindowEvent::RedrawRequested => {
                            events.send(GameWindowEvent::RedrawRequested);
                            game.is_render_frame = true;
                        },
                        WindowEvent::CloseRequested => {
                            game.quit = true;
                        },
                        WindowEvent::Resized(new_size) => {
                            game.resize_window(new_size);
                        },
                        _ => ()
                    }
                },
                winit::event::Event::DeviceEvent { event, .. } => {
                    match event {
                        DeviceEvent::MouseMotion { delta } => {
                            game.egui_winit_state.on_mouse_motion(delta);
                            events.send(MouseMoveEvent { delta: delta.into() });

                            // TODO temp
                            let x = game.surface_config.width / 2;
                            let y = game.surface_config.height / 2;
                            let _ = game.game_window.window().set_cursor_position(winit::dpi::PhysicalPosition::new(x, y));
                        },
                        _ => ()
                    }
                }
                _ => ()
            }
        }

    }

    fn on_render(&mut self, events: &mut EventManager, game: &mut crate::game::Game) {
        game.game_window.window().set_cursor_visible(false);
        game.egui_winit_state.handle_platform_output(game.game_window.window(), game.egui_full_output.platform_output.clone());
        let raw_input = game.egui_winit_state.take_egui_input(game.game_window.window());
        game.egui_winit_state.egui_ctx().begin_frame(raw_input);
    }
}

impl GameWindowLayer {
    pub fn new(events: &EventManager) -> Self {
        Self {
            winit_event_reader: EventReader::new(events),
        }
    }
}
