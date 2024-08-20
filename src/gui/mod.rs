use cgmath::Vector3;
use egui::{Color32, RichText, Ui};

use crate::{block::light::LightLevel, global_vector::GlobalVecF, world::World};

pub mod egui_renderer;
pub struct DebugGui {
    pub position: GlobalVecF,
    pub direction: Vector3<f32>,
    pub light_level: LightLevel,
    pub last_frame_time: std::time::Duration,
}

impl DebugGui {
    pub fn new(world: &World, last_frame_time: std::time::Duration) -> Self {
        Self {
            position: world.player.position,
            direction: world.player.direction,
            light_level: world.chunk_manager.chunk_map.get_light_level_global(world.player.position.into()).map(|f| *f).unwrap_or(LightLevel::new(0, 0).unwrap()),
            last_frame_time,
        }
    }

    pub fn show(&self, ctx: &egui::Context) {
        #[inline]
        fn add_label(ui: &mut Ui, text: String) {
            ui.label(RichText::new(text).size(16.0).color(Color32::WHITE));
        }
        egui::Window::new("debug")
        .title_bar(false)
        .resizable(false)
        .frame(egui::Frame {
            fill: Color32::TRANSPARENT,
            ..Default::default()
        })
        .show(ctx, |ui| {
            add_label(ui, format!("local:  x: {: <4.1} y: {: <4.1} z: {: <4.1}", self.position.local().x, self.position.local().y, self.position.local().z));
            add_label(ui, format!("chunk:  x: {: <4} y: {: <4} z: {: <4}", self.position.chunk.x, self.position.chunk.y, self.position.chunk.z));
            let global_position: Vector3<f64> = self.position.into();
            add_label(ui, format!("global:  x: {: <4.1} y: {: <4.1} z: {: <4.1}", global_position.x, global_position.y, global_position.z));
            add_label(ui, format!("light_level:  block: {: <2}   sky: {: <2}", self.light_level.get_block(), self.light_level.get_sky()));
            add_label(ui, format!("fps: {: <3}   mpf: {: <4.1}", (1.0 / self.last_frame_time.as_secs_f32()).floor() as u32, self.last_frame_time.as_secs_f32() * 1000.0));
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|center| {
                center.add(egui::Label::new(egui::RichText::new("+").color(Color32::DARK_GRAY).size(32.0).monospace()).selectable(false));
            });
        });
    }
}