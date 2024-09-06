pub mod bind_group_bundle;
pub mod render_pipeline_bundle;
pub mod index_buffer;

pub const fn none<T>() -> Option<T> { None }
pub const fn bool_true() -> bool { true }
pub const fn bool_false() -> bool { false }