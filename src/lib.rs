pub mod creature;
pub mod mind;
pub mod sim;
pub mod theme;
pub mod world;

#[cfg(not(target_arch = "wasm32"))]
pub mod config;
#[cfg(not(target_arch = "wasm32"))]
pub mod render;
#[cfg(not(target_arch = "wasm32"))]
pub mod sound;
#[cfg(target_arch = "wasm32")]
pub mod wasm;
