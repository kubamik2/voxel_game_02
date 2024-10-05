pub mod game_logic_layer;
pub mod chunk_rendering_layer;
pub mod game_window_layer;

use crate::{event::{Event, EventManager, EventManagerBuilder}, game::Game, GLOBAL_RESOURCES};

pub trait Layer {
    fn on_attach(&mut self, events: &EventManager) {}
    fn on_detach(&mut self, events: &EventManager) {}
    fn on_update(&mut self, events: &EventManager, game: &mut Game) {}
    fn on_render(&mut self, events: &EventManager, game: &mut Game) {}
}

pub struct LayerStack {
    layers: Vec<Box<dyn Layer>>,
    overlays: Vec<Box<dyn Layer>>,
}

impl LayerStack {
    pub fn new() -> Self {
        Self { layers: vec![], overlays: vec![] }
    }

    pub fn push_layer(&mut self, mut layer: Box<dyn Layer>) {
        layer.on_attach((*GLOBAL_RESOURCES).get::<EventManager>().unwrap());
        self.layers.push(layer);
    }

    pub fn remove_layer(&mut self, index: usize) {
        let mut layer = self.layers.remove(index);
        layer.on_detach((*GLOBAL_RESOURCES).get::<EventManager>().unwrap());
    }

    pub fn push_overlay(&mut self, mut layer: Box<dyn Layer>) {
        layer.on_attach((*GLOBAL_RESOURCES).get::<EventManager>().unwrap());
        self.overlays.push(layer);
    }

    pub fn remove_overlay(&mut self, index: usize) {
        let mut layer = self.overlays.remove(index);
        layer.on_detach((*GLOBAL_RESOURCES).get::<EventManager>().unwrap())
    }

    pub fn update(&mut self, game: &mut Game) {
        let event_manager = (*GLOBAL_RESOURCES).get::<EventManager>().unwrap();

        for layer in self.layers.iter_mut() {
            layer.on_update(event_manager, game);
            if game.is_render_frame {
                layer.on_render(event_manager, game);
            }
        }

        for overlay in self.overlays.iter_mut() {
            overlay.on_update(event_manager, game);
            if game.is_render_frame {
                overlay.on_render(event_manager, game);
            }
        }

        event_manager.update();
        game.is_render_frame = false;
    }
}

