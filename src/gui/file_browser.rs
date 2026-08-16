use crate::gui::theme::Theme;
use eframe::egui::{self, Color32, CornerRadius, Rect, RichText, Stroke, UiBuilder};
use std::fs;
use std::path::{Path, PathBuf};

const MEDIA_EXTENSIONS: &[&str] = &[
    "mp4", "mov", "mkv", "avi", "webm", "flv", "m4v", "wav", "mp3", "aac", "flac", "ogg", "m4a",
    "png", "jpg", "jpeg", "webp", "gif", "bmp",
];

pub fn is_media_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_ascii_lowercase();
        MEDIA_EXTENSIONS.contains(&ext_lower.as_str())
    } else {
        false
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileEntry {
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_media: bool,
    pub extension: String,
    pub name: String,
    pub size_str: String,
}

impl FileEntry {
    pub fn new(path: PathBuf) -> Self {
        let is_dir = path.is_dir();
        let is_media = is_media_file(&path);
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let size_str = if is_dir {
            "文件夹".into()
        } else if let Ok(meta) = path.metadata() {
            let bytes = meta.len();
            if bytes > 1024 * 1024 * 1024 {
                format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
            } else if bytes > 1024 * 1024 {
                format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
            } else if bytes > 1024 {
                format!("{:.0} KB", bytes as f64 / 1024.0)
            } else {
                format!("{} B", bytes)
            }
        } else {
            "--".into()
        };

        Self {
            path,
            is_dir,
            is_media,
            extension,
            name,
            size_str,
        }
    }
}

pub struct FileBrowserState {
    pub current_dir: PathBuf,
    pub selected_index: usize,
    pub filter_media: bool,
    pub entries: Vec<FileEntry>,
    pub parent_entries: Vec<FileEntry>,
    pub preview_entries: Vec<FileEntry>,
    pub status_notice: Option<String>,
}

impl Default for FileBrowserState {
    fn default() -> Self {
        let current_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let mut state = Self {
            current_dir,
            selected_index: 0,
            filter_media: true,
            entries: Vec::new(),
            parent_entries: Vec::new(),
            preview_entries: Vec::new(),
            status_notice: None,
        };
        state.refresh();
        state
    }
}

impl FileBrowserState {
    fn read_dir(dir: &Path, filter_media: bool) -> Vec<FileEntry> {
        let mut entries = Vec::new();
        if let Ok(rd) = fs::read_dir(dir) {
            for entry in rd.flatten() {
                if entry.file_name().to_string_lossy().starts_with('.') {
                    continue;
                }
                let fe = FileEntry::new(entry.path());
                if !filter_media || fe.is_dir || fe.is_media {
                    entries.push(fe);
                }
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
        self.entries = Self::read_dir(&self.current_dir, self.filter_media);
        if self.selected_index >= self.entries.len() && !self.entries.is_empty() {
            self.selected_index = self.entries.len() - 1;
        } else if self.entries.is_empty() {
            self.selected_index = 0;
        }

        self.parent_entries = if let Some(parent) = self.current_dir.parent() {
            Self::read_dir(parent, false)
        } else {
            Vec::new()
        };

        self.update_preview();
    }

    pub fn update_preview(&mut self) {
        if let Some(entry) = self.entries.get(self.selected_index) {
            if entry.is_dir {
                self.preview_entries = Self::read_dir(&entry.path, self.filter_media);
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
            self.current_dir = parent.to_path_buf();
            self.selected_index = 0;
            self.refresh();
        }
    }

    pub fn copy_current_dir(&mut self) -> String {
        let path_str = self.current_dir.to_string_lossy().to_string();
        self.status_notice = Some(format!("已复制目录路径: {}", path_str));
        path_str
    }

    pub fn copy_selected_file(&mut self) -> Option<String> {
        if let Some(entry) = self.entries.get(self.selected_index) {
            let path_str = entry.path.to_string_lossy().to_string();
            self.status_notice = Some(format!("已复制完整路径: {}", path_str));
            Some(path_str)
        } else {
            None
        }
    }

    pub fn toggle_filter(&mut self) {
        self.filter_media = !self.filter_media;
        self.refresh();
        let status = if self.filter_media {
            "已开启媒体文件过滤 (仅显示视频/音频/图片)"
        } else {
            "已关闭文件过滤 (显示所有文件)"
        };
        self.status_notice = Some(status.into());
    }

    pub fn selected_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.selected_index)
    }
}

pub fn show(
    ctx: &egui::Context,
    state: &mut FileBrowserState,
    is_open: &mut bool,
    on_import: impl FnOnce(PathBuf),
) {
    if !*is_open {
        return;
    }

    let mut open = *is_open;
    let mut import_target: Option<PathBuf> = None;
    let mut should_close = false;

    // 快捷键拦截
    ctx.input(|i| {
        if i.key_pressed(egui::Key::Escape) || (!i.modifiers.shift && i.key_pressed(egui::Key::X)) {
            open = false;
        }

        if i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::ArrowDown) {
            state.move_down();
        }
        if i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::ArrowUp) {
            state.move_up();
        }

        // Backspace: 返回上一层
        if i.key_pressed(egui::Key::Backspace) {
            state.go_up_dir();
        }

        // Shift+Enter: 选中当前所在文件夹
        if i.modifiers.shift && i.key_pressed(egui::Key::Enter) {
            import_target = Some(state.current_dir.clone());
            open = false;
        }
        // Enter: 进入文件夹 或 选中文件导入
        else if !i.modifiers.shift && i.key_pressed(egui::Key::Enter) {
            if let Some(entry) = state.selected_entry().cloned() {
                if entry.is_dir {
                    state.enter_dir();
                } else {
                    import_target = Some(entry.path);
                    open = false;
                }
            }
        }

        // 'Y' (大写): 复制当前所在绝对路径
        if i.modifiers.shift && i.key_pressed(egui::Key::Y) {
            let p = state.copy_current_dir();
            ctx.copy_text(p);
        }
        // 'y' (小写): 复制选中文件完整路径
        else if !i.modifiers.shift && i.key_pressed(egui::Key::Y) {
            if let Some(p) = state.copy_selected_file() {
                ctx.copy_text(p);
            }
        }
    });

    egui::Window::new("🗂 文件导入与浏览 (Miller Columns)")
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(820.0)
        .default_height(500.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            // 顶部导航栏与当前路径
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("当前路径:")
                        .size(13.0)
                        .color(Theme::TEXT_MUTED),
                );
                ui.label(
                    RichText::new(state.current_dir.to_string_lossy())
                        .size(13.0)
                        .strong()
                        .color(Theme::ACCENT_CYAN),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(if state.filter_media {
                            "🎬 媒体过滤: ON"
                        } else {
                            "📄 媒体过滤: OFF"
                        })
                        .clicked()
                    {
                        state.toggle_filter();
                    }
                });
            });

            ui.separator();

            // 三列 Miller Columns 布局
            let available_width = ui.available_width();
            let left_w = available_width * 0.25;
            let center_w = available_width * 0.42;
            let right_w = available_width * 0.33;

            ui.horizontal(|ui| {
                // 第一列：父目录 (25%)
                ui.scope_builder(
                    UiBuilder::new().max_rect(Rect::from_min_size(
                        ui.cursor().min,
                        egui::vec2(left_w, ui.available_height() - 36.0),
                    )),
                    |ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new("⬆ 上级目录")
                                    .strong()
                                    .size(12.0)
                                    .color(Theme::TEXT_MUTED),
                            );
                            ui.add_space(4.0);
                            egui::ScrollArea::vertical()
                                .id_salt("fb_parent_scroll")
                                .show(ui, |ui| {
                                    for entry in &state.parent_entries {
                                        let icon = if entry.is_dir { "📁" } else { "📄" };
                                        let mut text =
                                            RichText::new(format!("{} {}", icon, entry.name))
                                                .size(12.5);
                                        if entry.path == state.current_dir {
                                            text = text.strong().color(Theme::ACCENT_CYAN);
                                        } else {
                                            text = text.color(Theme::TEXT_MUTED);
                                        }
                                        ui.label(text);
                                    }
                                });
                        });
                    },
                );

                ui.separator();

                // 第二列：当前目录 (42%)
                ui.scope_builder(
                    UiBuilder::new().max_rect(Rect::from_min_size(
                        ui.cursor().min,
                        egui::vec2(center_w, ui.available_height() - 36.0),
                    )),
                    |ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new("📂 当前目录文件")
                                    .strong()
                                    .size(12.0)
                                    .color(Theme::TEXT_PRIMARY),
                            );
                            ui.add_space(4.0);

                            let mut new_selected_index = None;
                            let mut do_enter_dir = false;

                            egui::ScrollArea::vertical()
                                .id_salt("fb_current_scroll")
                                .show(ui, |ui| {
                                    for (i, entry) in state.entries.iter().enumerate() {
                                        let is_selected = i == state.selected_index;
                                        let icon = if entry.is_dir {
                                            "📁"
                                        } else if entry.is_media {
                                            "🎬"
                                        } else {
                                            "📄"
                                        };

                                        let bg_color = if is_selected {
                                            Theme::ACCENT_CYAN
                                        } else {
                                            Color32::TRANSPARENT
                                        };

                                        let (rect, resp) = ui.allocate_exact_size(
                                            egui::vec2(center_w - 20.0, 24.0),
                                            egui::Sense::click(),
                                        );
                                        let painter = ui.painter_at(rect);
                                        if is_selected {
                                            painter.rect_filled(
                                                rect,
                                                CornerRadius::same(4),
                                                bg_color,
                                            );
                                        } else if resp.hovered() {
                                            painter.rect_filled(
                                                rect,
                                                CornerRadius::same(4),
                                                Theme::BG_CARD_HOVER,
                                            );
                                        }

                                        painter.text(
                                            rect.min + egui::vec2(6.0, 4.0),
                                            egui::Align2::LEFT_TOP,
                                            format!("{} {}", icon, entry.name),
                                            egui::FontId::proportional(12.5),
                                            if is_selected {
                                                Color32::BLACK
                                            } else {
                                                Theme::TEXT_PRIMARY
                                            },
                                        );

                                        painter.text(
                                            rect.max - egui::vec2(6.0, 4.0),
                                            egui::Align2::RIGHT_BOTTOM,
                                            &entry.size_str,
                                            egui::FontId::proportional(10.5),
                                            if is_selected {
                                                Color32::BLACK
                                            } else {
                                                Theme::TEXT_MUTED
                                            },
                                        );

                                        if resp.clicked() {
                                            new_selected_index = Some(i);
                                        }
                                        if resp.double_clicked() && entry.is_dir {
                                            do_enter_dir = true;
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
                    },
                );

                ui.separator();

                // 第三列：富媒体预览 (33%)
                ui.scope_builder(
                    UiBuilder::new().max_rect(Rect::from_min_size(
                        ui.cursor().min,
                        egui::vec2(right_w, ui.available_height() - 36.0),
                    )),
                    |ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new("👁 内容预览")
                                    .strong()
                                    .size(12.0)
                                    .color(Theme::TEXT_PRIMARY),
                            );
                            ui.add_space(4.0);

                            if let Some(selected) = state.entries.get(state.selected_index) {
                                if selected.is_dir {
                                    ui.label(
                                        RichText::new(format!("目录: {}", selected.name))
                                            .size(12.0)
                                            .color(Theme::ACCENT_CYAN),
                                    );
                                    ui.separator();
                                    egui::ScrollArea::vertical()
                                        .id_salt("fb_preview_scroll")
                                        .show(ui, |ui| {
                                            for entry in &state.preview_entries {
                                                let text = format!(
                                                    "{} {}",
                                                    if entry.is_dir { "📁" } else { "📄" },
                                                    entry.name
                                                );
                                                ui.label(
                                                    RichText::new(text)
                                                        .size(11.5)
                                                        .color(Theme::TEXT_MUTED),
                                                );
                                            }
                                        });
                                } else {
                                    ui.label(
                                        RichText::new(format!("文件: {}", selected.name))
                                            .size(13.0)
                                            .strong()
                                            .color(Theme::ACCENT_CYAN),
                                    );
                                    ui.label(
                                        RichText::new(format!("大小: {}", selected.size_str))
                                            .size(11.0)
                                            .color(Theme::TEXT_MUTED),
                                    );
                                    ui.label(
                                        RichText::new(format!("格式: {}", selected.extension.to_uppercase()))
                                            .size(11.0)
                                            .color(Theme::ACCENT_GREEN),
                                    );
                                    ui.add_space(8.0);

                                    // 模拟预览卡片
                                    let prev_box =
                                        ui.allocate_space(egui::vec2(right_w - 20.0, 130.0)).1;
                                    let painter = ui.painter_at(prev_box);
                                    painter.rect_filled(
                                        prev_box,
                                        CornerRadius::same(6),
                                        Color32::from_rgb(25, 35, 50),
                                    );
                                    painter.rect_stroke(
                                        prev_box,
                                        CornerRadius::same(6),
                                        Stroke::new(1.0, Theme::BORDER_MEDIUM),
                                        egui::StrokeKind::Inside,
                                    );

                                    painter.text(
                                        prev_box.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "▶ 媒体解码就绪",
                                        egui::FontId::proportional(13.0),
                                        Color32::WHITE,
                                    );

                                    ui.add_space(10.0);
                                    if ui
                                        .button(
                                            RichText::new(" 📥 导入此文件到时间线 (Enter) ")
                                                .size(12.0)
                                                .color(Theme::ACCENT_CYAN),
                                        )
                                        .clicked()
                                    {
                                        import_target = Some(selected.path.clone());
                                        should_close = true;
                                    }
                                }
                            }
                        });
                    },
                );
            });

            // 底部状态与快捷键说明
            ui.separator();
            ui.horizontal(|ui| {
                if let Some(notice) = &state.status_notice {
                    ui.label(RichText::new(notice).size(11.5).color(Theme::ACCENT_GREEN));
                } else {
                    ui.label(
                        RichText::new("快捷键: [j/k] 移动 | [Enter] 进入/导入 | [Shift+Enter] 选中目录 | [Backspace] 上级 | [y] 复制文件路径 | [Y] 复制目录路径 | [Esc/x] 关闭")
                            .size(11.0)
                            .color(Theme::TEXT_MUTED),
                    );
                }
            });
        });

    if should_close {
        open = false;
    }

    if let Some(target) = import_target {
        on_import(target);
    }

    *is_open = open;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_file_extension_checking() {
        assert!(is_media_file(Path::new("intro.mp4")));
        assert!(is_media_file(Path::new("video.MKV")));
        assert!(is_media_file(Path::new("music.wav")));
        assert!(is_media_file(Path::new("cover.PNG")));
        assert!(!is_media_file(Path::new("code.rs")));
        assert!(!is_media_file(Path::new("script.lua")));
        assert!(!is_media_file(Path::new("README.md")));
    }

    #[test]
    fn test_file_browser_state_navigation() {
        let temp_dir = std::env::temp_dir();
        let mut state = FileBrowserState {
            current_dir: temp_dir.clone(),
            selected_index: 0,
            filter_media: false,
            entries: vec![
                FileEntry {
                    path: temp_dir.join("a.mp4"),
                    is_dir: false,
                    is_media: true,
                    extension: "mp4".into(),
                    name: "a.mp4".into(),
                    size_str: "10 MB".into(),
                },
                FileEntry {
                    path: temp_dir.join("b.mov"),
                    is_dir: false,
                    is_media: true,
                    extension: "mov".into(),
                    name: "b.mov".into(),
                    size_str: "20 MB".into(),
                },
            ],
            parent_entries: vec![],
            preview_entries: vec![],
            status_notice: None,
        };

        assert_eq!(state.selected_index, 0);
        state.move_down();
        assert_eq!(state.selected_index, 1);
        state.move_down(); // 边界
        assert_eq!(state.selected_index, 1);
        state.move_up();
        assert_eq!(state.selected_index, 0);

        let copied = state.copy_selected_file();
        assert!(copied.is_some());
        assert!(copied.unwrap().ends_with("a.mp4"));
    }
}
