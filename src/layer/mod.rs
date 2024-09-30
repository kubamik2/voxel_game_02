pub mod game_logic_layer;
pub mod chunk_rendering_layer;
pub mod game_window_layer;

use crate::{game::Game, event::{Events, Event}};

pub trait Layer {
    fn on_attach(&mut self, events: &mut Events) {}
    fn on_detach(&mut self, events: &mut Events) {}
    fn on_update(&mut self, events: &mut Events, game: &mut Game) {}
    fn on_render(&mut self, events: &mut Events, game: &mut Game) {}
}

pub struct LayerStack {
    layers: Vec<Box<dyn Layer>>,
    overlays: Vec<Box<dyn Layer>>,
    pub events: Events,
}

impl LayerStack {
    pub fn new() -> Self {
        Self { layers: vec![], overlays: vec![], events: Events::new() }
    }

    pub fn push_layer(&mut self, mut layer: Box<dyn Layer>) {
        layer.on_attach(&mut self.events);
        self.layers.push(layer);
    }

    pub fn remove_layer(&mut self, index: usize) {
        let mut layer = self.layers.remove(index);
        layer.on_detach(&mut self.events);
    }

    pub fn push_overlay(&mut self, mut layer: Box<dyn Layer>) {
        layer.on_attach(&mut self.events);
        self.overlays.push(layer);
    }

    pub fn remove_overlay(&mut self, index: usize) {
        let mut layer = self.overlays.remove(index);
        layer.on_detach(&mut self.events)
    }

    pub fn update(&mut self, game: &mut Game) {
        for layer in self.layers.iter_mut() {
            layer.on_update(&mut self.events, game);
            if game.is_render_frame {
                layer.on_render(&mut self.events, game);
            }
        }

        for overlay in self.overlays.iter_mut() {
            overlay.on_update(&mut self.events, game);
            if game.is_render_frame {
                overlay.on_render(&mut self.events, game);
            }
        }

        self.events.update();
        game.is_render_frame = false;
    }

    pub fn register_event_type<E: Event>(&mut self) {
        self.events.register_event_type::<E>();
    }
}

