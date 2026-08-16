use crate::editor::autocomplete::{Autocomplete, CompletionItem};
use crate::editor::syntax::LuaSyntaxHighlighter;
use crate::gui::theme::Theme;
use crate::lua_engine::LuaRuntime;
use crate::project::ProjectState;
use eframe::egui::{
    self, pos2, vec2, Color32, CornerRadius, Rect, RichText, Stroke, Ui, UiBuilder,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorFocusPane {
    ScriptEditor,
    ConsoleOutput,
    KeyframeInspector,
}

pub struct EditorState {
    pub text: String,
    pub autocomplete: Autocomplete,
    pub show_autocomplete: bool,
    pub completions: Vec<CompletionItem>,
    pub selected_completion: usize,
    pub active_tab: String,
    pub is_insert_mode: bool,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub execution_result: Option<Result<String, String>>,
    pub focus_pane: EditorFocusPane,
    pub clipboard: String,
    pub status_message: Option<String>,
}

impl Default for EditorState {
    fn default() -> Self {
        let sample_lua = r#"-- VideoCut Animation & Camera Script
local clip = VideoClip("landscape_intro.mp4")
local comp = Composition(1920, 1080, 60.0)

-- 1. 定义摄像机运镜平滑缩放
begin_camera()
    frame('0:00')
    actions.add( focuse(rect(0, 0, '100%', '100%'), true) )
    
    frame('+2:00', actions)
    actions.add( focuse(rect('10%', '10%', '80%', '80%'), true) )
    
    frame('end', {
        focuse(rect(0, 0, '100%', '100%'), true)
    })
end_camera()

-- 2. 文本标题淡入与位移动画
begin_animation()
    frame('0:00')
    actions.add( moveto('50%', '60%') )
    actions.add( resize(0.8, '80%', true) )
    
    frame('+1:30', actions)
    actions.add( moveto('50%', '50%') )
end_animation()
"#;

        Self {
            text: sample_lua.into(),
            autocomplete: Autocomplete::new(),
            show_autocomplete: false,
            completions: vec![],
            selected_completion: 0,
            active_tab: "animation.lua".into(),
            is_insert_mode: false,
            cursor_line: 14,
            cursor_col: 22,
            execution_result: None,
            focus_pane: EditorFocusPane::ScriptEditor,
            clipboard: String::new(),
            status_message: None,
        }
    }
}

impl EditorState {
    pub fn run_script(&mut self) {
        match LuaRuntime::new() {
            Ok(runtime) => match runtime.execute(&self.text) {
                Ok(_) => {
                    let anim_count = runtime
                        .get_animation_sequence()
                        .map(|a| a.keyframes.len())
                        .unwrap_or(0);
                    let cam_count = runtime
                        .get_camera_sequence()
                        .map(|c| c.keyframes.len())
                        .unwrap_or(0);
                    self.execution_result = Some(Ok(format!(
                        "执行成功！生成 {} 个动画关键帧，{} 个摄像机关键帧",
                        anim_count, cam_count
                    )));
                }
                Err(e) => {
                    self.execution_result = Some(Err(format!("Lua 执行报错: {}", e)));
                }
            },
            Err(e) => {
                self.execution_result = Some(Err(format!("Lua 引擎初始化失败: {}", e)));
            }
        }
    }
}

fn get_line_up_to_cursor(text: &str, cursor_index: usize) -> Option<String> {
    if cursor_index > text.len() {
        return None;
    }
    let up_to_cursor = &text[..cursor_index];
    let last_line = up_to_cursor.lines().last().unwrap_or("");
    Some(last_line.to_string())
}

pub fn show(ui: &mut Ui, project: &ProjectState, state: &mut EditorState) {
    let full_rect = ui.max_rect();
    ui.painter().rect_filled(full_rect, 0.0, Theme::BG_APP);

    let follow_targets: Vec<(String, String)> = project
        .timeline
        .tracks
        .iter()
        .flat_map(|track| {
            track.clips.iter().map(move |clip| {
                (
                    format!("{}.{}", track.name, clip.name),
                    format!("轨道 {} / 切片 {}", track.name, clip.name),
                )
            })
        })
        .collect();
    state.autocomplete.set_follow_targets(follow_targets);

    let status_height = 32.0;
    let main_height = full_rect.height() - status_height;

    // 左侧代码编辑区占据 50% 宽度，右侧占据 50% 宽度
    let left_width = (full_rect.width() * 0.50).max(420.0);
    let right_width = full_rect.width() - left_width;

    // 快捷键执行 (F5 或 Ctrl+Enter)
    ui.input(|i| {
        if i.key_pressed(egui::Key::F5) || (i.modifiers.ctrl && i.key_pressed(egui::Key::Enter)) {
            state.run_script();
        }
    });

    // --- 左侧：IDE 代码编辑器 ---
    let left_rect = Rect::from_min_size(full_rect.min, vec2(left_width, main_height));
    ui.painter().rect_filled(left_rect, 0.0, Theme::BG_PANEL);
    ui.painter().line_segment(
        [
            pos2(left_rect.max.x, left_rect.min.y),
            pos2(left_rect.max.x, left_rect.max.y),
        ],
        Stroke::new(1.0, Theme::BORDER_MEDIUM),
    );

    ui.scope_builder(UiBuilder::new().max_rect(left_rect), |ui| {
        ui.vertical(|ui| {
            // 1. Tab 栏与运行按钮
            let tab_bar_height = 36.0;
            let tab_rect = Rect::from_min_size(left_rect.min, vec2(left_width, tab_bar_height));
            let painter = ui.painter_at(tab_rect);
            painter.rect_filled(tab_rect, 0.0, Theme::BG_INPUT);
            painter.line_segment(
                [pos2(tab_rect.min.x, tab_rect.max.y), tab_rect.max],
                Stroke::new(1.0, Theme::BORDER_SUBTLE),
            );

            ui.scope_builder(UiBuilder::new().max_rect(tab_rect), |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(8.0);
                    let tabs = ["main.lua", "animation.lua", "camera.lua"];
                    for tab in tabs {
                        let is_active = state.active_tab == tab;
                        let tab_btn =
                            ui.button(RichText::new(format!(" 📜 {} ", tab)).size(12.0).color(
                                if is_active {
                                    Theme::ACCENT_CYAN
                                } else {
                                    Theme::TEXT_MUTED
                                },
                            ));
                        if tab_btn.clicked() {
                            state.active_tab = tab.to_string();
                        }
                    }

                    if ui
                        .button(
                            RichText::new(" ▶ 运行脚本 (F5) ")
                                .size(12.0)
                                .color(Theme::ACCENT_GREEN)
                                .strong(),
                        )
                        .clicked()
                    {
                        state.run_script();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("Tree-sitter Lua AST 驱动")
                                .size(11.0)
                                .color(Theme::TEXT_MUTED),
                        );
                    });
                });
            });

            // 2. 基于 Tree-sitter 的实时代码语法高亮渲染
            let mut layouter = |ui: &egui::Ui,
                                text_buf: &dyn egui::TextBuffer,
                                wrap_width: f32|
             -> std::sync::Arc<egui::Galley> {
                let mut layout_job = egui::text::LayoutJob::default();
                let string = text_buf.as_str();

                let mut highlighter = LuaSyntaxHighlighter::new();
                let spans = highlighter.highlight(string);

                let mut last_idx = 0;
                for span in spans {
                    if span.start > last_idx && span.start <= string.len() {
                        layout_job.append(
                            &string[last_idx..span.start],
                            0.0,
                            egui::TextFormat {
                                font_id: egui::FontId::monospace(13.5),
                                color: Theme::TEXT_PRIMARY,
                                ..Default::default()
                            },
                        );
                    }
                    let end = span.end.min(string.len());
                    if span.start < end {
                        layout_job.append(
                            &string[span.start..end],
                            0.0,
                            egui::TextFormat {
                                font_id: egui::FontId::monospace(13.5),
                                color: span.token_type.color(),
                                ..Default::default()
                            },
                        );
                        last_idx = end;
                    }
                }
                if last_idx < string.len() {
                    layout_job.append(
                        &string[last_idx..],
                        0.0,
                        egui::TextFormat {
                            font_id: egui::FontId::monospace(13.5),
                            color: Theme::TEXT_PRIMARY,
                            ..Default::default()
                        },
                    );
                }

                layout_job.wrap.max_width = wrap_width;
                ui.fonts_mut(|f| f.layout_job(layout_job))
            };

            // 按键交互拦截
            let mut consume_enter = false;
            if state.show_autocomplete && !state.completions.is_empty() {
                ui.input_mut(|i| {
                    if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown) {
                        state.selected_completion =
                            (state.selected_completion + 1).min(state.completions.len() - 1);
                    }
                    if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp) {
                        state.selected_completion = state.selected_completion.saturating_sub(1);
                    }
                    if i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                        || i.consume_key(egui::Modifiers::NONE, egui::Key::Tab)
                    {
                        consume_enter = true;
                    }
                    if i.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                        state.show_autocomplete = false;
                    }
                });
            }

            let output = egui::TextEdit::multiline(&mut state.text)
                .font(egui::FontId::monospace(13.5))
                .code_editor()
                .desired_width(f32::INFINITY)
                .desired_rows(32)
                .layouter(&mut layouter)
                .show(ui);

            if consume_enter {
                if let Some(cursor_range) = output.cursor_range {
                    let cursor_idx: usize = cursor_range.primary.index.into();
                    if let Some(line) = get_line_up_to_cursor(&state.text, cursor_idx) {
                        let prefix = Autocomplete::extract_prefix(&line);

                        if let Some(completion) = state.completions.get(state.selected_completion) {
                            if let Some(insert_text) = completion.label.strip_prefix(&prefix) {
                                state.text.insert_str(cursor_idx, insert_text);
                            }
                        }
                    }
                }
                state.show_autocomplete = false;
                output.response.request_focus();
            }

            // 智能自动补全触发
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

            // 悬浮补全弹窗
            if state.show_autocomplete && !state.completions.is_empty() {
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
                        egui::Frame::new()
                            .fill(Theme::BG_CARD_ACTIVE)
                            .corner_radius(CornerRadius::same(6))
                            .stroke(Stroke::new(1.0, Theme::ACCENT_CYAN))
                            .inner_margin(egui::Margin::same(6))
                            .show(ui, |ui| {
                                ui.set_max_width(280.0);
                                for (i, item) in state.completions.iter().enumerate() {
                                    let is_selected = i == state.selected_completion;
                                    let mut text =
                                        RichText::new(&item.label).monospace().size(12.0);
                                    if is_selected {
                                        text = text
                                            .color(Color32::BLACK)
                                            .background_color(Theme::ACCENT_CYAN);
                                    } else {
                                        text = text.color(Theme::TEXT_PRIMARY);
                                    }
                                    ui.horizontal(|ui| {
                                        ui.label(text);
                                        ui.label(
                                            RichText::new(&item.detail)
                                                .size(10.5)
                                                .color(Theme::TEXT_MUTED),
                                        );
                                    });
                                }
                            });
                    });
            }
        });
    });

    // --- 右侧：预览与运行时执行控制台 ---
    let right_rect = Rect::from_min_size(
        pos2(full_rect.min.x + left_width, full_rect.min.y),
        vec2(right_width, main_height),
    );

    let preview_height = main_height * 0.55;
    let console_height = main_height - preview_height;

    // 1. 右侧上半部：运镜与动效监视器 (Viewport Monitor)
    let preview_rect = Rect::from_min_size(right_rect.min, vec2(right_width, preview_height));
    ui.painter().rect_filled(preview_rect, 0.0, Theme::BG_PANEL);
    ui.painter().line_segment(
        [
            pos2(preview_rect.min.x, preview_rect.max.y),
            preview_rect.max,
        ],
        Stroke::new(1.0, Theme::BORDER_SUBTLE),
    );

    ui.scope_builder(UiBuilder::new().max_rect(preview_rect), |ui| {
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                ui.label(
                    RichText::new("📺 动效与运镜实时渲染监视器")
                        .size(13.0)
                        .strong()
                        .color(Theme::TEXT_PRIMARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(14.0);
                    ui.label(
                        RichText::new("00:01:24:12 / 00:05:32:04")
                            .monospace()
                            .size(12.0)
                            .color(Theme::ACCENT_CYAN),
                    );
                });
            });

            ui.add_space(6.0);
            let monitor_w = (preview_rect.width() - 30.0).max(200.0);
            let monitor_h = (preview_height - 65.0).max(120.0);
            let monitor_rect = ui.allocate_space(vec2(monitor_w, monitor_h)).1;
            let painter = ui.painter_at(monitor_rect);

            painter.rect_filled(
                monitor_rect,
                CornerRadius::same(6),
                Color32::from_rgb(10, 10, 14),
            );
            painter.rect_stroke(
                monitor_rect,
                CornerRadius::same(6),
                Stroke::new(1.0, Theme::BORDER_MEDIUM),
                egui::StrokeKind::Inside,
            );

            // 模拟预览画面
            let sim_frame = monitor_rect.shrink(2.0);
            painter.rect_filled(
                sim_frame,
                CornerRadius::same(4),
                Color32::from_rgb(30, 50, 70),
            );

            // 摄像机镜头框选 (Camera Rect Overlay)
            let cam_box = Rect::from_center_size(
                sim_frame.center(),
                vec2(sim_frame.width() * 0.7, sim_frame.height() * 0.7),
            );
            painter.rect_stroke(
                cam_box,
                CornerRadius::same(0),
                Stroke::new(1.5, Theme::ACCENT_ORANGE),
                egui::StrokeKind::Inside,
            );
            painter.text(
                cam_box.min + vec2(6.0, 6.0),
                egui::Align2::LEFT_TOP,
                "Camera Focus: 80%",
                egui::FontId::monospace(10.0),
                Theme::ACCENT_ORANGE,
            );

            // 播放控制条
            ui.add_space(4.0);
            ui.horizontal_centered(|ui| {
                let _ = ui.button(
                    RichText::new(" ▶ / ⏸ ")
                        .size(12.0)
                        .color(Theme::ACCENT_CYAN),
                );
                let _ = ui.button(RichText::new(" ⏮ ").size(11.0));
                let _ = ui.button(RichText::new(" ⏭ ").size(11.0));
                ui.label(
                    RichText::new("缩放: 100% | 只读轨道联动")
                        .size(11.0)
                        .color(Theme::TEXT_MUTED),
                );
            });

            // 紧凑型切片上下文轨道条 (Compact Context Track Strip)
            ui.add_space(6.0);
            let track_strip_h = 28.0;
            let track_strip_rect = ui.allocate_space(vec2(monitor_w, track_strip_h)).1;
            let p_strip = ui.painter_at(track_strip_rect);
            p_strip.rect_filled(track_strip_rect, CornerRadius::same(4), Theme::BG_CARD);
            p_strip.rect_stroke(track_strip_rect, CornerRadius::same(4), Stroke::new(1.0, Theme::BORDER_SUBTLE), egui::StrokeKind::Inside);

            // 前切片
            let prev_w = monitor_w * 0.22;
            let prev_rect = Rect::from_min_size(track_strip_rect.min + vec2(2.0, 2.0), vec2(prev_w, track_strip_h - 4.0));
            p_strip.rect_filled(prev_rect, CornerRadius::same(3), Color32::from_rgb(30, 35, 45));
            p_strip.text(prev_rect.center(), egui::Align2::CENTER_CENTER, "Clip_Prev", egui::FontId::monospace(10.0), Theme::TEXT_MUTED);

            // 当前正在编辑特效的切片 (Active Editing Clip)
            let curr_w = monitor_w * 0.52;
            let curr_rect = Rect::from_min_size(track_strip_rect.min + vec2(prev_w + 6.0, 2.0), vec2(curr_w, track_strip_h - 4.0));
            p_strip.rect_filled(curr_rect, CornerRadius::same(3), Theme::TRACK_V1);
            p_strip.rect_stroke(curr_rect, CornerRadius::same(3), Stroke::new(1.5, Theme::ACCENT_CYAN), egui::StrokeKind::Inside);
            p_strip.text(curr_rect.center(), egui::Align2::CENTER_CENTER, "🎞 landscape_intro.mp4 (Focus)", egui::FontId::proportional(11.0), Color32::WHITE);

            // 后切片
            let next_x = prev_w + curr_w + 10.0;
            let next_w = (track_strip_rect.max.x - track_strip_rect.min.x - next_x - 2.0).max(10.0);
            let next_rect = Rect::from_min_size(track_strip_rect.min + vec2(next_x, 2.0), vec2(next_w, track_strip_h - 4.0));
            p_strip.rect_filled(next_rect, CornerRadius::same(3), Color32::from_rgb(30, 35, 45));
            p_strip.text(next_rect.center(), egui::Align2::CENTER_CENTER, "Clip_Next", egui::FontId::monospace(10.0), Theme::TEXT_MUTED);
        });
    });

    // 2. 右侧下半部：Lua 运行输出控制台与错误日志 (Execution Console)
    let console_rect = Rect::from_min_size(
        pos2(right_rect.min.x, right_rect.min.y + preview_height),
        vec2(right_width, console_height),
    );
    ui.painter().rect_filled(console_rect, 0.0, Theme::BG_PANEL);

    ui.scope_builder(UiBuilder::new().max_rect(console_rect), |ui| {
        ui.vertical(|ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                ui.label(
                    RichText::new("💻 Lua 运行输出控制台 (Console)")
                        .size(12.0)
                        .strong()
                        .color(Theme::TEXT_SECONDARY),
                );
            });
            ui.separator();

            egui::ScrollArea::vertical()
                .id_salt("editor_console_scroll")
                .show(ui, |ui| {
                    ui.add_space(4.0);
                    if let Some(ref res) = state.execution_result {
                        match res {
                            Ok(msg) => {
                                ui.label(
                                    RichText::new(format!("✔ [SUCCESS] {}", msg))
                                        .monospace()
                                        .size(12.0)
                                        .color(Theme::ACCENT_GREEN),
                                );
                            }
                            Err(err) => {
                                ui.label(
                                    RichText::new(format!("✖ [ERROR] {}", err))
                                        .monospace()
                                        .size(12.0)
                                        .color(Theme::ACCENT_RED),
                                );
                            }
                        }
                    } else {
                        ui.label(
                            RichText::new(
                                "控制台就绪。按 F5 或点击顶部 '▶ 运行脚本' 执行当前 Lua 脚本。",
                            )
                            .size(11.5)
                            .color(Theme::TEXT_MUTED),
                        );
                    }
                });
        });
    });

    // 绘制分栏高亮焦点框
    if state.focus_pane == EditorFocusPane::ScriptEditor {
        ui.painter().rect_stroke(
            left_rect,
            CornerRadius::same(0),
            Stroke::new(1.5, Theme::ACCENT_CYAN),
            egui::StrokeKind::Inside,
        );
    } else if state.focus_pane == EditorFocusPane::KeyframeInspector {
        ui.painter().rect_stroke(
            preview_rect,
            CornerRadius::same(0),
            Stroke::new(1.5, Theme::ACCENT_CYAN),
            egui::StrokeKind::Inside,
        );
    } else if state.focus_pane == EditorFocusPane::ConsoleOutput {
        ui.painter().rect_stroke(
            console_rect,
            CornerRadius::same(0),
            Stroke::new(1.5, Theme::ACCENT_CYAN),
            egui::StrokeKind::Inside,
        );
    }

    // --- 底部编辑器状态栏 (Editor Status Bar) ---
    let status_rect = Rect::from_min_size(
        pos2(full_rect.min.x, full_rect.max.y - status_height),
        vec2(full_rect.width(), status_height),
    );
    ui.painter().rect_filled(status_rect, 0.0, Theme::BG_APP);
    ui.painter().line_segment(
        [status_rect.min, pos2(status_rect.max.x, status_rect.min.y)],
        Stroke::new(1.0, Theme::BORDER_SUBTLE),
    );

    ui.scope_builder(UiBuilder::new().max_rect(status_rect), |ui| {
        ui.horizontal_centered(|ui| {
            ui.add_space(10.0);
            let (mode_badge, bg_col) = if state.is_insert_mode {
                (" INSERT ", Theme::ACCENT_ORANGE)
            } else {
                (" NORMAL ", Theme::ACCENT_GREEN)
            };
            ui.label(
                RichText::new(mode_badge)
                    .size(12.0)
                    .strong()
                    .color(Color32::BLACK)
                    .background_color(bg_col),
            );

            ui.add_space(8.0);
            let pane_name = match state.focus_pane {
                EditorFocusPane::ScriptEditor => "代码编辑",
                EditorFocusPane::ConsoleOutput => "运行控制台",
                EditorFocusPane::KeyframeInspector => "监视器/关键帧",
            };
            ui.label(
                RichText::new(format!("焦点: [{}]", pane_name))
                    .size(12.0)
                    .color(Theme::ACCENT_CYAN)
                    .strong(),
            );

            ui.add_space(10.0);
            ui.label(
                RichText::new("[Alt+h/l/j] 切换分栏 | [F5] 运行脚本 | [i] 进入编辑 | [Esc] 退出编辑")
                    .size(11.5)
                    .color(Theme::TEXT_MUTED),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(14.0);
                ui.label(
                    RichText::new(format!(
                        "[ {}:{} ]  {} lines | utf-8 | unix | lua",
                        state.cursor_line,
                        state.cursor_col,
                        state.text.lines().count()
                    ))
                    .size(11.0)
                    .color(Theme::TEXT_MUTED),
                );
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_state_run_valid_and_invalid_script() {
        let mut state = EditorState::default();
        state.text = r#"
            begin_animation()
            frame('0:00')
            actions.add(moveto('50%', '50%'))
            end_animation()
        "#
        .into();

        state.run_script();
        assert!(state.execution_result.is_some());
        assert!(state.execution_result.as_ref().unwrap().is_ok());

        // 测试语法错误
        state.text = "this is not valid lua code !!!".into();
        state.run_script();
        assert!(state.execution_result.as_ref().unwrap().is_err());
    }
}
