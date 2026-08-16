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

use eframe::egui;

fn main() -> eframe::Result<()> {
    // 初始化日志记录
    env_logger::init();
    
    // 设置 eframe 窗口选项
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_title("VideoCut"),
        ..Default::default()
    };

    // 运行原生应用
    eframe::run_native(
        "VideoCut",
        options,
        Box::new(|cc| Ok(Box::new(app::VideoCutApp::new(cc)))),
    )
}
