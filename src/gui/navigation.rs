use crate::gui::theme::Theme;
use eframe::egui::{
    self, pos2, vec2, Color32, CornerRadius, Rect, RichText, Stroke, Ui, UiBuilder,
};

#[derive(Clone, Debug)]
pub struct ProjectCardInfo {
    pub id: String,
    pub title: String,
    pub pinyin: String,
    pub duration_str: String,
    pub modified_time: String,
    pub clips_count: usize,
    pub quick_key: char,
    pub gradient_colors: (Color32, Color32),
}

pub struct NavigationState {
    pub selected_index: usize, // 0 为“+ 新建项目”，1.. 为已有项目
    pub is_search_active: bool,
    pub search_query: String,
    pub is_quick_jump_active: bool,
    pub projects: Vec<ProjectCardInfo>,
    pub deleted_projects_stack: Vec<ProjectCardInfo>,
    pub clipboard_project: Option<ProjectCardInfo>,
    pub status_message: Option<(String, f64)>,
    pub show_new_project_modal: bool,
    pub new_project_input: String,
    pub show_rename_project_modal: bool,
    pub rename_project_input: String,
}

impl Default for NavigationState {
    fn default() -> Self {
        let sample_projects = vec![
            ProjectCardInfo {
                id: "proj_1".into(),
                title: "Summer Vlog 2026".into(),
                pinyin: "xiatianvlog".into(),
                duration_str: "03:45".into(),
                modified_time: "2026-08-16 17:20".into(),
                clips_count: 18,
                quick_key: 'f',
                gradient_colors: (
                    Color32::from_rgb(40, 80, 140),
                    Color32::from_rgb(20, 35, 60),
                ),
            },
            ProjectCardInfo {
                id: "proj_2".into(),
                title: "Product Launch Video".into(),
                pinyin: "chanpinfabu".into(),
                duration_str: "01:30".into(),
                modified_time: "2026-08-15 21:05".into(),
                clips_count: 8,
                quick_key: 'j',
                gradient_colors: (
                    Color32::from_rgb(120, 60, 40),
                    Color32::from_rgb(50, 25, 20),
                ),
            },
            ProjectCardInfo {
                id: "proj_3".into(),
                title: "Short Film Edit - Cyberpunk".into(),
                pinyin: "saibopengke".into(),
                duration_str: "08:12".into(),
                modified_time: "2026-08-14 09:30".into(),
                clips_count: 42,
                quick_key: 'k',
                gradient_colors: (
                    Color32::from_rgb(80, 40, 130),
                    Color32::from_rgb(30, 20, 60),
                ),
            },
            ProjectCardInfo {
                id: "proj_4".into(),
                title: "Travel Montage Japan".into(),
                pinyin: "lvyouriben".into(),
                duration_str: "04:55".into(),
                modified_time: "2026-08-12 18:40".into(),
                clips_count: 26,
                quick_key: 'l',
                gradient_colors: (
                    Color32::from_rgb(35, 100, 90),
                    Color32::from_rgb(15, 45, 40),
                ),
            },
            ProjectCardInfo {
                id: "proj_5".into(),
                title: "Music Video Rough Cut".into(),
                pinyin: "yinyeshipin".into(),
                duration_str: "02:18".into(),
                modified_time: "2026-08-10 11:15".into(),
                clips_count: 14,
                quick_key: 'p',
                gradient_colors: (
                    Color32::from_rgb(110, 85, 30),
                    Color32::from_rgb(50, 40, 15),
                ),
            },
            ProjectCardInfo {
                id: "proj_6".into(),
                title: "Tutorial Series Rust & Wgpu".into(),
                pinyin: "jiaocheng".into(),
                duration_str: "15:40".into(),
                modified_time: "2026-08-08 16:50".into(),
                clips_count: 31,
                quick_key: 'r',
                gradient_colors: (
                    Color32::from_rgb(50, 55, 110),
                    Color32::from_rgb(20, 25, 55),
                ),
            },
            ProjectCardInfo {
                id: "proj_7".into(),
                title: "Podcast Audio Master".into(),
                pinyin: "boke".into(),
                duration_str: "45:20".into(),
                modified_time: "2026-08-05 14:10".into(),
                clips_count: 12,
                quick_key: 'm',
                gradient_colors: (Color32::from_rgb(90, 45, 80), Color32::from_rgb(45, 20, 40)),
            },
        ];

        Self {
            selected_index: 0,
            is_search_active: false,
            search_query: String::new(),
            is_quick_jump_active: false,
            projects: sample_projects,
            deleted_projects_stack: Vec::new(),
            clipboard_project: None,
            status_message: None,
            show_new_project_modal: false,
            new_project_input: String::new(),
            show_rename_project_modal: false,
            rename_project_input: String::new(),
        }
    }
}

