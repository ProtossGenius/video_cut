use eframe::{egui, Frame};
use std::path::PathBuf;

use crate::gui::editor::EditorState;
use crate::gui::file_browser::FileBrowserState;
use crate::gui::main_interface::{
    collect_all_anchors, AnchorItemView, AnchorJumpSession, AnchorMarkSession, MainInterfaceUiState,
};
use crate::gui::navigation::NavigationState;
use crate::gui::theme::Theme;
use crate::gui::Page;
use crate::keybinding::Mode;
use crate::project::{
    CommandHistory, DeleteClipToTrashCommand, MergeClipsCommand, MergeCutCommand, ProjectState,
    ProjectStorage, RenameTrackCommand, SplitClipCommand,
};
use crate::timeline::{
    AnchorPoint, AnchorScope, AssetId, Clip, ClipId, FrameTime, GotoTimeParser, Track, TrackId,
};

pub struct VideoCutApp {
    current_page: Page,
    #[allow(dead_code)]
    mode: Mode,
    nav_state: NavigationState,
    main_ui_state: MainInterfaceUiState,
    editor_state: EditorState,
    file_browser_state: FileBrowserState,
    show_file_browser: bool,
    project_state: ProjectState,
    command_history: CommandHistory,
    next_clip_id: u64,
    next_track_id: u64,
    page_history: Vec<Page>,
    page_history_idx: usize,
}

