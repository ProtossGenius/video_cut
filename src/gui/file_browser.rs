use std::path::PathBuf;
use std::fs;
use eframe::egui;

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub path: PathBuf,
    pub is_dir: bool,
    pub name: String,
}

impl FileEntry {
    pub fn new(path: PathBuf) -> Self {
        let is_dir = path.is_dir();
        let name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        Self { path, is_dir, name }
    }
}

pub struct FileBrowserState {
    pub current_dir: PathBuf,
    pub selected_index: usize,
    pub entries: Vec<FileEntry>,
    pub parent_entries: Vec<FileEntry>,
    pub preview_entries: Vec<FileEntry>,
}

impl Default for FileBrowserState {
    fn default() -> Self {
        let current_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let mut state = Self {
            current_dir,
            selected_index: 0,
            entries: Vec::new(),
            parent_entries: Vec::new(),
            preview_entries: Vec::new(),
        };
        state.refresh();
        state
    }
}

impl FileBrowserState {
    fn read_dir(dir: &PathBuf) -> Vec<FileEntry> {
        let mut entries = Vec::new();
        if let Ok(rd) = fs::read_dir(dir) {
            for entry in rd.flatten() {
                // Ignore hidden files
                if entry.file_name().to_string_lossy().starts_with('.') {
                    continue;
                }
                entries.push(FileEntry::new(entry.path()));
            }
        }
        entries.sort_by(|a, b| {
            if a.is_dir && !b.is_dir {
                std::cmp::Ordering::Less
            } else if !a.is_dir && b.is_dir {
                std::cmp::Ordering::Greater
            } else {
                a.name.cmp(&b.name)
            }
        });
        entries
    }

    pub fn refresh(&mut self) {
        self.entries = Self::read_dir(&self.current_dir);
        if self.selected_index >= self.entries.len() && !self.entries.is_empty() {
            self.selected_index = self.entries.len() - 1;
        } else if self.entries.is_empty() {
            self.selected_index = 0;
        }

        self.parent_entries = if let Some(parent) = self.current_dir.parent() {
            Self::read_dir(&parent.to_path_buf())
        } else {
            Vec::new()
        };

        self.update_preview();
    }

    pub fn update_preview(&mut self) {
        if let Some(entry) = self.entries.get(self.selected_index) {
            if entry.is_dir {
                self.preview_entries = Self::read_dir(&entry.path);
            } else {
                self.preview_entries = Vec::new();
            }
        } else {
            self.preview_entries = Vec::new();
        }
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.update_preview();
        }
    }

    pub fn move_down(&mut self) {
        if !self.entries.is_empty() && self.selected_index < self.entries.len() - 1 {
            self.selected_index += 1;
            self.update_preview();
        }
    }

    pub fn enter_dir(&mut self) {
        if let Some(entry) = self.entries.get(self.selected_index).cloned() {
            if entry.is_dir {
                self.current_dir = entry.path;
                self.selected_index = 0;
                self.refresh();
            }
        }
    }

    pub fn go_up_dir(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            let old_dir = self.current_dir.clone();
            self.current_dir = parent.to_path_buf();
            self.refresh();
            
            if let Some(pos) = self.entries.iter().position(|e| e.path == old_dir) {
                self.selected_index = pos;
            } else {
                self.selected_index = 0;
            }
            self.update_preview();
        }
    }
}

pub fn show(ui: &mut egui::Ui, state: &mut FileBrowserState) {
    ui.heading("文件导入与浏览");
    ui.label(format!("当前目录: {}", state.current_dir.display()));
    ui.separator();

    let available_width = ui.available_width();
    // left: 25%, center: 40%, right: 35%
    let left_w = available_width * 0.25;
    let center_w = available_width * 0.40;
    let right_w = available_width * 0.35;

    ui.horizontal(|ui| {
        // 第一列：父目录
        ui.allocate_ui(egui::vec2(left_w, ui.available_height()), |ui| {
            egui::ScrollArea::vertical().id_salt("parent_scroll").show(ui, |ui| {
                for entry in &state.parent_entries {
                    let mut text = egui::RichText::new(format!("{} {}", if entry.is_dir { "📁" } else { "📄" }, entry.name));
                    if entry.is_dir {
                        text = text.strong();
                    }
                    if entry.path == state.current_dir {
                        text = text.background_color(egui::Color32::from_rgb(50, 50, 100)).color(egui::Color32::WHITE);
                    }
                    ui.label(text);
                }
            });
        });

        ui.separator();

        // 第二列：当前目录
        ui.allocate_ui(egui::vec2(center_w, ui.available_height()), |ui| {
            let mut new_selected_index = None;
            let mut do_enter_dir = false;

            egui::ScrollArea::vertical().id_salt("current_scroll").show(ui, |ui| {
                for (i, entry) in state.entries.iter().enumerate() {
                    let mut text = egui::RichText::new(format!("{} {}", if entry.is_dir { "📁" } else { "📄" }, entry.name)).size(16.0);
                    if entry.is_dir {
                        text = text.strong();
                    }
                    let is_selected = i == state.selected_index;
                    if is_selected {
                        text = text.background_color(egui::Color32::from_rgb(80, 120, 220)).color(egui::Color32::WHITE);
                    }
                    
                    let resp = ui.add(egui::Label::new(text).sense(egui::Sense::click()));
                    if resp.clicked() {
                        new_selected_index = Some(i);
                    }
                    if resp.double_clicked() && entry.is_dir {
                        do_enter_dir = true;
                    }
                    
                    if is_selected {
                        resp.scroll_to_me(Some(egui::Align::Center));
                    }
                }
            });

            if let Some(i) = new_selected_index {
                state.selected_index = i;
                state.update_preview();
            }
            if do_enter_dir {
                state.enter_dir();
            }
        });

        ui.separator();

        // 第三列：预览
        ui.allocate_ui(egui::vec2(right_w, ui.available_height()), |ui| {
            if let Some(selected) = state.entries.get(state.selected_index) {
                if selected.is_dir {
                    ui.heading("文件夹预览");
                    ui.separator();
                    egui::ScrollArea::vertical().id_salt("preview_scroll").show(ui, |ui| {
                        for entry in &state.preview_entries {
                            let text = format!("{} {}", if entry.is_dir { "📁" } else { "📄" }, entry.name);
                            ui.label(text);
                        }
                    });
                } else {
                    ui.heading("文件预览");
                    ui.separator();
                    ui.label("这里将使用 ffmpeg-next 提取第一帧并渲染。");
                    ui.label(format!("路径: {}", selected.path.display()));
                }
            }
        });
    });
}
