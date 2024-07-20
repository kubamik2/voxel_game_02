use winit::{
    event_loop::EventLoop, window::{Window, WindowBuilder}
};
use std::sync::Arc;
use crate::settings::Settings;

#[derive(Clone)]
pub struct GameWindow {
    window: Arc<Window>
}

impl GameWindow {
    pub fn new(settings: &Settings) -> anyhow::Result<(Self, EventLoop<()>)> {
        let event_loop = EventLoop::new()?;
        let monitor_handle = event_loop.primary_monitor().unwrap(); // TODO handle multiple monitors
        let fullscreen = if settings.fullscreen {
            Some(
                if settings.borderless {
                    winit::window::Fullscreen::Borderless(None)
                } else {
                    winit::window::Fullscreen::Exclusive(monitor_handle.video_modes().next().unwrap()) // TODO handle no video modes
                }
            )
        } else {
            None
        };
        
        let window = WindowBuilder::new()
            .with_fullscreen(fullscreen)
            .with_inner_size(winit::dpi::PhysicalSize::new(settings.resolution.0, settings.resolution.1))
            .with_resizable(false)
            .build(&event_loop)?;

        window.set_cursor_visible(false);

        Ok((Self { window: Arc::new(window) }, event_loop))
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn window_arc(&self) -> Arc<Window> {
        self.window.clone()
    }

    pub fn resize<T: Into<winit::dpi::Size>>(&self, new_size: T) {
        let new_size: winit::dpi::Size = new_size.into();
        self.window.set_min_inner_size(Some(new_size));
        self.window.set_max_inner_size(Some(new_size));
    }
}
