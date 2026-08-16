use egui::Ui;

pub fn show(ui: &mut Ui) {
    egui::Panel::left("editor_text_area").resizable(true).min_size(300.0).show(ui, |ui| {
        ui.heading("Lua Editor");
        ui.label("-- Write lua scripts here");
    });

    egui::Panel::bottom("editor_timeline").resizable(true).min_size(150.0).show(ui, |ui| {
        ui.heading("Timeline (Read-only)");
    });

    egui::CentralPanel::default().show(ui, |ui| {
        ui.heading("Preview");
    });
}
