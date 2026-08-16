use eframe::{egui, Frame};
use crate::gui::Page;

pub struct VideoCutApp {
    current_page: Page,
}

impl Default for VideoCutApp {
    fn default() -> Self {
        Self {
            current_page: Page::Navigation,
        }
    }
}

impl eframe::App for VideoCutApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        // 全局头部导航栏
        egui::Panel::top("global_header").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("VideoCut - Developer Preview");
                ui.separator();
                if ui.button("Nav").clicked() { self.current_page = Page::Navigation; }
                if ui.button("Main").clicked() { self.current_page = Page::MainInterface; }
                if ui.button("File").clicked() { self.current_page = Page::FileBrowser; }
                if ui.button("Editor").clicked() { self.current_page = Page::Editor; }
            });
        });

        // 根据当前状态路由到不同的页面
        egui::CentralPanel::default().show(ui, |ui| {
            match self.current_page {
                Page::Navigation => {
                    crate::gui::navigation::show(ui);
                }
                Page::MainInterface => {
                    crate::gui::main_interface::show(ui);
                }
                Page::FileBrowser => {
                    crate::gui::file_browser::show(ui);
                }
                Page::Editor => {
                    crate::gui::editor::show(ui);
                }
            }
        });
    }
}