impl NavigationState {
    pub fn total_cards_count(&self) -> usize {
        1 + self.filtered_indices().len()
    }

    pub fn filtered_indices(&self) -> Vec<usize> {
        if self.search_query.trim().is_empty() {
            (0..self.projects.len()).collect()
        } else {
            let mut matches: Vec<(usize, u32)> = self
                .projects
                .iter()
                .enumerate()
                .filter_map(|(i, p)| {
                    crate::search::PinyinFuzzyMatcher::match_query(&p.title, &self.search_query)
                        .map(|res| (i, res.score))
                })
                .collect();
            // 按得分从高到低排序
            matches.sort_by_key(|b| std::cmp::Reverse(b.1));
            matches.into_iter().map(|(i, _)| i).collect()
        }
    }

    pub fn move_left(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_right(&mut self) {
        let max_idx = self.total_cards_count().saturating_sub(1);
        if self.selected_index < max_idx {
            self.selected_index += 1;
        }
    }

    pub fn move_up(&mut self, columns: usize) {
        if self.selected_index >= columns {
            self.selected_index -= columns;
        }
    }

    pub fn move_down(&mut self, columns: usize) {
        let max_idx = self.total_cards_count().saturating_sub(1);
        if self.selected_index + columns <= max_idx {
            self.selected_index += columns;
        }
    }

    pub fn jump_by_key(&mut self, key_char: char) -> Option<usize> {
        let filtered = self.filtered_indices();
        for (grid_pos, &proj_idx) in filtered.iter().enumerate() {
            if let Some(proj) = self.projects.get(proj_idx) {
                if proj.quick_key.eq_ignore_ascii_case(&key_char) {
                    let actual_index = grid_pos + 1; // +1 因为 0 是新建
                    self.selected_index = actual_index;
                    return Some(actual_index);
                }
            }
        }
        None
    }

    pub fn delete_selected(&mut self) {
        if self.selected_index > 0 {
            let filtered = self.filtered_indices();
            if let Some(&proj_idx) = filtered.get(self.selected_index - 1) {
                if proj_idx < self.projects.len() {
                    let deleted = self.projects.remove(proj_idx);
                    let trash_dir = std::env::temp_dir().join("vcut_trash");
                    let _ = std::fs::create_dir_all(&trash_dir);
                    let backup_file = trash_dir.join(format!("{}.txt", deleted.id));
                    let _ = std::fs::write(
                        &backup_file,
                        format!("id: {}\ntitle: {}\n", deleted.id, deleted.title),
                    );
                    self.set_status(format!(
                        "已将项目 '{}' 移至回收站 (/tmp/vcut_trash)，按 Shift+P 可恢复",
                        deleted.title
                    ));
                    self.deleted_projects_stack.push(deleted);
                    if self.selected_index >= self.total_cards_count() {
                        self.selected_index = self.total_cards_count().saturating_sub(1);
                    }
                }
            }
        }
    }

    pub fn copy_selected(&mut self) {
        if self.selected_index > 0 {
            let filtered = self.filtered_indices();
            if let Some(&proj_idx) = filtered.get(self.selected_index - 1) {
                if let Some(proj) = self.projects.get(proj_idx) {
                    self.clipboard_project = Some(proj.clone());
                    self.set_status(format!("已复制项目 '{}'", proj.title));
                }
            }
        }
    }

    pub fn paste_project(&mut self) {
        if let Some(mut restored) = self.deleted_projects_stack.pop() {
            let title = restored.title.clone();
            restored.modified_time = "刚刚".into();
            self.projects.insert(0, restored);
            self.selected_index = 1;
            self.set_status(format!("已恢复已删除的项目 '{}'", title));
        } else if let Some(clip) = &self.clipboard_project {
            let mut new_proj = clip.clone();
            new_proj.id = format!("proj_{}", self.projects.len() + 1);
            new_proj.title = format!("{} (Copy)", clip.title);
            new_proj.modified_time = "刚刚".into();
            self.projects.insert(0, new_proj);
            self.selected_index = 1;
            self.set_status(format!("已粘贴新项目 '{}'", clip.title));
        }
    }

    /// 新建项目并加入列表首位
    pub fn create_new_project(&mut self, title: &str) -> ProjectCardInfo {
        let clean_title = if title.trim().is_empty() {
            "未命名工程".to_string()
        } else {
            title.trim().to_string()
        };
        let pinyin = crate::search::PinyinFuzzyMatcher::to_pinyin_string(&clean_title);
        let new_id = format!("proj_{}", self.projects.len() + 1);
        let new_card = ProjectCardInfo {
            id: new_id,
            title: clean_title.clone(),
            pinyin,
            duration_str: "00:00".into(),
            modified_time: "刚刚".into(),
            clips_count: 0,
            quick_key: 'a',
            gradient_colors: (
                Color32::from_rgb(50, 90, 150),
                Color32::from_rgb(25, 40, 70),
            ),
        };
        self.projects.insert(0, new_card.clone());
        self.selected_index = 1;
        self.set_status(format!("已新建工程: {}", clean_title));
        new_card
    }

    /// 重命名当前选中的项目
    pub fn rename_selected_project(&mut self, new_title: &str) {
        if self.selected_index > 0 {
            let filtered = self.filtered_indices();
            if let Some(&proj_idx) = filtered.get(self.selected_index - 1) {
                if let Some(proj) = self.projects.get_mut(proj_idx) {
                    let clean = new_title.trim();
                    if !clean.is_empty() {
                        let old = proj.title.clone();
                        proj.title = clean.to_string();
                        proj.pinyin = crate::search::PinyinFuzzyMatcher::to_pinyin_string(clean);
                        self.set_status(format!("已将项目 '{}' 重命名为 '{}'", old, clean));
                    }
                }
            }
        }
    }

    /// 克隆当前选中的项目为副本
    pub fn clone_selected_project(&mut self) -> Option<ProjectCardInfo> {
        if self.selected_index > 0 {
            let filtered = self.filtered_indices();
            if let Some(&proj_idx) = filtered.get(self.selected_index - 1) {
                if let Some(orig) = self.projects.get(proj_idx) {
                    let mut cloned = orig.clone();
                    cloned.id = format!("proj_{}", self.projects.len() + 1);
                    cloned.title = format!("{} (副本)", orig.title);
                    cloned.modified_time = "刚刚".into();
                    self.projects.insert(0, cloned.clone());
                    self.selected_index = 1;
                    self.set_status(format!("已创建副本项目: {}", cloned.title));
                    return Some(cloned);
                }
            }
        }
        None
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), 0.0));
    }
}

