use egui::Ui;

pub fn show(ui: &mut Ui) {
    egui::Panel::bottom("status_bar").show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label("Status Bar | Mode: NORMAL");
        });
    });

    egui::Panel::left("media_pool").resizable(true).show(ui, |ui| {
        ui.heading("Media Pool");
        ui.label("导入的文件列表");
    });

    egui::Panel::bottom("timeline").resizable(true).min_size(200.0).show(ui, |ui| {
        ui.heading("Timeline");
        ui.label("多轨道区域");
    });

    egui::CentralPanel::default().show(ui, |ui| {
        ui.heading("Viewport");
        ui.label("视频预览区域");
    });
}
