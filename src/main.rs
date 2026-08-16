pub mod app;
pub mod editor;
pub mod effects;
pub mod gui;
pub mod keybinding;
pub mod lua_engine;
pub mod media;
pub mod project;
pub mod rendering;
pub mod search;
pub mod timeline;

fn main() {
    // Initialize logger
    env_logger::init();

    println!("Welcome to VideoCut!");
    // TODO: Initialize egui/eframe application here
}