pub fn show(ui: &mut Ui, state: &mut NavigationState) {
    let full_rect = ui.max_rect();

    // 绘制暗色背景
    ui.painter().rect_filled(full_rect, 0.0, Theme::BG_APP);

    // 顶部标题栏
    let top_rect = Rect::from_min_size(
        full_rect.min + vec2(30.0, 24.0),
        vec2(full_rect.width() - 60.0, 60.0),
    );
    ui.scope_builder(UiBuilder::new().max_rect(top_rect), |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("✦ VIDEOCUT")
                            .size(24.0)
                            .strong()
                            .color(Theme::ACCENT_CYAN),
                    );
                    ui.label(
                        RichText::new("STUDIO")
                            .size(24.0)
                            .strong()
                            .color(Theme::TEXT_PRIMARY),
                    );
                    ui.add_space(10.0);
                    // Badge
                    let badge_text = RichText::new(" v0.1.0-alpha ")
                        .size(12.0)
                        .color(Theme::ACCENT_BLUE);
                    ui.label(badge_text);
                });
                ui.label(
                    RichText::new("Pure Keyboard-Driven Non-Linear Video Editor")
                        .size(13.0)
                        .color(Theme::TEXT_MUTED),
                );
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if state.is_quick_jump_active {
                    let pill = RichText::new(" [f] 快速跳转模式激活：按下卡片角标字母直接打开 ")
                        .size(13.0)
                        .color(Color32::BLACK)
                        .background_color(Theme::ACCENT_ORANGE);
                    ui.label(pill);
                } else {
                    let hint = RichText::new(format!("共 {} 个项目", state.projects.len()))
                        .size(13.0)
                        .color(Theme::TEXT_SECONDARY);
                    ui.label(hint);
                }
            });
        });
    });

    // 计算网格布局区域
    let grid_rect = Rect::from_min_size(
        full_rect.min + vec2(30.0, 95.0),
        vec2(full_rect.width() - 60.0, full_rect.height() - 175.0),
    );

    let columns = 4.max((grid_rect.width() / 280.0) as usize);
    let card_gap = 16.0;
    let card_width =
        ((grid_rect.width() - (columns as f32 - 1.0) * card_gap) / columns as f32).max(180.0);
    let card_height = (card_width * 0.70).max(170.0);

    let filtered_indices = state.filtered_indices();
    let total_cards = 1 + filtered_indices.len();

    // 绘制卡片滚动区域
    let mut clicked_index = None;
    ui.scope_builder(UiBuilder::new().max_rect(grid_rect), |ui| {
        egui::ScrollArea::vertical()
            .id_salt("nav_grid_scroll")
            .show(ui, |ui| {
                egui::Grid::new("nav_cards_grid")
                    .spacing(vec2(card_gap, card_gap))
                    .min_col_width(card_width)
                    .show(ui, |ui| {
                        for card_idx in 0..total_cards {
                            let is_selected = state.selected_index == card_idx;
                            let (rect, response) = ui.allocate_exact_size(
                                vec2(card_width, card_height),
                                egui::Sense::click(),
                            );
                            let is_hovered = response.hovered();

                            if response.clicked() {
                                clicked_index = Some(card_idx);
                            }

                            let painter = ui.painter_at(rect);

                            if card_idx == 0 {
                                // 绘制 "+ 新建项目" 卡片
                                let border_color = if is_selected {
                                    Theme::ACCENT_CYAN
                                } else if is_hovered {
                                    Theme::ACCENT_BLUE
                                } else {
                                    Theme::BORDER_SUBTLE
                                };
                                let bg_color = if is_selected {
                                    Color32::from_rgb(25, 32, 45)
                                } else if is_hovered {
                                    Theme::BG_CARD_HOVER
                                } else {
                                    Theme::BG_CARD
                                };

                                painter.rect_filled(rect, CornerRadius::same(8), bg_color);
                                painter.rect_stroke(
                                    rect,
                                    CornerRadius::same(8),
                                    Stroke::new(if is_selected { 2.0 } else { 1.0 }, border_color),
                                    egui::StrokeKind::Inside,
                                );

                                // 中心加号与文字
                                let center = rect.center();
                                let icon_rect = Rect::from_center_size(
                                    center - vec2(0.0, 14.0),
                                    vec2(42.0, 42.0),
                                );
                                painter.rect_filled(
                                    icon_rect,
                                    CornerRadius::same(21),
                                    Color32::from_rgb(36, 44, 60),
                                );
                                painter.text(
                                    icon_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "+",
                                    egui::FontId::proportional(26.0),
                                    Theme::ACCENT_CYAN,
                                );

                                painter.text(
                                    center + vec2(0.0, 22.0),
                                    egui::Align2::CENTER_CENTER,
                                    "新建项目",
                                    egui::FontId::proportional(15.0),
                                    if is_selected {
                                        Color32::WHITE
                                    } else {
                                        Theme::TEXT_PRIMARY
                                    },
                                );

                                painter.text(
                                    center + vec2(0.0, 42.0),
                                    egui::Align2::CENTER_CENTER,
                                    "按 Enter 创建",
                                    egui::FontId::proportional(11.0),
                                    Theme::TEXT_MUTED,
                                );

                                if is_selected {
                                    // 浮现角标
                                    let tag_rect = Rect::from_min_size(
                                        rect.min + vec2(8.0, 8.0),
                                        vec2(20.0, 20.0),
                                    );
                                    painter.rect_filled(
                                        tag_rect,
                                        CornerRadius::same(4),
                                        Theme::ACCENT_CYAN,
                                    );
                                    painter.text(
                                        tag_rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "↵",
                                        egui::FontId::proportional(12.0),
                                        Color32::BLACK,
                                    );
                                }
                            } else {
                                // 绘制已有项目卡片
                                let actual_proj_idx = filtered_indices[card_idx - 1];
                                if let Some(proj) = state.projects.get(actual_proj_idx) {
                                    let border_color = if is_selected {
                                        Theme::ACCENT_CYAN
                                    } else if is_hovered {
                                        Theme::ACCENT_BLUE
                                    } else {
                                        Theme::BORDER_SUBTLE
                                    };

                                    let bg_color = if is_selected {
                                        Theme::BG_CARD_ACTIVE
                                    } else if is_hovered {
                                        Theme::BG_CARD_HOVER
                                    } else {
                                        Theme::BG_CARD
                                    };

                                    painter.rect_filled(rect, CornerRadius::same(8), bg_color);
                                    painter.rect_stroke(
                                        rect,
                                        CornerRadius::same(8),
                                        Stroke::new(
                                            if is_selected { 2.0 } else { 1.0 },
                                            border_color,
                                        ),
                                        egui::StrokeKind::Inside,
                                    );

                                    // 顶部封面缩略图区域 (高约 60%)
                                    let thumb_height = card_height * 0.60;
                                    let thumb_rect = Rect::from_min_size(
                                        rect.min,
                                        vec2(card_width, thumb_height),
                                    );

                                    // 绘制渐变色背景模拟缩略图
                                    painter.rect_filled(
                                        thumb_rect,
                                        CornerRadius::same(8),
                                        proj.gradient_colors.0,
                                    );

                                    // 播放图标和时长标签
                                    let play_center = thumb_rect.center();
                                    painter.circle_filled(
                                        play_center,
                                        16.0,
                                        Color32::from_black_alpha(120),
                                    );
                                    painter.text(
                                        play_center + vec2(1.0, 0.0),
                                        egui::Align2::CENTER_CENTER,
                                        "▶",
                                        egui::FontId::proportional(14.0),
                                        Color32::WHITE,
                                    );

                                    // 右下角时长徽章
                                    let dur_pos = thumb_rect.max - vec2(8.0, 8.0);
                                    let dur_bg =
                                        Rect::from_two_pos(dur_pos - vec2(42.0, 16.0), dur_pos);
                                    painter.rect_filled(
                                        dur_bg,
                                        CornerRadius::same(3),
                                        Color32::from_black_alpha(180),
                                    );
                                    painter.text(
                                        dur_bg.center(),
                                        egui::Align2::CENTER_CENTER,
                                        &proj.duration_str,
                                        egui::FontId::monospace(10.0),
                                        Color32::WHITE,
                                    );

                                    // 左上角快捷键字母 (f 模式高亮，平时低调展示)
                                    let badge_pos = thumb_rect.min + vec2(8.0, 8.0);
                                    let badge_bg = Rect::from_min_size(badge_pos, vec2(22.0, 22.0));
                                    let (pill_bg, pill_fg) = if state.is_quick_jump_active {
                                        (Theme::ACCENT_ORANGE, Color32::BLACK)
                                    } else if is_selected {
                                        (Theme::ACCENT_CYAN, Color32::BLACK)
                                    } else {
                                        (Color32::from_black_alpha(180), Theme::TEXT_PRIMARY)
                                    };
                                    painter.rect_filled(badge_bg, CornerRadius::same(4), pill_bg);
                                    painter.text(
                                        badge_bg.center(),
                                        egui::Align2::CENTER_CENTER,
                                        proj.quick_key.to_uppercase(),
                                        egui::FontId::proportional(12.0),
                                        pill_fg,
                                    );

                                    // 下半部信息区域
                                    let info_y = rect.min.y + thumb_height + 8.0;

                                    // 标题
                                    let title_rect = Rect::from_min_size(
                                        pos2(rect.min.x + 10.0, info_y),
                                        vec2(card_width - 20.0, 20.0),
                                    );
                                    let title_color = if is_selected {
                                        Color32::WHITE
                                    } else {
                                        Theme::TEXT_PRIMARY
                                    };
                                    painter.with_clip_rect(title_rect).text(
                                        title_rect.min,
                                        egui::Align2::LEFT_TOP,
                                        &proj.title,
                                        egui::FontId::proportional(14.0),
                                        title_color,
                                    );

                                    // 底部修改时间与切片数
                                    let footer_y = rect.max.y - 18.0;
                                    painter.text(
                                        pos2(rect.min.x + 10.0, footer_y),
                                        egui::Align2::LEFT_BOTTOM,
                                        &proj.modified_time,
                                        egui::FontId::proportional(11.0),
                                        Theme::TEXT_MUTED,
                                    );

                                    painter.text(
                                        pos2(rect.max.x - 10.0, footer_y),
                                        egui::Align2::RIGHT_BOTTOM,
                                        format!("{} clips", proj.clips_count),
                                        egui::FontId::proportional(11.0),
                                        Theme::ACCENT_BLUE,
                                    );
                                }
                            }

                            if (card_idx + 1) % columns == 0 {
                                ui.end_row();
                            }
                        }
                    });
            });
    });

    if let Some(idx) = clicked_index {
        state.selected_index = idx;
    }

    // 底部状态与快捷键常驻控制条
    let bottom_bar_rect = Rect::from_min_size(
        pos2(full_rect.min.x, full_rect.max.y - 50.0),
        vec2(full_rect.width(), 50.0),
    );

    ui.painter()
        .rect_filled(bottom_bar_rect, 0.0, Theme::BG_PANEL);
    ui.painter().line_segment(
        [
            bottom_bar_rect.min,
            pos2(bottom_bar_rect.max.x, bottom_bar_rect.min.y),
        ],
        Stroke::new(1.0, Theme::BORDER_SUBTLE),
    );

    ui.scope_builder(UiBuilder::new().max_rect(bottom_bar_rect), |ui| {
        ui.horizontal_centered(|ui| {
            ui.add_space(20.0);

            // 搜索框区域
            if state.is_search_active {
                ui.label(
                    RichText::new(" 🔍 搜索: ")
                        .color(Theme::ACCENT_CYAN)
                        .strong(),
                );
                let search_edit = ui.add(
                    egui::TextEdit::singleline(&mut state.search_query)
                        .desired_width(220.0)
                        .hint_text("输入标题或拼音..."),
                );
                search_edit.request_focus();
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    state.is_search_active = false;
                }
            } else {
                let search_btn = ui
                    .button(RichText::new(" / 搜索项目 (拼音/英文) ").color(Theme::TEXT_SECONDARY));
                if search_btn.clicked() {
                    state.is_search_active = true;
                }
            }

            ui.add_space(20.0);

            // 状态提示信息 (如“已删除项目”)
            if let Some((ref msg, _)) = state.status_message {
                ui.label(RichText::new(msg).color(Theme::ACCENT_GREEN));
            }

            // 右侧按键提示 Pills
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(20.0);

                let shortcuts = [
                    ("[?]", "帮助"),
                    ("[Shift+P]", "粘贴"),
                    ("[Shift+C]", "克隆"),
                    ("[Shift+Y]", "复制"),
                    ("[Shift+D]", "删除"),
                    ("[r]", "重命名"),
                    ("[n]", "新建"),
                    ("[f]", "字母跳跃"),
                    ("[Enter]", "打开"),
                    ("[H/J/K/L]", "移动"),
                ];

                for (key, desc) in shortcuts.iter() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(*desc).size(12.0).color(Theme::TEXT_MUTED));
                        ui.label(
                            RichText::new(*key)
                                .size(12.0)
                                .color(Theme::ACCENT_CYAN)
                                .strong(),
                        );
                    });
                    ui.add_space(6.0);
                }
            });
        });
    });

    if state.show_new_project_modal {
        draw_new_project_modal(ui, state);
    }

    if state.show_rename_project_modal {
        draw_rename_project_modal(ui, state);
    }
}