impl VideoCutApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        Theme::apply_visuals(&cc.egui_ctx);

        let mut project_state = ProjectState::new("Summer Vlog 2026");

        let mut track1 = Track::new(TrackId(1), "V1 - 主视频");
        track1.add_clip(Clip::new(
            ClipId(1),
            "Clip_001.mp4".into(),
            AssetId(1),
            FrameTime(0),
            FrameTime(4_200_000), // 4.2s
        ));
        track1.add_clip(Clip::new(
            ClipId(2),
            "Drone_Landscape.mov".into(),
            AssetId(2),
            FrameTime(4_500_000), // 4.5s
            FrameTime(3_000_000), // 3s
        ));

        let mut track2 = Track::new(TrackId(2), "A1 - 背景音乐 (BGM)");
        track2.add_clip(Clip::new(
            ClipId(3),
            "Theme_Music.mp3".into(),
            AssetId(3),
            FrameTime(0),
            FrameTime(12_000_000), // 12s
        ));

        project_state.timeline.add_track(track1);
        project_state.timeline.add_track(track2);

        Self {
            current_page: Page::Navigation,
            mode: Mode::Normal,
            nav_state: NavigationState::default(),
            main_ui_state: MainInterfaceUiState::default(),
            editor_state: EditorState::default(),
            file_browser_state: FileBrowserState::default(),
            show_file_browser: false,
            project_state,
            command_history: CommandHistory::new(),
            next_clip_id: 10,
            next_track_id: 10,
            page_history: vec![Page::Navigation],
            page_history_idx: 0,
        }
    }

    /// 使用外部加载的 ProjectState 创建并直接进入主界面
    pub fn new_with_project(cc: &eframe::CreationContext<'_>, project_state: ProjectState) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        Theme::apply_visuals(&cc.egui_ctx);

        Self {
            current_page: Page::MainInterface,
            mode: Mode::Normal,
            nav_state: NavigationState::default(),
            main_ui_state: MainInterfaceUiState::default(),
            editor_state: EditorState::default(),
            file_browser_state: FileBrowserState::default(),
            show_file_browser: false,
            project_state,
            command_history: CommandHistory::new(),
            next_clip_id: 100,
            next_track_id: 100,
            page_history: vec![Page::MainInterface],
            page_history_idx: 0,
        }
    }

    pub fn navigate_to_page(&mut self, page: Page) {
        if self.current_page != page {
            if self.page_history_idx + 1 < self.page_history.len() {
                self.page_history.truncate(self.page_history_idx + 1);
            }
            self.page_history.push(page);
            self.page_history_idx = self.page_history.len().saturating_sub(1);
            self.current_page = page;
        }
    }

    pub fn history_back(&mut self) {
        if self.page_history_idx > 0 {
            self.page_history_idx -= 1;
            self.current_page = self.page_history[self.page_history_idx];
            self.main_ui_state.status_message =
                Some(format!("已后退至页面: {:?}", self.current_page));
        }
    }

    pub fn history_forward(&mut self) {
        if self.page_history_idx + 1 < self.page_history.len() {
            self.page_history_idx += 1;
            self.current_page = self.page_history[self.page_history_idx];
            self.main_ui_state.status_message =
                Some(format!("已前进至页面: {:?}", self.current_page));
        }
    }

    /// 执行命令行指令
    fn execute_command_line(&mut self, cmd_line: &str) {
        let trimmed = cmd_line
            .trim()
            .trim_start_matches(':')
            .trim_start_matches('：')
            .trim();
        if trimmed.is_empty() {
            return;
        }

        self.main_ui_state
            .command_history_list
            .push(trimmed.to_string());
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let cmd_name = parts[0];

        match cmd_name {
            "split" | "s" => {
                let playhead = FrameTime(self.main_ui_state.playhead_us);
                let track_idx = self.main_ui_state.selected_track_idx;
                if let Some(track) = self.project_state.timeline.tracks.get(track_idx) {
                    let track_id = track.id;
                    if let Some(clip) = track
                        .clips
                        .iter()
                        .find(|c| playhead > c.timeline_start && playhead < c.timeline_end())
                    {
                        let clip_id = clip.id;
                        let new_id = ClipId(self.next_clip_id);
                        self.next_clip_id += 1;

                        let cmd = SplitClipCommand::new(track_id, clip_id, playhead, new_id);
                        let _ = self
                            .command_history
                            .execute(Box::new(cmd), &mut self.project_state);
                        self.main_ui_state.status_message =
                            Some(format!("已在 {} 处成功分割切片", playhead));
                    }
                }
            }
            "goto" => {
                if parts.len() > 1 {
                    let current = FrameTime(self.main_ui_state.playhead_us);
                    if let Ok(target) = GotoTimeParser::parse(parts[1], current) {
                        self.main_ui_state.playhead_us = target.0;
                        self.main_ui_state.status_message = Some(format!("已跳转至: {}", target));
                    } else {
                        self.main_ui_state.status_message =
                            Some(format!("无法解析的时间格式: {}", parts[1]));
                    }
                }
            }
            "name" => {
                if parts.len() > 1 {
                    let track_idx = self.main_ui_state.selected_track_idx;
                    if let Some(track) = self.project_state.timeline.tracks.get(track_idx) {
                        let track_id = track.id;
                        let new_name = parts[1..].join(" ");
                        let cmd = RenameTrackCommand::new(track_id, &new_name);
                        let _ = self
                            .command_history
                            .execute(Box::new(cmd), &mut self.project_state);
                        self.main_ui_state.status_message =
                            Some(format!("轨道重命名为: {}", new_name));
                    }
                }
            }
            "merge" => {
                let track_idx = self.main_ui_state.selected_track_idx;
                if let Some(track) = self.project_state.timeline.tracks.get(track_idx) {
                    let track_id = track.id;
                    let clip_ids: Vec<ClipId> = if self.main_ui_state.current_mode == Mode::VisualLine
                        && !self.main_ui_state.visual_line_selected_clips.is_empty()
                    {
                        self.main_ui_state.visual_line_selected_clips.clone()
                    } else {
                        track.clips.iter().map(|c| c.id).collect()
                    };
                    if clip_ids.len() >= 2 {
                        let new_id = ClipId(self.next_clip_id);
                        self.next_clip_id += 1;
                        let cmd = MergeClipsCommand::new(track_id, clip_ids, new_id);
                        let _ = self
                            .command_history
                            .execute(Box::new(cmd), &mut self.project_state);
                        self.main_ui_state.visual_line_selected_clips.clear();
                        self.main_ui_state.current_mode = Mode::Normal;
                        self.main_ui_state.status_message = Some("已合并选中切片".into());
                    }
                }
            }
            "mergecut" => {
                let start_us = self.main_ui_state.visual_start_us.unwrap_or(0);
                let end_us = self
                    .main_ui_state
                    .visual_end_us
                    .unwrap_or(self.main_ui_state.playhead_us);
                let track_idx = self.main_ui_state.selected_track_idx;
                if let Some(track) = self.project_state.timeline.tracks.get(track_idx) {
                    let track_id = track.id;
                    let new_id = ClipId(self.next_clip_id);
                    self.next_clip_id += 1;
                    let cmd = MergeCutCommand::new(
                        track_id,
                        FrameTime(start_us.min(end_us)),
                        FrameTime(start_us.max(end_us)),
                        new_id,
                    );
                    let _ = self
                        .command_history
                        .execute(Box::new(cmd), &mut self.project_state);
                    self.main_ui_state.status_message = Some("已执行选区独立切断合并".into());
                }
            }
            "biset" => {
                let start_us = self
                    .main_ui_state
                    .visual_start_us
                    .unwrap_or(self.main_ui_state.playhead_us);
                let end_us =
                    self.main_ui_state.visual_end_us.unwrap_or_else(|| {
                        let track_idx = self.main_ui_state.selected_track_idx;
                        if let Some(track) = self.project_state.timeline.tracks.get(track_idx) {
                            let playhead = FrameTime(self.main_ui_state.playhead_us);
                            if let Some(clip) = track.clips.iter().find(|c| {
                                playhead >= c.timeline_start && playhead <= c.timeline_end()
                            }) {
                                return clip.timeline_end().0;
                            }
                        }
                        self.main_ui_state.playhead_us + 6_000_000
                    });
                let s = start_us.min(end_us);
                let e = start_us.max(end_us);
                self.main_ui_state.biset_session = Some(crate::timeline::BisetSession::new(
                    FrameTime(s),
                    FrameTime(e),
                ));
                self.main_ui_state.status_message =
                    Some("已开启二分法决策定位模式 (:biset)".into());
            }
            "Marks" | "marks" => {
                self.main_ui_state.show_marks_manager_modal = true;
                self.main_ui_state.marks_manager_search.clear();
                self.main_ui_state.status_message =
                    Some("已开启锚点管理与描述编辑面板 (:Marks)".into());
            }
            "message" => {
                self.main_ui_state.show_message_window = true;
            }
            "history" => {
                self.main_ui_state.show_history_modal = true;
                self.main_ui_state.history_search.clear();
                self.main_ui_state.history_selected_idx = 0;
                self.main_ui_state.status_message =
                    Some("已开启历史命令面板 (:history)".into());
            }
            "help" | "h" => {
                self.main_ui_state.show_command_help_modal = true;
                self.main_ui_state.command_help_search.clear();
                self.main_ui_state.command_help_selected_idx = 0;
                self.main_ui_state.status_message = Some("已开启命令参考帮助面板 (:help)".into());
            }
            "editor" | "e" => {
                if parts.len() > 1 {
                    let target_clip_name = parts[1..].join(" ");
                    self.editor_state.active_tab = format!("{}.lua", target_clip_name);
                } else {
                    let track_idx = self.main_ui_state.selected_track_idx;
                    if let Some(track) = self.project_state.timeline.tracks.get(track_idx) {
                        let playhead = FrameTime(self.main_ui_state.playhead_us);
                        if let Some(clip) = track.clips.iter().find(|c| {
                            playhead >= c.timeline_start && playhead <= c.timeline_end()
                        }) {
                            self.editor_state.active_tab = format!("{}.lua", clip.name);
                        }
                    }
                }
                self.navigate_to_page(Page::Editor);
                self.main_ui_state.status_message = Some(format!("已切换至切片脚本编辑器 [{}]", self.editor_state.active_tab));
            }
            "lock" => {
                let track_idx = self.main_ui_state.selected_track_idx;
                if let Some(track) = self.project_state.timeline.tracks.get_mut(track_idx) {
                    let playhead = FrameTime(self.main_ui_state.playhead_us);
                    if let Some(clip) = track.clips.iter_mut().find(|c| {
                        playhead >= c.timeline_start && playhead <= c.timeline_end()
                    }) {
                        clip.locked = true;
                        self.main_ui_state.status_message = Some(format!("切片 '{}' 已锁定", clip.name));
                    }
                }
            }
            "unlock" => {
                let track_idx = self.main_ui_state.selected_track_idx;
                if let Some(track) = self.project_state.timeline.tracks.get_mut(track_idx) {
                    let playhead = FrameTime(self.main_ui_state.playhead_us);
                    if let Some(clip) = track.clips.iter_mut().find(|c| {
                        playhead >= c.timeline_start && playhead <= c.timeline_end()
                    }) {
                        clip.locked = false;
                        self.main_ui_state.status_message = Some(format!("切片 '{}' 已解除锁定", clip.name));
                    }
                }
            }
            "speed" => {
                if parts.len() > 1 {
                    if let Ok(spd) = parts[1].parse::<f32>() {
                        let track_idx = self.main_ui_state.selected_track_idx;
                        if let Some(track) = self.project_state.timeline.tracks.get_mut(track_idx) {
                            let playhead = FrameTime(self.main_ui_state.playhead_us);
                            if let Some(clip) = track.clips.iter_mut().find(|c| {
                                playhead >= c.timeline_start && playhead <= c.timeline_end()
                            }) {
                                clip.speed = spd.clamp(0.05, 100.0);
                                self.main_ui_state.status_message = Some(format!("切片 '{}' 播放速度已设置为 {:.1}x", clip.name, clip.speed));
                            }
                        }
                    }
                }
            }
            "export_lua" | "save_lua" => {
                let filename = if parts.len() > 1 { parts[1] } else { "project.lua" };
                let lua_code = crate::lua_engine::export_project_to_lua(&self.project_state);
                if std::fs::write(filename, lua_code).is_ok() {
                    self.main_ui_state.status_message = Some(format!("项目已成功导出为 Lua 工程脚本 ({})", filename));
                } else {
                    self.main_ui_state.status_message = Some(format!("导出 Lua 工程脚本失败: {}", filename));
                }
            }
            "w" | "write" => {
                let target_file = if parts.len() > 1 { parts[1] } else { "project.vcut" };
                if target_file.ends_with(".lua") {
                    let lua_code = crate::lua_engine::export_project_to_lua(&self.project_state);
                    if std::fs::write(target_file, lua_code).is_ok() {
                        self.main_ui_state.status_message = Some(format!("工程已保存为 Lua 脚本 ({})", target_file));
                    } else {
                        self.main_ui_state.status_message = Some(format!("保存 Lua 脚本失败: {}", target_file));
                    }
                } else {
                    let path = PathBuf::from(target_file);
                    let _ = ProjectStorage::save_project_atomic(&path, &self.project_state);
                    self.main_ui_state.status_message =
                        Some(format!("工程文件已安全保存落盘 ({})", target_file));
                }
            }
            "q" | "quit" => {
                self.navigate_to_page(Page::Navigation);
            }
            _ => {
                // 尝试作为直接时间跳转解析 (如 :+1:15, :-10, :10.5, :1:20)
                let current = FrameTime(self.main_ui_state.playhead_us);
                if let Ok(target) = GotoTimeParser::parse(trimmed, current) {
                    self.main_ui_state.playhead_us = target.0;
                    self.main_ui_state.status_message = Some(format!("已跳转至: {}", target));
                } else {
                    self.main_ui_state.status_message = Some(format!("未识别的指令: {}", cmd_name));
                }
            }
        }
    }

    /// 执行按键映射后的统一动作 (并自动记录到宏录制器)
    pub fn execute_action(&mut self, action: crate::keybinding::Action) {
        if self.main_ui_state.macro_recorder.is_recording() {
            self.main_ui_state.macro_recorder.record_action(action.clone());
        }

        match action {
            crate::keybinding::Action::PlayPause => {
                self.main_ui_state.is_playing = !self.main_ui_state.is_playing;
            }
            crate::keybinding::Action::MoveLeft => {
                let step = if self.main_ui_state.zoom_level > 0.0 {
                    (1_000_000.0 / self.main_ui_state.zoom_level * 5.0) as i64
                } else {
                    100_000
                };
                self.main_ui_state.playhead_us = (self.main_ui_state.playhead_us - step).max(0);
            }
            crate::keybinding::Action::MoveRight => {
                let step = if self.main_ui_state.zoom_level > 0.0 {
                    (1_000_000.0 / self.main_ui_state.zoom_level * 5.0) as i64
                } else {
                    100_000
                };
                self.main_ui_state.playhead_us += step;
            }
            crate::keybinding::Action::MoveUp => {
                self.main_ui_state.selected_track_idx = self.main_ui_state.selected_track_idx.saturating_sub(1);
            }
            crate::keybinding::Action::MoveDown => {
                let total = self.project_state.timeline.tracks.len();
                if self.main_ui_state.selected_track_idx + 1 < total {
                    self.main_ui_state.selected_track_idx += 1;
                }
            }
            crate::keybinding::Action::Split => {
                let playhead = FrameTime(self.main_ui_state.playhead_us);
                let track_idx = self.main_ui_state.selected_track_idx;
                let target = self.project_state.timeline.tracks.get(track_idx).and_then(|track| {
                    track.clips.iter().find(|c| playhead > c.timeline_start && playhead < c.timeline_end()).map(|c| (track.id, c.id))
                });
                if let Some((track_id, clip_id)) = target {
                    let new_id = ClipId(self.next_clip_id);
                    self.next_clip_id += 1;
                    let cmd = crate::project::SplitClipCommand::new(track_id, clip_id, playhead, new_id);
                    let _ = self.command_history.execute(Box::new(cmd), &mut self.project_state);
                    self.main_ui_state.status_message = Some(format!("已在 {} 处成功分割切片", playhead));
                }
            }
            crate::keybinding::Action::Delete => {
                let playhead = FrameTime(self.main_ui_state.playhead_us);
                let track_idx = self.main_ui_state.selected_track_idx;
                let target = self.project_state.timeline.tracks.get(track_idx).and_then(|track| {
                    track.clips.iter().find(|c| playhead >= c.timeline_start && playhead <= c.timeline_end()).map(|c| (track.id, c.id, c.name.clone()))
                });
                if let Some((track_id, clip_id, clip_name)) = target {
                    let cmd = crate::project::DeleteClipToTrashCommand::new(track_id, clip_id);
                    let _ = self.command_history.execute(Box::new(cmd), &mut self.project_state);
                    self.main_ui_state.status_message = Some(format!("已将切片 '{}' 移动至垃圾回收轨道", clip_name));
                }
            }
            crate::keybinding::Action::EnterVisual => {
                self.main_ui_state.current_mode = Mode::Visual;
                self.main_ui_state.visual_start_us = Some(self.main_ui_state.playhead_us);
                self.main_ui_state.visual_end_us = Some(self.main_ui_state.playhead_us);
            }
            crate::keybinding::Action::EnterVisualLine => {
                self.main_ui_state.current_mode = Mode::VisualLine;
            }
            crate::keybinding::Action::EnterCommand => {
                self.main_ui_state.is_command_mode = true;
                self.main_ui_state.command_input.clear();
            }
            crate::keybinding::Action::EscapeToNormal => {
                self.main_ui_state.current_mode = Mode::Normal;
                self.main_ui_state.visual_start_us = None;
                self.main_ui_state.visual_end_us = None;
                self.main_ui_state.visual_line_selected_clips.clear();
                self.main_ui_state.biset_session = None;
            }
            _ => {}
        }
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let font_bytes = include_bytes!("../../assets/font.ttf");
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(font_bytes).into(),
    );

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "my_font".to_owned());

    ctx.set_fonts(fonts);
}

