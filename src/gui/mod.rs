use cgmath::Vector3;
use egui::{Color32, RichText, Ui};

use crate::{block::light::LightLevel, camera::Player, global_vector::GlobalVecF, world::chunk::chunk_map::ChunkMap};

pub mod egui_renderer;
pub struct Gui {
    pub position: GlobalVecF,
    pub direction: Vector3<f32>,
    pub light_level: LightLevel,
    pub last_frame_time: std::time::Duration,
}

impl Gui {
    pub fn new(player: &Player, chunk_map: &ChunkMap, last_frame_time: std::time::Duration) -> Self {
        Self {
            position: player.position,
            direction: player.direction,
            light_level: chunk_map.get_light_level_global(player.position.into()).map(|f| *f).unwrap_or(LightLevel::new(0, 0).unwrap()),
            last_frame_time,
        }
    }
    pub fn debug(&self, ctx: &egui::Context) {
        #[inline]
        fn add_label(ui: &mut Ui, text: String) {
            ui.label(RichText::new(text).size(14.0).color(Color32::WHITE));
        }
        egui::Window::new("debug")
        .title_bar(false)
        .resizable(false)
        .show(ctx, |ui| {
            add_label(ui, format!("local:  x: {: <4.1} y: {: <4.1} z: {: <4.1}", self.position.local().x, self.position.local().y, self.position.local().z));
            add_label(ui, format!("chunk:  x: {: <4} y: {: <4} z: {: <4}", self.position.chunk.x, self.position.chunk.y, self.position.chunk.z));
            add_label(ui, format!("block: {: <2}   sky: {: <2}", self.light_level.get_block(), self.light_level.get_sky()));
            add_label(ui, format!("fps: {: <3}   mpf: {: <4.1}", (1.0 / self.last_frame_time.as_secs_f32()).floor() as u32, self.last_frame_time.as_secs_f32() * 1000.0));
        });
    }
}