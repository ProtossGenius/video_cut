use egui::Ui;

pub fn show(ui: &mut Ui) {
    ui.heading("File Browser");
    ui.label("悬浮的文件导入页面。");
    ui.horizontal(|ui| {
        ui.label("Left: Parent");
        ui.separator();
        ui.label("Center: Current");
        ui.separator();
        ui.label("Right: Preview");
    });
}