/// 绘制新建项目交互弹窗
fn draw_new_project_modal(ui: &mut Ui, state: &mut NavigationState) {
    let full_rect = ui.max_rect();
    let modal_w = 460.0;
    let modal_h = 200.0;
    let modal_rect = Rect::from_center_size(full_rect.center(), vec2(modal_w, modal_h));

    ui.painter().rect_filled(full_rect, 0.0, Color32::from_black_alpha(160));
    ui.painter().rect_filled(modal_rect, CornerRadius::same(10), Theme::BG_PANEL_ALT);
    ui.painter().rect_stroke(
        modal_rect,
        CornerRadius::same(10),
        Stroke::new(1.5, Theme::ACCENT_CYAN),
        egui::StrokeKind::Inside,
    );

    let mut do_create = false;
    let mut do_close = false;

    ui.scope_builder(UiBuilder::new().max_rect(modal_rect.shrink(20.0)), |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("✨ 新建视频剪辑工程").size(16.0).strong().color(Theme::ACCENT_CYAN));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(" ✕ (Esc) ").clicked() {
                        do_close = true;
                    }
                });
            });

            ui.add_space(12.0);
            ui.label(RichText::new("请输入新工程名称:").color(Theme::TEXT_PRIMARY));
            let text_resp = ui.add(
                egui::TextEdit::singleline(&mut state.new_project_input)
                    .hint_text("例如: My Awesome Video 2026")
                    .desired_width(ui.available_width()),
            );
            text_resp.request_focus();

            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("按 Enter 确认创建 | Esc 取消").size(11.0).color(Theme::TEXT_MUTED));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new(" 确认创建 (Enter) ").color(Theme::ACCENT_CYAN).strong()).clicked() {
                        do_create = true;
                    }
                });
            });
        });
    });

    if do_create {
        let title = state.new_project_input.clone();
        state.create_new_project(&title);
        state.show_new_project_modal = false;
    }

    if do_close {
        state.show_new_project_modal = false;
    }
}

