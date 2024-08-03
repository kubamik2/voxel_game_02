use cgmath::Vector3;

use crate::{block::light::LightLevel, camera::Player, global_vector::GlobalVecF, world::chunk::chunk_map::ChunkMap};

pub mod egui_renderer;
pub struct Gui {
    pub position: GlobalVecF,
    pub direction: Vector3<f32>,
    pub light_level: LightLevel,
}

impl Gui {
    pub fn new(player: &Player, chunk_map: &ChunkMap) -> Self {
        Self {
            position: player.position,
            direction: player.direction,
            light_level: chunk_map.get_light_level_global(player.position.into()).map(|f| *f).unwrap_or(LightLevel::new(0, 0).unwrap())
        }
    }
    pub fn debug(&self, ctx: &egui::Context) {
        egui::Window::new("debug")
        .title_bar(false)
        .show(ctx, |ui| {
            ui.label(format!("local:  x: {: <4.1} y: {: <4.1} z: {: <4.1}", self.position.local().x, self.position.local().y, self.position.local().z));
            ui.label(format!("chunk:  x: {: <4} y: {: <4} z: {: <4}", self.position.chunk.x, self.position.chunk.y, self.position.chunk.z));
            ui.label(format!("block: {: <2}   sky: {: <2}", self.light_level.get_block(), self.light_level.get_sky()));
        });
    }
}