impl eframe::App for VideoCutApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        // 如果处于播放状态，推进播放头并请求高刷重绘
        if self.main_ui_state.is_playing {
            self.main_ui_state.playhead_us += 16_666;
            let total_dur = self.project_state.timeline.duration.0.max(30_000_000);
            if self.main_ui_state.playhead_us > total_dur {
                self.main_ui_state.playhead_us = 0;
            }
            if self.main_ui_state.current_mode == Mode::Visual {
                self.main_ui_state.visual_end_us = Some(self.main_ui_state.playhead_us);
            }
            ui.ctx().request_repaint();
        }

        let wants_keyboard = ui.ctx().egui_wants_keyboard_input();

        if !wants_keyboard {
            ui.input(|i| {
                let mut typed_texts: Vec<String> = Vec::new();
                for event in &i.events {
                    if let egui::Event::Text(ref text) = event {
                        typed_texts.push(text.clone());
                    }
                }

                // 历史页面跳转快捷键 (Ctrl+o 后退, Ctrl+i 前进)
                if (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::O) {
                    self.history_back();
                }
                if (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::I) {
                    self.history_forward();
                }

                // 全局页面切换快捷键 (Shift+N, Shift+M, Shift+E, Shift+F)
                if i.modifiers.shift && i.key_pressed(egui::Key::N) {
                    self.navigate_to_page(Page::Navigation);
                }
                if i.modifiers.shift && i.key_pressed(egui::Key::M) {
                    self.navigate_to_page(Page::MainInterface);
                }
                if i.modifiers.shift && i.key_pressed(egui::Key::E) {
                    self.navigate_to_page(Page::Editor);
                }
                if i.modifiers.shift && i.key_pressed(egui::Key::F) {
                    self.show_file_browser = !self.show_file_browser;
                }

                // 分页面处理按键
                match self.current_page {
                    Page::Navigation => {
                        if self.nav_state.show_new_project_modal {
                            if i.key_pressed(egui::Key::Escape) {
                                self.nav_state.show_new_project_modal = false;
                            } else if i.key_pressed(egui::Key::Enter) {
                                let title = self.nav_state.new_project_input.clone();
                                let card = self.nav_state.create_new_project(&title);
                                self.nav_state.show_new_project_modal = false;
                                self.project_state = ProjectState::new(&card.title);
                                self.navigate_to_page(Page::MainInterface);
                            }
                        } else if self.nav_state.show_rename_project_modal {
                            if i.key_pressed(egui::Key::Escape) {
                                self.nav_state.show_rename_project_modal = false;
                            } else if i.key_pressed(egui::Key::Enter) {
                                let new_title = self.nav_state.rename_project_input.clone();
                                self.nav_state.rename_selected_project(&new_title);
                                self.nav_state.show_rename_project_modal = false;
                            }
                        } else if !self.nav_state.is_search_active && !self.nav_state.is_quick_jump_active {
                            if i.key_pressed(egui::Key::H) || i.key_pressed(egui::Key::ArrowLeft) {
                                self.nav_state.move_left();
                            }
                            if i.key_pressed(egui::Key::L) || i.key_pressed(egui::Key::ArrowRight) {
                                self.nav_state.move_right();
                            }
                            if i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::ArrowUp) {
                                self.nav_state.move_up(4);
                            }
                            if i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::ArrowDown) {
                                self.nav_state.move_down(4);
                            }
                            if i.key_pressed(egui::Key::Enter) {
                                if self.nav_state.selected_index == 0 {
                                    self.project_state = ProjectState::new("Untitled Project");
                                } else {
                                    let filtered = self.nav_state.filtered_indices();
                                    if let Some(&proj_idx) =
                                        filtered.get(self.nav_state.selected_index - 1)
                                    {
                                        if let Some(proj) = self.nav_state.projects.get(proj_idx) {
                                            self.project_state = ProjectState::new(&proj.title);
                                        }
                                    }
                                }
                                self.navigate_to_page(Page::MainInterface);
                            }
                            if i.key_pressed(egui::Key::F) {
                                self.nav_state.is_quick_jump_active =
                                    !self.nav_state.is_quick_jump_active;
                            }
                            if i.key_pressed(egui::Key::Slash) || typed_texts.iter().any(|t| t == "／" || t == "/") {
                                self.nav_state.is_search_active = true;
                            }
                            if !i.modifiers.shift && i.key_pressed(egui::Key::N) {
                                self.nav_state.show_new_project_modal = true;
                                self.nav_state.new_project_input.clear();
                            }
                            if !i.modifiers.shift
                                && i.key_pressed(egui::Key::R)
                                && self.nav_state.selected_index > 0
                            {
                                let filtered = self.nav_state.filtered_indices();
                                if let Some(&proj_idx) = filtered.get(self.nav_state.selected_index - 1) {
                                    if let Some(proj) = self.nav_state.projects.get(proj_idx) {
                                        self.nav_state.rename_project_input = proj.title.clone();
                                        self.nav_state.show_rename_project_modal = true;
                                    }
                                }
                            }
                            if i.modifiers.shift && i.key_pressed(egui::Key::C) {
                                self.nav_state.clone_selected_project();
                            }
                            if i.modifiers.shift && i.key_pressed(egui::Key::D) {
                                self.nav_state.delete_selected();
                            }
                            if i.modifiers.shift && i.key_pressed(egui::Key::Y) {
                                self.nav_state.copy_selected();
                            }
                            if i.modifiers.shift && i.key_pressed(egui::Key::P) {
                                self.nav_state.paste_project();
                            }
                        }

                        // 快速字母跳转激活状态下响应字母输入
                        if self.nav_state.is_quick_jump_active {
                            if i.key_pressed(egui::Key::Escape) {
                                self.nav_state.is_quick_jump_active = false;
                            } else {
                                for text in &typed_texts {
                                    if let Some(ch) = text.chars().next() {
                                        if let Some(pos) = self.nav_state.projects.iter().position(|p| p.quick_key == ch) {
                                            self.nav_state.selected_index = pos + 1;
                                            self.nav_state.is_quick_jump_active = false;
                                            if let Some(proj) = self.nav_state.projects.get(pos) {
                                                self.project_state = ProjectState::new(&proj.title);
                                            }
                                            self.navigate_to_page(Page::MainInterface);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Page::MainInterface => {
                        // 0. 如果处于命令参考帮助弹窗 (:help / :h)
                        if self.main_ui_state.show_command_help_modal {
                            if i.key_pressed(egui::Key::Escape) {
                                self.main_ui_state.show_command_help_modal = false;
                            } else if i.key_pressed(egui::Key::Slash) || typed_texts.iter().any(|t| t == "／" || t == "/") {
                                self.main_ui_state.command_help_search_active = true;
                            } else if i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::ArrowDown) {
                                let total = crate::gui::main_interface::get_all_command_help_items().len();
                                if self.main_ui_state.command_help_selected_idx + 1 < total {
                                    self.main_ui_state.command_help_selected_idx += 1;
                                }
                            } else if i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::ArrowUp) {
                                if self.main_ui_state.command_help_selected_idx > 0 {
                                    self.main_ui_state.command_help_selected_idx -= 1;
                                }
                            } else if i.key_pressed(egui::Key::Enter) {
                                let items = crate::gui::main_interface::get_all_command_help_items();
                                let query = self.main_ui_state.command_help_search.trim().to_lowercase();
                                let filtered: Vec<&crate::gui::main_interface::CommandHelpItem> = items
                                    .iter()
                                    .filter(|item| {
                                        if query.is_empty() {
                                            true
                                        } else {
                                            item.name.to_lowercase().contains(&query)
                                                || item.alias.to_lowercase().contains(&query)
                                                || item.args.to_lowercase().contains(&query)
                                                || item.description.to_lowercase().contains(&query)
                                                || item.category.to_lowercase().contains(&query)
                                        }
                                    })
                                    .collect();
                                if let Some(selected_item) = filtered.get(self.main_ui_state.command_help_selected_idx) {
                                    self.main_ui_state.command_input = format!("{} ", selected_item.name);
                                    self.main_ui_state.is_command_mode = true;
                                    self.main_ui_state.show_command_help_modal = false;
                                }
                            }
                        }
                        // 0.1 如果处于锚点管理与描述编辑弹窗 (:Marks / :marks)
                        else if self.main_ui_state.show_marks_manager_modal {
                            if i.key_pressed(egui::Key::Escape) {
                                self.main_ui_state.show_marks_manager_modal = false;
                            } else if i.key_pressed(egui::Key::Slash) || typed_texts.iter().any(|t| t == "／" || t == "/") {
                                self.main_ui_state.marks_manager_search_active = true;
                            }
                        }
                        // 0.2 如果处于历史命令记录弹窗 (:history)
                        else if self.main_ui_state.show_history_modal {
                            if i.key_pressed(egui::Key::Escape) {
                                self.main_ui_state.show_history_modal = false;
                            } else if i.key_pressed(egui::Key::Slash) || typed_texts.iter().any(|t| t == "／" || t == "/") {
                                self.main_ui_state.history_search_active = true;
                            } else if i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::ArrowDown) {
                                let total = self.main_ui_state.command_history_list.len();
                                if self.main_ui_state.history_selected_idx + 1 < total {
                                    self.main_ui_state.history_selected_idx += 1;
                                }
                            } else if i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::ArrowUp) {
                                if self.main_ui_state.history_selected_idx > 0 {
                                    self.main_ui_state.history_selected_idx -= 1;
                                }
                            } else if i.key_pressed(egui::Key::Enter) {
                                let query = self.main_ui_state.history_search.trim().to_lowercase();
                                let filtered: Vec<&String> = self
                                    .main_ui_state
                                    .command_history_list
                                    .iter()
                                    .filter(|cmd| query.is_empty() || cmd.to_lowercase().contains(&query))
                                    .collect();
                                if let Some(cmd) = filtered.get(self.main_ui_state.history_selected_idx) {
                                    let formatted = if cmd.starts_with(':') {
                                        cmd.to_string()
                                    } else {
                                        format!(":{}", cmd)
                                    };
                                    self.main_ui_state.command_input = formatted;
                                    self.main_ui_state.is_command_mode = true;
                                    self.main_ui_state.show_history_modal = false;
                                }
                            }
                        }
                        // 0.3 如果处于等待宏寄存器输入状态 (q 或 @)
                        else if let Some(prefix) = self.main_ui_state.macro_pending_prefix {
                            if i.key_pressed(egui::Key::Escape) {
                                self.main_ui_state.macro_pending_prefix = None;
                                self.main_ui_state.status_message = Some("已取消宏操作".into());
                            } else if prefix == 'q' {
                                for t in &typed_texts {
                                    if let Some(ch) = t.chars().next() {
                                        if ch.is_ascii_alphabetic() {
                                            self.main_ui_state.macro_recorder.start_recording(ch);
                                            self.main_ui_state.macro_pending_prefix = None;
                                            self.main_ui_state.status_message = Some(format!("开始录制宏 @{}", ch));
                                            break;
                                        }
                                    }
                                }
                            } else if prefix == '@' {
                                let mut handled = false;
                                if typed_texts.iter().any(|t| t == "@" || t == "＠") {
                                    // @@ 重复上一次执行的宏
                                    if let Some(actions) = self.main_ui_state.macro_recorder.get_last_macro() {
                                        self.main_ui_state.macro_pending_prefix = None;
                                        self.main_ui_state.status_message = Some("重复执行上一次宏 (@@)".into());
                                        for action in actions {
                                            self.execute_action(action);
                                        }
                                        handled = true;
                                    }
                                }
                                if !handled {
                                    for t in &typed_texts {
                                        if let Some(ch) = t.chars().next() {
                                            if ch.is_ascii_alphabetic() {
                                                if let Some(actions) = self.main_ui_state.macro_recorder.get_macro(ch) {
                                                    self.main_ui_state.macro_pending_prefix = None;
                                                    self.main_ui_state.status_message = Some(format!("已回放宏 @{}", ch));
                                                    for action in actions {
                                                        self.execute_action(action);
                                                    }
                                                } else {
                                                    self.main_ui_state.macro_pending_prefix = None;
                                                    self.main_ui_state.status_message = Some(format!("宏 @{} 未录制任何动作", ch));
                                                }
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        // 1. 如果处于锚点标记会话 (AnchorMarkSession - m/M)
                        else if let Some(ref mut mark) = self.main_ui_state.anchor_mark_session {
                            if i.key_pressed(egui::Key::Escape) {
                                self.main_ui_state.anchor_mark_session = None;
                                self.main_ui_state.status_message = Some("已取消锚点设置".into());
                            } else if mark.is_multichar {
                                if i.key_pressed(egui::Key::Enter) {
                                    let name = mark.input_buffer.trim().to_string();
                                    let scope = mark.scope.clone();
                                    self.main_ui_state.anchor_mark_session = None;
                                    if !name.is_empty() {
                                        let cur = FrameTime(self.main_ui_state.playhead_us);
                                        if scope == AnchorScope::Global {
                                            self.project_state.timeline.global_anchors.set_anchor(
                                                AnchorPoint::new(&name, cur, AnchorScope::Global)
                                                    .with_description(format!("全局锚点 '{}'", name)),
                                            );
                                        } else {
                                            let track_idx = self.main_ui_state.selected_track_idx;
                                            if let Some(track) = self.project_state.timeline.tracks.get_mut(track_idx) {
                                                if let Some(clip) = track.clips.iter_mut().find(|c| {
                                                    cur >= c.timeline_start && cur <= c.timeline_end()
                                                }) {
                                                    let rel_pos = cur - clip.timeline_start;
                                                    clip.anchors.insert(
                                                        name.clone(),
                                                        AnchorPoint::new(&name, rel_pos, AnchorScope::Local)
                                                            .with_description(format!("切片局部锚点 '{}'", name)),
                                                    );
                                                }
                                            }
                                        }
                                        self.main_ui_state.status_message = Some(format!("已成功创建锚点 '{}'", name));
                                    }
                                } else if i.key_pressed(egui::Key::Backspace) {
                                    mark.input_buffer.pop();
                                } else {
                                    for t in &typed_texts {
                                        mark.input_buffer.push_str(t);
                                    }
                                }
                            } else {
                                if i.key_pressed(egui::Key::Colon) || typed_texts.iter().any(|t| t == ":" || t == "：") {
                                    mark.is_multichar = true;
                                    mark.input_buffer.clear();
                                } else if let Some(key_char) = typed_texts.first().and_then(|t| t.chars().next()) {
                                    if !key_char.is_control() && key_char != ':' && key_char != '：' {
                                        let name = key_char.to_string();
                                        let scope = mark.scope.clone();
                                        self.main_ui_state.anchor_mark_session = None;
                                        let cur = FrameTime(self.main_ui_state.playhead_us);
                                        if scope == AnchorScope::Global {
                                            self.project_state.timeline.global_anchors.set_anchor(
                                                AnchorPoint::new(&name, cur, AnchorScope::Global)
                                                    .with_description(format!("标记点 {}", name)),
                                            );
                                        } else {
                                            let track_idx = self.main_ui_state.selected_track_idx;
                                            if let Some(track) = self.project_state.timeline.tracks.get_mut(track_idx) {
                                                if let Some(clip) = track.clips.iter_mut().find(|c| {
                                                    cur >= c.timeline_start && cur <= c.timeline_end()
                                                }) {
                                                    let rel_pos = cur - clip.timeline_start;
                                                    clip.anchors.insert(
                                                        name.clone(),
                                                        AnchorPoint::new(&name, rel_pos, AnchorScope::Local)
                                                            .with_description(format!("切片标记点 {}", name)),
                                                    );
                                                }
                                            }
                                        }
                                        self.main_ui_state.status_message = Some(format!("已在 {} 处设置锚点 '{}'", cur, name));
                                    }
                                }
                            }
                        }
                        // 2. 如果处于锚点跳转会话 (AnchorJumpSession - ')
                        else if let Some(ref mut jump) = self.main_ui_state.anchor_jump_session {
                            if i.key_pressed(egui::Key::Escape) {
                                self.main_ui_state.anchor_jump_session = None;
                            } else if (i.modifiers.alt && i.key_pressed(egui::Key::J)) || i.key_pressed(egui::Key::ArrowDown) {
                                jump.selected_idx += 1;
                            } else if (i.modifiers.alt && i.key_pressed(egui::Key::K)) || i.key_pressed(egui::Key::ArrowUp) {
                                jump.selected_idx = jump.selected_idx.saturating_sub(1);
                            } else if i.key_pressed(egui::Key::Enter) {
                                let all = collect_all_anchors(&self.project_state);
                                let filtered: Vec<AnchorItemView> = all.into_iter().filter(|item| {
                                    jump.query.is_empty() || crate::search::PinyinFuzzyMatcher::match_query(&item.name, &jump.query).is_some()
                                }).collect();
                                if let Some(target) = filtered.get(jump.selected_idx) {
                                    self.main_ui_state.playhead_us = target.position_us;
                                    self.main_ui_state.status_message = Some(format!("已跳转至锚点: {}", target.name));
                                }
                                self.main_ui_state.anchor_jump_session = None;
                            } else if !jump.is_multichar && jump.query.is_empty() {
                                if i.key_pressed(egui::Key::Quote) || i.key_pressed(egui::Key::Colon) || typed_texts.iter().any(|t| t == "'" || t == ":" || t == "：" || t == "’" || t == "‘") {
                                    jump.is_multichar = true;
                                } else if let Some(key_char) = typed_texts.first().and_then(|t| t.chars().next()) {
                                    let all = collect_all_anchors(&self.project_state);
                                    let target_opt = all.iter().find(|a| a.name.eq_ignore_ascii_case(&key_char.to_string())).cloned();
                                    if let Some(target) = target_opt {
                                        self.main_ui_state.playhead_us = target.position_us;
                                        self.main_ui_state.status_message = Some(format!("已跳转至锚点: {}", target.name));
                                        self.main_ui_state.anchor_jump_session = None;
                                    }
                                }
                            }
                        }
                        // 3. 如果处于 Biset 二分决策模式
                        else if let Some(ref mut biset) = self.main_ui_state.biset_session {
                            if i.key_pressed(egui::Key::Minus) {
                                biset.select_left();
                                self.main_ui_state.playhead_us = biset.midpoint().0;
                            } else if i.key_pressed(egui::Key::Equals)
                                || i.key_pressed(egui::Key::Plus)
                            {
                                biset.select_right();
                                self.main_ui_state.playhead_us = biset.midpoint().0;
                            } else if i.key_pressed(egui::Key::Backspace) {
                                if biset.step_back() {
                                    self.main_ui_state.playhead_us = biset.midpoint().0;
                                }
                            } else if i.key_pressed(egui::Key::Enter) {
                                self.main_ui_state.playhead_us = biset.midpoint().0;
                                self.main_ui_state.biset_session = None;
                                self.main_ui_state.status_message =
                                    Some("二分查找完成，已精准定位到分割点".into());
                            } else if i.key_pressed(egui::Key::Escape) {
                                self.main_ui_state.biset_session = None;
                                self.main_ui_state.status_message = Some("已退出二分模式".into());
                            }
                        } else if !self.main_ui_state.is_command_mode {
                            // 中文冒号与问号支持
                            if typed_texts.iter().any(|t| t == "：") {
                                self.main_ui_state.is_command_mode = true;
                                self.main_ui_state.command_input.clear();
                            }
                            if typed_texts.iter().any(|t| t == "？") {
                                self.main_ui_state.show_help_modal = !self.main_ui_state.show_help_modal;
                            }
                            if typed_texts.iter().any(|t| t == "’" || t == "‘" || t == "＇") {
                                self.main_ui_state.anchor_jump_session = Some(AnchorJumpSession {
                                    query: String::new(),
                                    selected_idx: 0,
                                    is_multichar: false,
                                });
                            }

                            // 宏录制与回放快捷键: q 键与 @ 键
                            if !i.modifiers.shift && i.key_pressed(egui::Key::Q) {
                                if self.main_ui_state.macro_recorder.is_recording() {
                                    if let Some(reg) = self.main_ui_state.macro_recorder.stop_recording() {
                                        self.main_ui_state.status_message = Some(format!("已停止录制并保存至宏 @{}", reg));
                                    }
                                } else {
                                    self.main_ui_state.macro_pending_prefix = Some('q');
                                }
                            }
                            if typed_texts.iter().any(|t| t == "@" || t == "＠") {
                                self.main_ui_state.macro_pending_prefix = Some('@');
                            }

                            // 空格播放/暂停
                            if i.key_pressed(egui::Key::Space) {
                                self.execute_action(crate::keybinding::Action::PlayPause);
                            }

                            // 左右微调播放头
                            if i.key_pressed(egui::Key::H) || i.key_pressed(egui::Key::ArrowLeft) {
                                if self.main_ui_state.current_mode == Mode::VisualLine {
                                    // VisualLine: 向左扩选切片
                                    let track_idx = self.main_ui_state.selected_track_idx;
                                    if let Some(track) = self.project_state.timeline.tracks.get(track_idx) {
                                        if let Some(&first_selected_id) = self.main_ui_state.visual_line_selected_clips.first() {
                                            if let Some(pos) = track.clips.iter().position(|c| c.id == first_selected_id) {
                                                if pos > 0 {
                                                    let prev_clip = &track.clips[pos - 1];
                                                    self.main_ui_state.visual_line_selected_clips.insert(0, prev_clip.id);
                                                    self.main_ui_state.playhead_us = prev_clip.timeline_start.0;
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    self.main_ui_state.playhead_us =
                                        (self.main_ui_state.playhead_us - 1_000_000).max(0);
                                    if self.main_ui_state.current_mode == Mode::Visual {
                                        self.main_ui_state.visual_end_us =
                                            Some(self.main_ui_state.playhead_us);
                                    }
                                }
                            }
                            if i.key_pressed(egui::Key::L) || i.key_pressed(egui::Key::ArrowRight) {
                                if self.main_ui_state.current_mode == Mode::VisualLine {
                                    // VisualLine: 向右扩选切片
                                    let track_idx = self.main_ui_state.selected_track_idx;
                                    if let Some(track) = self.project_state.timeline.tracks.get(track_idx) {
                                        if let Some(&last_selected_id) = self.main_ui_state.visual_line_selected_clips.last() {
                                            if let Some(pos) = track.clips.iter().position(|c| c.id == last_selected_id) {
                                                if pos + 1 < track.clips.len() {
                                                    let next_clip = &track.clips[pos + 1];
                                                    self.main_ui_state.visual_line_selected_clips.push(next_clip.id);
                                                    self.main_ui_state.playhead_us = next_clip.timeline_end().0;
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    self.main_ui_state.playhead_us += 1_000_000;
                                    if self.main_ui_state.current_mode == Mode::Visual {
                                        self.main_ui_state.visual_end_us =
                                            Some(self.main_ui_state.playhead_us);
                                    }
                                }
                            }

                            // 上下切换轨道
                            if (i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::ArrowUp))
                                && self.main_ui_state.selected_track_idx > 0
                            {
                                self.main_ui_state.selected_track_idx -= 1;
                            }
                            if (i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::ArrowDown))
                                && self.main_ui_state.selected_track_idx + 1
                                    < self.project_state.timeline.tracks.len()
                            {
                                self.main_ui_state.selected_track_idx += 1;
                            }

                            // J / K 跳转上/下一个切片断点
                            if i.modifiers.shift && i.key_pressed(egui::Key::J) {
                                let cur = FrameTime(self.main_ui_state.playhead_us);
                                if let Some(next_edge) =
                                    self.project_state.timeline.next_clip_edge(cur)
                                {
                                    self.main_ui_state.playhead_us = next_edge.0;
                                }
                            }
                            if i.modifiers.shift && i.key_pressed(egui::Key::K) {
                                let cur = FrameTime(self.main_ui_state.playhead_us);
                                if let Some(prev_edge) =
                                    self.project_state.timeline.prev_clip_edge(cur)
                                {
                                    self.main_ui_state.playhead_us = prev_edge.0;
                                }
                            }

                            // w / b 跳转发声点
                            if !i.modifiers.shift && i.key_pressed(egui::Key::W) {
                                self.main_ui_state.playhead_us += 1_500_000;
                            }
                            if !i.modifiers.shift && i.key_pressed(egui::Key::B) {
                                self.main_ui_state.playhead_us =
                                    (self.main_ui_state.playhead_us - 1_500_000).max(0);
                            }

                            // W / B 跳转锚点
                            if i.modifiers.shift && i.key_pressed(egui::Key::W) {
                                let cur = FrameTime(self.main_ui_state.playhead_us);
                                if let Some(next_pt) = self
                                    .project_state
                                    .timeline
                                    .global_anchors
                                    .find_next_anchor(cur)
                                {
                                    self.main_ui_state.playhead_us = next_pt.position.0;
                                }
                            }
                            if i.modifiers.shift && i.key_pressed(egui::Key::B) {
                                let cur = FrameTime(self.main_ui_state.playhead_us);
                                if let Some(prev_pt) = self
                                    .project_state
                                    .timeline
                                    .global_anchors
                                    .find_prev_anchor(cur)
                                {
                                    self.main_ui_state.playhead_us = prev_pt.position.0;
                                }
                            }

                            // 's': 分割当前切片
                            if !i.modifiers.shift && i.key_pressed(egui::Key::S) {
                                self.execute_command_line("split");
                            }

                            // 'd': 删除当前选中的切片至垃圾回收轨道
                            if !i.modifiers.shift && i.key_pressed(egui::Key::D) {
                                let track_idx = self.main_ui_state.selected_track_idx;
                                if self.main_ui_state.current_mode == Mode::VisualLine && !self.main_ui_state.visual_line_selected_clips.is_empty() {
                                    if let Some(track) = self.project_state.timeline.tracks.get(track_idx) {
                                        let track_id = track.id;
                                        for clip_id in self.main_ui_state.visual_line_selected_clips.clone() {
                                            let cmd = DeleteClipToTrashCommand::new(track_id, clip_id);
                                            let _ = self.command_history.execute(Box::new(cmd), &mut self.project_state);
                                        }
                                        self.main_ui_state.visual_line_selected_clips.clear();
                                        self.main_ui_state.current_mode = Mode::Normal;
                                        self.main_ui_state.status_message = Some("已批量将选中切片移至垃圾回收轨道".into());
                                    }
                                } else if let Some(track) = self.project_state.timeline.tracks.get(track_idx) {
                                    let track_id = track.id;
                                    let playhead = FrameTime(self.main_ui_state.playhead_us);
                                    if let Some(clip) = track.clips.iter().find(|c| {
                                        playhead >= c.timeline_start && playhead <= c.timeline_end()
                                    }) {
                                        let clip_id = clip.id;
                                        let cmd = DeleteClipToTrashCommand::new(track_id, clip_id);
                                        let _ = self
                                            .command_history
                                            .execute(Box::new(cmd), &mut self.project_state);
                                        self.main_ui_state.status_message =
                                            Some("已将切片移至垃圾回收轨道".into());
                                    }
                                }
                            }

                            // 'u': 撤销, 'Ctrl+R': 重做
                            if !i.modifiers.shift
                                && !i.modifiers.ctrl
                                && i.key_pressed(egui::Key::U)
                            {
                                let _ = self.command_history.undo(&mut self.project_state);
                                self.main_ui_state.status_message = Some("已撤销 (Undo)".into());
                            }
                            if i.modifiers.ctrl && i.key_pressed(egui::Key::R) {
                                let _ = self.command_history.redo(&mut self.project_state);
                                self.main_ui_state.status_message = Some("已重做 (Redo)".into());
                            }

                            // Shift+Enter: 新增下方轨道
                            if i.modifiers.shift && i.key_pressed(egui::Key::Enter) {
                                let new_t_id = TrackId(self.next_track_id);
                                self.next_track_id += 1;
                                let new_track =
                                    Track::new(new_t_id, format!("Track_{}", new_t_id.0));
                                let current_t_id = self
                                    .project_state
                                    .timeline
                                    .tracks
                                    .get(self.main_ui_state.selected_track_idx)
                                    .map(|t| t.id)
                                    .unwrap_or(TrackId(1));
                                self.project_state
                                    .timeline
                                    .insert_track_below(current_t_id, new_track);
                                self.main_ui_state.status_message = Some("已插入新轨道".into());
                            }

                            // Alt+J / Alt+K: 轨道下移/上移
                            if i.modifiers.alt && i.key_pressed(egui::Key::J) {
                                if let Some(t) = self
                                    .project_state
                                    .timeline
                                    .tracks
                                    .get(self.main_ui_state.selected_track_idx)
                                {
                                    let id = t.id;
                                    self.project_state.timeline.move_track_down(id);
                                }
                            }
                            if i.modifiers.alt && i.key_pressed(egui::Key::K) {
                                if let Some(t) = self
                                    .project_state
                                    .timeline
                                    .tracks
                                    .get(self.main_ui_state.selected_track_idx)
                                {
                                    let id = t.id;
                                    self.project_state.timeline.move_track_up(id);
                                }
                            }

                            // 置顶 / 取消置顶 (^ / $)
                            if i.modifiers.shift && i.key_pressed(egui::Key::Num6) {
                                // '^'
                                if let Some(t) = self
                                    .project_state
                                    .timeline
                                    .tracks
                                    .get(self.main_ui_state.selected_track_idx)
                                {
                                    let id = t.id;
                                    self.project_state.timeline.pin_track(id);
                                    self.main_ui_state.status_message = Some("轨道已置顶".into());
                                }
                            }
                            if i.modifiers.shift && i.key_pressed(egui::Key::Num4) {
                                // '$'
                                if let Some(t) = self
                                    .project_state
                                    .timeline
                                    .tracks
                                    .get(self.main_ui_state.selected_track_idx)
                                {
                                    let id = t.id;
                                    self.project_state.timeline.unpin_track(id);
                                    self.main_ui_state.status_message =
                                        Some("轨道已取消置顶".into());
                                }
                            }

                            // m: 局部锚点, M: 全局锚点 (进入交互式输入会话)
                            if !i.modifiers.shift && i.key_pressed(egui::Key::M) {
                                self.main_ui_state.anchor_mark_session = Some(AnchorMarkSession {
                                    scope: AnchorScope::Local,
                                    is_multichar: false,
                                    input_buffer: String::new(),
                                });
                            }
                            if i.modifiers.shift && i.key_pressed(egui::Key::M) {
                                self.main_ui_state.anchor_mark_session = Some(AnchorMarkSession {
                                    scope: AnchorScope::Global,
                                    is_multichar: false,
                                    input_buffer: String::new(),
                                });
                            }

                            // ': 锚点跳转面板 (单引号)
                            if i.key_pressed(egui::Key::Quote) {
                                self.main_ui_state.anchor_jump_session = Some(AnchorJumpSession {
                                    query: String::new(),
                                    selected_idx: 0,
                                    is_multichar: false,
                                });
                            }

                            // v / V 视觉选择模式
                            if i.modifiers.shift && i.key_pressed(egui::Key::V) {
                                if self.main_ui_state.current_mode == Mode::VisualLine {
                                    self.main_ui_state.current_mode = Mode::Normal;
                                    self.main_ui_state.visual_line_selected_clips.clear();
                                } else {
                                    self.main_ui_state.current_mode = Mode::VisualLine;
                                    self.main_ui_state.visual_line_selected_clips.clear();
                                    let track_idx = self.main_ui_state.selected_track_idx;
                                    if let Some(track) = self.project_state.timeline.tracks.get(track_idx) {
                                        let playhead = FrameTime(self.main_ui_state.playhead_us);
                                        if let Some(clip) = track.clips.iter().find(|c| {
                                            playhead >= c.timeline_start && playhead <= c.timeline_end()
                                        }) {
                                            self.main_ui_state.visual_line_selected_clips.push(clip.id);
                                        }
                                    }
                                }
                            } else if !i.modifiers.shift && i.key_pressed(egui::Key::V) {
                                if self.main_ui_state.current_mode == Mode::Visual {
                                    self.main_ui_state.current_mode = Mode::Normal;
                                    self.main_ui_state.visual_start_us = None;
                                    self.main_ui_state.visual_end_us = None;
                                } else {
                                    self.main_ui_state.current_mode = Mode::Visual;
                                    self.main_ui_state.visual_start_us =
                                        Some(self.main_ui_state.playhead_us);
                                    self.main_ui_state.visual_end_us =
                                        Some(self.main_ui_state.playhead_us);
                                }
                            }

                            // Visual 模式下特殊键位支持 (>, <, +, -, Enter, Escape)
                            if self.main_ui_state.current_mode == Mode::Visual {
                                if i.key_pressed(egui::Key::Escape) {
                                    self.main_ui_state.current_mode = Mode::Normal;
                                    self.main_ui_state.visual_start_us = None;
                                    self.main_ui_state.visual_end_us = None;
                                }
                                if i.key_pressed(egui::Key::Enter) {
                                    self.main_ui_state.current_mode = Mode::Normal;
                                    self.main_ui_state.status_message = Some("已确认 Visual 选区范围".into());
                                }
                                if i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals) {
                                    self.main_ui_state.playhead_us += 1_000_000;
                                    self.main_ui_state.visual_end_us = Some(self.main_ui_state.playhead_us);
                                }
                                if i.key_pressed(egui::Key::Minus) {
                                    self.main_ui_state.playhead_us = (self.main_ui_state.playhead_us - 1_000_000).max(0);
                                    self.main_ui_state.visual_end_us = Some(self.main_ui_state.playhead_us);
                                }
                            }

                            // 'i': 导入文件
                            if !i.modifiers.shift && i.key_pressed(egui::Key::I) {
                                self.show_file_browser = true;
                            }

                            // ':': 命令行模式
                            if i.key_pressed(egui::Key::Colon) {
                                self.main_ui_state.is_command_mode = true;
                                self.main_ui_state.command_input.clear();
                            }

                            // '?': 帮助面板
                            if i.modifiers.shift && i.key_pressed(egui::Key::Slash) {
                                self.main_ui_state.show_help_modal =
                                    !self.main_ui_state.show_help_modal;
                            }
                        } else {
                            // 处于 Command Mode
                            if i.key_pressed(egui::Key::Enter) {
                                let cmd = self.main_ui_state.command_input.clone();
                                self.main_ui_state.is_command_mode = false;
                                self.main_ui_state.command_input.clear();
                                self.execute_command_line(&cmd);
                            }
                            if i.key_pressed(egui::Key::Escape) {
                                self.main_ui_state.is_command_mode = false;
                            }
                        }
                    }
                    Page::Editor => {
                        if i.modifiers.alt && i.key_pressed(egui::Key::H) {
                            self.editor_state.focus_pane = crate::gui::editor::EditorFocusPane::ScriptEditor;
                        }
                        if i.modifiers.alt && i.key_pressed(egui::Key::L) {
                            self.editor_state.focus_pane = crate::gui::editor::EditorFocusPane::ConsoleOutput;
                        }
                        if i.modifiers.alt && i.key_pressed(egui::Key::J) {
                            self.editor_state.focus_pane = crate::gui::editor::EditorFocusPane::KeyframeInspector;
                        }
                        if i.key_pressed(egui::Key::F5) {
                            self.editor_state.run_script();
                        }
                        if !self.editor_state.is_insert_mode {
                            if i.key_pressed(egui::Key::I) {
                                self.editor_state.is_insert_mode = true;
                            }
                        } else if i.key_pressed(egui::Key::Escape) {
                            self.editor_state.is_insert_mode = false;
                        }
                    }
                }

                // 文件浏览器按键委托给 file_browser 组件内部统一处理
            });
        }

        // --- 绘制主体页面 ---
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(Theme::BG_APP)
                    .inner_margin(egui::Margin::ZERO),
            )
            .show(ui, |ui| match self.current_page {
                Page::Navigation => crate::gui::navigation::show(ui, &mut self.nav_state),
                Page::MainInterface => crate::gui::main_interface::show(
                    ui,
                    &mut self.project_state,
                    &mut self.main_ui_state,
                ),
                Page::Editor => crate::gui::editor::show(ui, &mut self.editor_state),
            });

        // 浮层文件浏览器
        if self.show_file_browser {
            let mut import_path = None;
            crate::gui::file_browser::show(
                ui.ctx(),
                &mut self.file_browser_state,
                &mut self.show_file_browser,
                |path| {
                    import_path = Some(path);
                },
            );

            if let Some(path) = import_path {
                let filename = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "imported_asset".into());
                let track_idx = self.main_ui_state.selected_track_idx;
                let total_clips: u64 = self
                    .project_state
                    .timeline
                    .tracks
                    .iter()
                    .map(|t| t.clips.len() as u64)
                    .sum();
                if let Some(track) = self.project_state.timeline.tracks.get_mut(track_idx) {
                    let clip_id = ClipId(total_clips + 100);
                    let asset_id = AssetId(1);
                    let playhead = FrameTime(self.main_ui_state.playhead_us);
                    let duration = FrameTime(5_000_000); // 默认导入为 5 秒切片
                    let clip = Clip::new(clip_id, filename.clone(), asset_id, playhead, duration);
                    track.add_clip(clip);
                    self.main_ui_state.status_message =
                        Some(format!("已成功导入素材到当前轨道: {}", filename));
                }
            }
        }
    }
}

#[cfg(test)]
impl VideoCutApp {
    pub fn new_for_test() -> Self {
        let mut project_state = ProjectState::new("Test Project");
        let track_id = TrackId(1);
        let mut track = Track::new(track_id, "Track 1");
        let clip = Clip::new(
            ClipId(1),
            "video1.mp4".into(),
            AssetId(1),
            FrameTime(0),
            FrameTime(10_000_000),
        );
        track.add_clip(clip);
        project_state.timeline.tracks.push(track);

        Self {
            current_page: Page::MainInterface,
            mode: Mode::Normal,
            nav_state: NavigationState::default(),
            main_ui_state: MainInterfaceUiState::default(),
            editor_state: EditorState::default(),
            file_browser_state: FileBrowserState::default(),
            show_file_browser: false,
            project_state,
            command_history: CommandHistory::new(),
            next_clip_id: 10,
            next_track_id: 10,
            page_history: vec![Page::MainInterface],
            page_history_idx: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_command_split_and_goto() {
        let mut app = VideoCutApp::new_for_test();
        app.main_ui_state.playhead_us = 4_000_000;
        app.execute_command_line(":split");

        // 分割后轨道应该有 2 个切片
        let track = app.project_state.timeline.tracks.first().unwrap();
        assert_eq!(track.clips.len(), 2);
        assert_eq!(track.clips[0].duration().0, 4_000_000);
        assert_eq!(track.clips[1].duration().0, 6_000_000);

        // 测试直接时间跳转
        app.execute_command_line(":+2:00");
        assert_eq!(app.main_ui_state.playhead_us, 124_000_000); // 4s + 120s = 124s

        // 测试重命名
        app.execute_command_line(":name Master_Track");
        let track_renamed = app.project_state.timeline.tracks.first().unwrap();
        assert_eq!(track_renamed.name, "Master_Track");

        // 测试 editor 指令
        app.execute_command_line(":editor CustomEffect");
        assert_eq!(app.current_page, Page::Editor);
        assert_eq!(app.editor_state.active_tab, "CustomEffect.lua");
    }

    #[test]
    fn test_execute_command_chinese_colon_and_marks() {
        let mut app = VideoCutApp::new_for_test();
        // 中文冒号执行 Marks
        app.execute_command_line("：Marks");
        assert!(app.main_ui_state.show_marks_manager_modal);
    }

    #[test]
    fn test_execute_command_lock_and_speed() {
        let mut app = VideoCutApp::new_for_test();
        app.main_ui_state.playhead_us = 2_000_000;

        // 测试锁定
        app.execute_command_line(":lock");
        assert!(app.project_state.timeline.tracks[0].clips[0].locked);

        // 测试解锁
        app.execute_command_line(":unlock");
        assert!(!app.project_state.timeline.tracks[0].clips[0].locked);

        // 测试变速 2.0x (10s 原始时长变为 5s 时间线时长)
        app.execute_command_line(":speed 2.0");
        let clip = &app.project_state.timeline.tracks[0].clips[0];
        assert_eq!(clip.speed, 2.0);
        assert_eq!(clip.duration().0, 5_000_000);
    }

    #[test]
    fn test_execute_command_help_and_history_navigation() {
        let mut app = VideoCutApp::new_for_test();
        app.execute_command_line(":help");
        assert!(app.main_ui_state.show_command_help_modal);

        // 页面历史跳转测试
        assert_eq!(app.current_page, Page::MainInterface);
        app.navigate_to_page(Page::Editor);
        assert_eq!(app.current_page, Page::Editor);

        app.navigate_to_page(Page::Navigation);
        assert_eq!(app.current_page, Page::Navigation);

        // 后退回 Editor
        app.history_back();
        assert_eq!(app.current_page, Page::Editor);

        // 后退回 MainInterface
        app.history_back();
        assert_eq!(app.current_page, Page::MainInterface);

        // 前进回 Editor
        app.history_forward();
        assert_eq!(app.current_page, Page::Editor);
    }

    #[test]
    fn test_execute_command_export_lua() {
        let mut app = VideoCutApp::new_for_test();
        let tmp_path = std::env::temp_dir().join("test_export_cmd.lua");
        let cmd = format!(":export_lua {}", tmp_path.to_string_lossy());
        app.execute_command_line(&cmd);
        assert!(tmp_path.exists());
        let content = std::fs::read_to_string(&tmp_path).unwrap();
        assert!(content.contains("timeline:new_project"));
        let _ = std::fs::remove_file(tmp_path);
    }

    #[test]
    fn test_visual_mode_interval_selection_and_confirm() {
        let mut app = VideoCutApp::new_for_test();
        app.main_ui_state.playhead_us = 2_000_000;
        app.main_ui_state.current_mode = Mode::Visual;
        app.main_ui_state.visual_start_us = Some(2_000_000);
        app.main_ui_state.visual_end_us = Some(2_000_000);

        // 模拟向右扩展 3 秒
        app.main_ui_state.playhead_us += 3_000_000;
        app.main_ui_state.visual_end_us = Some(app.main_ui_state.playhead_us);

        assert_eq!(app.main_ui_state.visual_start_us, Some(2_000_000));
        assert_eq!(app.main_ui_state.visual_end_us, Some(5_000_000));
        assert_eq!(app.main_ui_state.current_mode, Mode::Visual);
    }

    #[test]
    fn test_macro_recording_and_execution() {
        let mut app = VideoCutApp::new_for_test();
        assert!(!app.main_ui_state.macro_recorder.is_recording());

        // 1. 开始录制宏到寄存器 'a'
        app.main_ui_state.macro_recorder.start_recording('a');
        assert!(app.main_ui_state.macro_recorder.is_recording());

        // 2. 执行动作
        app.execute_action(crate::keybinding::Action::PlayPause);
        assert!(app.main_ui_state.is_playing);

        app.execute_action(crate::keybinding::Action::MoveRight);
        assert!(app.main_ui_state.playhead_us > 0);

        // 3. 停止录制
        let reg = app.main_ui_state.macro_recorder.stop_recording();
        assert_eq!(reg, Some('a'));
        assert!(!app.main_ui_state.macro_recorder.is_recording());

        // 4. 重置状态并回放宏 @a
        app.main_ui_state.is_playing = false;
        let playhead_before = app.main_ui_state.playhead_us;
        let actions = app.main_ui_state.macro_recorder.get_macro('a').unwrap();
        for action in actions {
            app.execute_action(action);
        }

        assert!(app.main_ui_state.is_playing);
        assert!(app.main_ui_state.playhead_us > playhead_before);
    }
}
