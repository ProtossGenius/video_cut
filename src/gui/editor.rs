use egui::{Ui, Color32, RichText};
use crate::editor::autocomplete::{Autocomplete, CompletionItem};

pub struct EditorState {
    pub text: String,
    pub autocomplete: Autocomplete,
    pub show_autocomplete: bool,
    pub completions: Vec<CompletionItem>,
    pub selected_completion: usize,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            text: "-- Write lua scripts here\n".into(),
            autocomplete: Autocomplete::new(),
            show_autocomplete: false,
            completions: vec![],
            selected_completion: 0,
        }
    }
}

fn get_line_up_to_cursor(text: &str, cursor_index: usize) -> Option<String> {
    if cursor_index > text.len() { return None; }
    let up_to_cursor = &text[..cursor_index];
    let last_line = up_to_cursor.lines().last().unwrap_or("");
    Some(last_line.to_string())
}

pub fn show(ui: &mut Ui, state: &mut EditorState) {
    egui::Panel::left("editor_text_area").resizable(true).min_size(400.0).show(ui, |ui| {
        ui.heading("Lua Editor");
        
        let mut layouter = |ui: &egui::Ui, text_buf: &dyn egui::TextBuffer, wrap_width: f32| -> std::sync::Arc<egui::Galley> {
            let mut layout_job = egui::text::LayoutJob::default();
            // 简单的词法染色（以空格和特殊符号简单切分）
            let mut current_word = String::new();
            let string = text_buf.as_str();
            
            let flush_word = |word: &mut String, job: &mut egui::text::LayoutJob| {
                if !word.is_empty() {
                    let color = match word.as_str() {
                        "local" | "function" | "end" | "if" | "then" | "else" => Color32::from_rgb(200, 100, 200),
                        "main" | "animation" | "camera" | "actions" => Color32::from_rgb(100, 200, 255),
                        _ => Color32::WHITE,
                    };
                    job.append(word, 0.0, egui::TextFormat {
                        font_id: egui::FontId::monospace(14.0),
                        color,
                        ..Default::default()
                    });
                    word.clear();
                }
            };

            for c in string.chars() {
                if c.is_alphanumeric() || c == '_' {
                    current_word.push(c);
                } else {
                    flush_word(&mut current_word, &mut layout_job);
                    layout_job.append(&c.to_string(), 0.0, egui::TextFormat {
                        font_id: egui::FontId::monospace(14.0),
                        color: Color32::from_gray(180),
                        ..Default::default()
                    });
                }
            }
            flush_word(&mut current_word, &mut layout_job);
            layout_job.wrap.max_width = wrap_width;
            ui.fonts_mut(|f| f.layout_job(layout_job))
        };

        // 按键拦截（处理自动补全的上下选择和回车确认）
        let mut consume_enter = false;
        if state.show_autocomplete && !state.completions.is_empty() {
            ui.input_mut(|i| {
                if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown) {
                    state.selected_completion = (state.selected_completion + 1).min(state.completions.len() - 1);
                }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp) {
                    state.selected_completion = state.selected_completion.saturating_sub(1);
                }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::Enter) || i.consume_key(egui::Modifiers::NONE, egui::Key::Tab) {
                    consume_enter = true;
                }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                    state.show_autocomplete = false;
                }
            });
        }

        let output = egui::TextEdit::multiline(&mut state.text)
            .font(egui::FontId::monospace(14.0))
            .code_editor()
            .desired_width(f32::INFINITY)
            .desired_rows(30)
            .layouter(&mut layouter)
            .show(ui);
            
        // 如果按下了回车并且在补全状态，则执行补全
        if consume_enter {
            if let Some(cursor_range) = output.cursor_range {
                let cursor_idx: usize = cursor_range.primary.index.into();
                if let Some(line) = get_line_up_to_cursor(&state.text, cursor_idx) {
                    let prefix: String = line.chars().rev().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == ':').collect();
                    let prefix_len = prefix.len();
                    
                    if let Some(completion) = state.completions.get(state.selected_completion) {
                        let insert_text = &completion.label[prefix_len..];
                        // 插入文本
                        state.text.insert_str(cursor_idx, insert_text);
                        // 我们需要光标移动到插入后的位置，但这在egui比较繁琐。简单实现即插入即可
                    }
                }
            }
            state.show_autocomplete = false;
            // 要求重绘并重新聚焦
            output.response.request_focus();
        }

        // 检查是否需要触发补全
        if output.response.has_focus() && output.response.changed() {
            if let Some(cursor_range) = output.cursor_range {
                let cursor_idx: usize = cursor_range.primary.index.into();
                if let Some(line) = get_line_up_to_cursor(&state.text, cursor_idx) {
                    let completions = state.autocomplete.complete(&line);
                    if !completions.is_empty() {
                        state.show_autocomplete = true;
                        state.completions = completions;
                        state.selected_completion = 0;
                    } else {
                        state.show_autocomplete = false;
                    }
                }
            }
        }

        // 绘制补全悬浮窗
        if state.show_autocomplete && !state.completions.is_empty() {
            // 获取光标在屏幕上的位置
            let mut popup_pos = ui.min_rect().min;
            if let Some(cursor_range) = output.cursor_range {
                let cursor = cursor_range.primary;
                let galley = output.galley;
                let cursor_rect = galley.pos_from_cursor(cursor);
                popup_pos = output.galley_pos + cursor_rect.max.to_vec2();
            }

            egui::Area::new("autocomplete_popup".into())
                .fixed_pos(popup_pos)
                .order(egui::Order::Foreground)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.set_max_width(300.0);
                        for (i, item) in state.completions.iter().enumerate() {
                            let is_selected = i == state.selected_completion;
                            let mut text = RichText::new(&item.label).monospace();
                            if is_selected {
                                text = text.color(Color32::BLACK).background_color(Color32::from_rgb(150, 200, 255));
                            }
                            ui.horizontal(|ui| {
                                ui.label(text);
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(&item.detail).color(Color32::from_gray(120)));
                                });
                            });
                        }
                    });
                });
        }
    });

    egui::Panel::bottom("editor_timeline").resizable(true).min_size(150.0).show(ui, |ui| {
        ui.heading("Timeline (Read-only)");
    });

    egui::CentralPanel::default().show(ui, |ui| {
        ui.heading("Preview");
    });
}
