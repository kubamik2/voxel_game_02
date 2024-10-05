pub mod game_logic_layer;
pub mod chunk_rendering_layer;
pub mod game_window_layer;

use crate::{event::{Event, EventManager, EventManagerBuilder}, game::Game};

pub trait Layer {
    fn on_attach(&mut self, events: &EventManager) {}
    fn on_detach(&mut self, events: &EventManager) {}
    fn on_update(&mut self, events: &EventManager, game: &mut Game) {}
    fn on_render(&mut self, events: &EventManager, game: &mut Game) {}
}

pub struct LayerStack {
    layers: Vec<Box<dyn Layer>>,
    overlays: Vec<Box<dyn Layer>>,
    pub event_manager: EventManager,
}

impl LayerStack {
    pub fn new<F: Fn(EventManagerBuilder) -> EventManagerBuilder>(f: F) -> Self {
        let mut event_manager = f(EventManagerBuilder::default()).build();
        Self { layers: vec![], overlays: vec![], event_manager }
    }

    pub fn push_layer(&mut self, mut layer: Box<dyn Layer>) {
        layer.on_attach(&mut self.event_manager);
        self.layers.push(layer);
    }

    pub fn remove_layer(&mut self, index: usize) {
        let mut layer = self.layers.remove(index);
        layer.on_detach(&mut self.event_manager);
    }

    pub fn push_overlay(&mut self, mut layer: Box<dyn Layer>) {
        layer.on_attach(&mut self.event_manager);
        self.overlays.push(layer);
    }

    pub fn remove_overlay(&mut self, index: usize) {
        let mut layer = self.overlays.remove(index);
        layer.on_detach(&mut self.event_manager)
    }

    pub fn update(&mut self, game: &mut Game) {
        for layer in self.layers.iter_mut() {
            layer.on_update(&mut self.event_manager, game);
            if game.is_render_frame {
                layer.on_render(&mut self.event_manager, game);
            }
        }

        for overlay in self.overlays.iter_mut() {
            overlay.on_update(&mut self.event_manager, game);
            if game.is_render_frame {
                overlay.on_render(&mut self.event_manager, game);
            }
        }

        self.event_manager.update();
        game.is_render_frame = false;
    }
}

