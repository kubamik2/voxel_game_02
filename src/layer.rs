use crate::event::{Event, EventQueue};

pub trait Layer {
    fn id(&self) -> u32;
    fn on_attach(&mut self);
    fn on_detach(&mut self);
    fn on_update(&mut self, event_queue: &mut EventQueue);
}

pub struct LayerStack {
    layers: Vec<Box<dyn Layer>>,
    overlays: Vec<Box<dyn Layer>>,
}

impl LayerStack {
    pub fn push_layer(&mut self, layer: Box<dyn Layer>) {
        self.layers.push(layer);
    }

    pub fn remove_layer(&mut self, layer_id: u32) {
        let Some(index) = self.layers.iter().position(|p| p.id() == layer_id) else { return; };
        self.layers.remove(index);
    }

    pub fn push_overlay(&mut self, layer: Box<dyn Layer>) {
        self.overlays.push(layer);
    }

    pub fn remove_overlay(&mut self, layer_id: u32) {
        let Some(index) = self.overlays.iter().position(|p| p.id() == layer_id) else { return; };
        self.overlays.remove(index);
    }
}