/// 绘制重命名项目交互弹窗
fn draw_rename_project_modal(ui: &mut Ui, state: &mut NavigationState) {
    let full_rect = ui.max_rect();
    let modal_w = 460.0;
    let modal_h = 200.0;
    let modal_rect = Rect::from_center_size(full_rect.center(), vec2(modal_w, modal_h));

    ui.painter().rect_filled(full_rect, 0.0, Color32::from_black_alpha(160));
    ui.painter().rect_filled(modal_rect, CornerRadius::same(10), Theme::BG_PANEL_ALT);
    ui.painter().rect_stroke(
        modal_rect,
        CornerRadius::same(10),
        Stroke::new(1.5, Theme::ACCENT_ORANGE),
        egui::StrokeKind::Inside,
    );

    let mut do_rename = false;
    let mut do_close = false;

    ui.scope_builder(UiBuilder::new().max_rect(modal_rect.shrink(20.0)), |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("✏ 重命名工程").size(16.0).strong().color(Theme::ACCENT_ORANGE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(" ✕ (Esc) ").clicked() {
                        do_close = true;
                    }
                });
            });

            ui.add_space(12.0);
            ui.label(RichText::new("请输入新的工程名称:").color(Theme::TEXT_PRIMARY));
            let text_resp = ui.add(
                egui::TextEdit::singleline(&mut state.rename_project_input)
                    .desired_width(ui.available_width()),
            );
            text_resp.request_focus();

            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("按 Enter 确认重命名 | Esc 取消").size(11.0).color(Theme::TEXT_MUTED));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new(" 确认重命名 (Enter) ").color(Theme::ACCENT_ORANGE).strong()).clicked() {
                        do_rename = true;
                    }
                });
            });
        });
    });

    if do_rename {
        let new_title = state.rename_project_input.clone();
        state.rename_selected_project(&new_title);
        state.show_rename_project_modal = false;
    }

    if do_close {
        state.show_rename_project_modal = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_state_grid_moves() {
        let mut state = NavigationState::default();
        assert_eq!(state.selected_index, 0);

        state.move_right();
        assert_eq!(state.selected_index, 1);

        state.move_left();
        assert_eq!(state.selected_index, 0);

        state.move_down(4);
        assert_eq!(state.selected_index, 4);

        state.move_up(4);
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_navigation_quick_jump_and_trash() {
        let mut state = NavigationState::default();
        let jumped = state.jump_by_key('k');
        assert!(jumped.is_some());

        let initial_count = state.projects.len();
        state.delete_selected();
        assert_eq!(state.projects.len(), initial_count - 1);

        // 验证回收站备份文件存在
        let trash_dir = std::env::temp_dir().join("vcut_trash");
        assert!(trash_dir.exists());

        // 恢复项目
        state.paste_project();
        assert_eq!(state.projects.len(), initial_count);
    }

    #[test]
    fn test_navigation_create_rename_clone() {
        let mut state = NavigationState::default();
        let initial_count = state.projects.len();

        // 1. 测试新建工程
        let new_p = state.create_new_project("New Awesome vlog");
        assert_eq!(state.projects.len(), initial_count + 1);
        assert_eq!(state.projects[0].title, "New Awesome vlog");
        assert_eq!(new_p.title, "New Awesome vlog");

        // 2. 测试重命名
        state.selected_index = 1;
        state.rename_selected_project("Renamed vlog 2026");
        assert_eq!(state.projects[0].title, "Renamed vlog 2026");

        // 3. 测试克隆副本
        let cloned = state.clone_selected_project().unwrap();
        assert_eq!(cloned.title, "Renamed vlog 2026 (副本)");
        assert_eq!(state.projects.len(), initial_count + 2);
    }
}
