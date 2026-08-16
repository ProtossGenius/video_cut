use crate::gui::theme::Theme;
use crate::project::ProjectState;
use crate::rendering::compositor::{RenderLayer, VideoCompositor};
use crate::timeline::{AnchorScope, Clip, FrameTime};
use eframe::egui::{
    self, pos2, vec2, Color32, CornerRadius, Rect, RichText, Stroke, Ui, UiBuilder,
};

use crate::keybinding::Mode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnchorMarkSession {
    pub scope: AnchorScope,
    pub is_multichar: bool,
    pub input_buffer: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnchorJumpSession {
    pub query: String,
    pub selected_idx: usize,
    pub is_multichar: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnchorItemView {
    pub name: String,
    pub description: String,
    pub position_us: i64,
    pub scope_label: String,
    pub is_global: bool,
}

/// 命令帮助条目数据结构
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandHelpItem {
    pub name: &'static str,
    pub alias: &'static str,
    pub args: &'static str,
    pub description: &'static str,
    pub category: &'static str,
}

/// 获取系统全部已支持命令的详细参考列表
pub fn get_all_command_help_items() -> Vec<CommandHelpItem> {
    vec![
        CommandHelpItem {
            name: ":split",
            alias: ":s",
            args: "",
            description: "在当前播放指针位置分割当前切片",
            category: "剪辑操作",
        },
        CommandHelpItem {
            name: ":rd",
            alias: ":ripple_delete",
            args: "",
            description: "波纹删除当前切片并将其后切片自动向左吸附平移 (Shift+X)",
            category: "剪辑操作",
        },
        CommandHelpItem {
            name: ":close_gaps",
            alias: ":closegaps",
            args: "",
            description: "自动消除当前轨道上所有相邻切片间的空白间隙",
            category: "剪辑操作",
        },
        CommandHelpItem {
            name: ":goto",
            alias: "",
            args: "<时间/增量>",
            description: "跳转至指定绝对或相对时间 (如 10, 10.12, 1:10, +1:15.3, -30)",
            category: "时间导航",
        },
        CommandHelpItem {
            name: ":name",
            alias: "",
            args: "<新名称>",
            description: "修改当前选中轨道的名称",
            category: "轨道管理",
        },
        CommandHelpItem {
            name: ":lock",
            alias: "",
            args: "",
            description: "锁定当前切片（防止后续操作意外移位）",
            category: "切片属性",
        },
        CommandHelpItem {
            name: ":unlock",
            alias: "",
            args: "",
            description: "解除当前切片的锁定状态",
            category: "切片属性",
        },
        CommandHelpItem {
            name: ":speed",
            alias: "",
            args: "<倍率>",
            description: "设置当前切片的播放速度乘数 (如 :speed 2.0)",
            category: "切片特效",
        },
        CommandHelpItem {
            name: ":rotate",
            alias: "",
            args: "<角度>",
            description: "旋转当前切片画面 (如 :rotate 90, :rotate -45)",
            category: "切片特效",
        },
        CommandHelpItem {
            name: ":flip",
            alias: "",
            args: "[h|v|both]",
            description: "水平或垂直翻转当前切片画面 (如 :flip h, :flip v)",
            category: "切片特效",
        },
        CommandHelpItem {
            name: ":scale_clip",
            alias: ":scaleclip",
            args: "<倍率>",
            description: "缩放当前切片画面视口大小 (如 :scale_clip 1.5)",
            category: "切片特效",
        },
        CommandHelpItem {
            name: ":reset_transform",
            alias: ":resettransform",
            args: "",
            description: "重置当前切片的所有几何变换至初始状态",
            category: "切片特效",
        },
        CommandHelpItem {
            name: ":brightness",
            alias: "",
            args: "<-1.0~1.0>",
            description: "调节当前切片的画面亮度 (如 :brightness 0.15)",
            category: "色彩分级",
        },
        CommandHelpItem {
            name: ":contrast",
            alias: "",
            args: "<0.0~3.0>",
            description: "调节当前切片的画面对比度 (如 :contrast 1.2)",
            category: "色彩分级",
        },
        CommandHelpItem {
            name: ":saturation",
            alias: ":sat",
            args: "<0.0~3.0>",
            description: "调节当前切片的色彩饱和度 (如 :saturation 1.4, 0.0 为黑白)",
            category: "色彩分级",
        },
        CommandHelpItem {
            name: ":temp",
            alias: ":temperature",
            args: "<-1.0~1.0>",
            description: "调节当前切片的色温偏向 (正值偏暖橙，负值偏冷蓝)",
            category: "色彩分级",
        },
        CommandHelpItem {
            name: ":lut",
            alias: "",
            args: "<预设名称>",
            description: "应用电影级 3D LUT 滤镜预设 (如 :lut teal_orange, :lut cinematic)",
            category: "色彩分级",
        },
        CommandHelpItem {
            name: ":reset_color",
            alias: ":resetcolor",
            args: "",
            description: "重置当前切片的所有色彩分级与 LUT 滤镜参数",
            category: "色彩分级",
        },
        CommandHelpItem {
            name: ":transition",
            alias: ":trans",
            args: "<类型> [时长秒数]",
            description: "在当前切片尾部接缝处添加视频转场特效 (如 :transition dissolve 1.0)",
            category: "转场特效",
        },
        CommandHelpItem {
            name: ":text",
            alias: ":subtitle",
            args: "<文本内容>",
            description: "为当前切片添加或设置屏幕字幕与多行富文本 (如 :text 欢迎观看)",
            category: "字幕文本",
        },
        CommandHelpItem {
            name: ":fontsize",
            alias: "",
            args: "<字号像素>",
            description: "设置当前切片字幕的字体大小 (如 :fontsize 32)",
            category: "字幕文本",
        },
        CommandHelpItem {
            name: ":textcolor",
            alias: ":color_text",
            args: "<颜色/十六进制>",
            description: "设置当前切片字幕的字体颜色 (如 :textcolor yellow, :textcolor #ffff00)",
            category: "字幕文本",
        },
        CommandHelpItem {
            name: ":bgbox",
            alias: ":textbg",
            args: "<on|off>",
            description: "开启或关闭当前切片字幕的半透明气泡底框 (如 :bgbox on)",
            category: "字幕文本",
        },
        CommandHelpItem {
            name: ":clear_text",
            alias: ":cleartext",
            args: "",
            description: "清除当前切片上的所有字幕与富文本覆盖",
            category: "字幕文本",
        },
        CommandHelpItem {
            name: ":snap",
            alias: ":snapping",
            args: "[on|off]",
            description: "开启、关闭或切换磁性时间线吸附与智能标尺辅助线",
            category: "时间导航",
        },
        CommandHelpItem {
            name: ":easing",
            alias: ":curve",
            args: "[preset | bezier x1 y1 x2 y2]",
            description: "打开贝塞尔曲线可视化编辑器或设置缓动曲线 (linear, ease_in, ease_out, ease_in_out, bounce, elastic)",
            category: "特效控制",
        },
        CommandHelpItem {
            name: ":pip",
            alias: ":picture_in_picture",
            args: "<corner_br|tr|bl|tl | split_left|right|top|bottom | grid_tl|tr|bl|br | center | reset>",
            description: "一键应用画中画浮窗或分屏排布模板预设",
            category: "特效控制",
        },
        CommandHelpItem {
            name: ":fadein",
            alias: ":fade_in",
            args: "<秒数/时间>",
            description: "设置当前切片的音频淡入包络时长 (如 :fadein 0.5)",
            category: "音频控制",
        },
        CommandHelpItem {
            name: ":fadeout",
            alias: ":fade_out",
            args: "<秒数/时间>",
            description: "设置当前切片的音频淡出包络时长 (如 :fadeout 1.0)",
            category: "音频控制",
        },
        CommandHelpItem {
            name: ":vol",
            alias: ":volume",
            args: "<0.0~2.0>",
            description: "设置主音频增益 (如 :vol 1.2 设置为 120% 音量)",
            category: "音频控制",
        },
        CommandHelpItem {
            name: ":mute",
            alias: "",
            args: "",
            description: "静音主音频输出",
            category: "音频控制",
        },
        CommandHelpItem {
            name: ":unmute",
            alias: "",
            args: "",
            description: "恢复主音频输出",
            category: "音频控制",
        },
        CommandHelpItem {
            name: ":editor",
            alias: ":e",
            args: "[切片名]",
            description: "进入切片专属 Lua 特效脚本 IDE 编辑器",
            category: "特效开发",
        },
        CommandHelpItem {
            name: ":merge",
            alias: "",
            args: "",
            description: "(Visual多选) 将选中的多个连续切片合并为一个完整切片",
            category: "剪辑操作",
        },
        CommandHelpItem {
            name: ":mergecut",
            alias: "",
            args: "",
            description: "(Visual选区) 在选区左右两端切断并将中间部分合并",
            category: "剪辑操作",
        },
        CommandHelpItem {
            name: ":biset",
            alias: "",
            args: "",
            description: "开启二分法快速区间决策模式 ([-] / [=])",
            category: "剪辑操作",
        },
        CommandHelpItem {
            name: ":Marks",
            alias: ":marks",
            args: "",
            description: "弹出锚点总览列表，可查看/跳转/编辑所有锚点描述",
            category: "锚点系统",
        },
        CommandHelpItem {
            name: ":history",
            alias: "",
            args: "",
            description: "查看历史执行过的命令列表，按回车可快速填入复用",
            category: "系统命令",
        },
        CommandHelpItem {
            name: ":message",
            alias: "",
            args: "",
            description: "弹出系统历史日志与底层调试输出窗口",
            category: "系统命令",
        },
        CommandHelpItem {
            name: ":export",
            alias: "",
            args: "[path.mp4] [h264|hevc|prores]",
            description: "打开视频渲染导出面板，支持多预设硬件加速与分片极速拼接",
            category: "工程渲染",
        },
        CommandHelpItem {
            name: ":export_lua",
            alias: ":save_lua",
            args: "[path.lua]",
            description: "将当前整个项目状态完整导出为规范的 Lua 工程脚本",
            category: "工程管理",
        },
        CommandHelpItem {
            name: ":w",
            alias: ":write",
            args: "[path]",
            description: "安全原子保存工程文件 (支持 .vcut 或 .lua 格式)",
            category: "工程管理",
        },
        CommandHelpItem {
            name: ":q",
            alias: ":quit",
            args: "",
            description: "退出项目，返回项目导航选择网格页面",
            category: "页面导航",
        },
        CommandHelpItem {
            name: ":help",
            alias: ":h",
            args: "",
            description: "弹出支持的命令列表参考窗口，支持 / 键搜索筛选",
            category: "帮助系统",
        },
    ]
}

pub struct MainInterfaceUiState {
    pub playhead_us: i64, // 当前播放指针微秒 (us)
    pub is_playing: bool,
    pub zoom_level: f32, // 像素/秒 (默认 100.0)
    pub selected_track_idx: usize,
    pub selected_clip_id: Option<u64>,
    pub show_help_modal: bool,
    pub show_command_help_modal: bool, // :help / :h 弹出的独立命令帮助窗口
    pub command_help_search: String,
    pub command_help_selected_idx: usize,
    pub command_help_search_active: bool,
    pub show_marks_manager_modal: bool, // :Marks / :marks 弹出的交互式锚点管理弹窗
    pub marks_manager_search: String,
    pub marks_manager_selected_idx: usize,
    pub marks_manager_search_active: bool,
    pub show_history_modal: bool, // :history 弹出的交互式历史命令弹窗
    pub history_search: String,
    pub history_selected_idx: usize,
    pub history_search_active: bool,
    pub show_export_modal: bool, // :export 弹出的视频渲染导出弹窗
    pub export_state: crate::rendering::smart_export::ExportTaskState, // 导出任务实时进度状态
    pub macro_recorder: crate::keybinding::MacroRecorder, // 键盘宏录制与回放器
    pub macro_pending_prefix: Option<char>, // 正在等待输入的宏寄存器前缀 ('q' 或 '@')
    pub master_volume: f32, // 主音频增益 (0.0 ~ 2.0)
    pub is_muted: bool,     // 是否静音
    pub vu_meter: crate::media::audio_pipeline::VuMeterState, // 立体声 VU 电平表物理衰减计算状态
    pub command_input: String,
    pub is_command_mode: bool,
    pub media_search: String,
    pub visual_start_us: Option<i64>, // Visual 选区起点
    pub visual_end_us: Option<i64>,   // Visual 选区终点
    pub visual_line_selected_clips: Vec<crate::timeline::ClipId>, // VisualLine 模式选中的切片 ID 列表
    pub biset_session: Option<crate::timeline::BisetSession>, // 二分法决策会话
    pub anchor_mark_session: Option<AnchorMarkSession>, // 锚点标记交互会话 (m/M)
    pub anchor_jump_session: Option<AnchorJumpSession>, // 锚点跳转交互会话 (')
    pub current_mode: Mode,
    pub history_output: Vec<String>,
    pub show_message_window: bool,
    pub command_history_list: Vec<String>,
    pub status_message: Option<String>,
    pub snapping_enabled: bool, // 磁性吸附开关 (默认开启)
    pub active_snap_guide: Option<crate::timeline::SnapResult>, // 当前吸附对齐标尺线与说明
    pub show_easing_modal: bool, // :easing / :curve 弹出的贝塞尔缓动曲线可视化编辑器
}

impl Default for MainInterfaceUiState {
    fn default() -> Self {
        Self {
            playhead_us: 4_200_000, // 4.2 秒
            is_playing: false,
            zoom_level: 100.0,
            selected_track_idx: 0,
            selected_clip_id: Some(1),
            show_help_modal: false,
            show_command_help_modal: false,
            command_help_search: String::new(),
            command_help_selected_idx: 0,
            command_help_search_active: false,
            show_marks_manager_modal: false,
            marks_manager_search: String::new(),
            marks_manager_selected_idx: 0,
            marks_manager_search_active: false,
            show_history_modal: false,
            history_search: String::new(),
            history_selected_idx: 0,
            history_search_active: false,
            show_export_modal: false,
            export_state: crate::rendering::smart_export::ExportTaskState::default(),
            macro_recorder: crate::keybinding::MacroRecorder::default(),
            macro_pending_prefix: None,
            master_volume: 1.0,
            is_muted: false,
            vu_meter: crate::media::audio_pipeline::VuMeterState::default(),
            command_input: String::new(),
            is_command_mode: false,
            media_search: String::new(),
            visual_start_us: None,
            visual_end_us: None,
            visual_line_selected_clips: Vec::new(),
            biset_session: None,
            anchor_mark_session: None,
            anchor_jump_session: None,
            current_mode: Mode::Normal,
            history_output: Vec::new(),
            show_message_window: false,
            command_history_list: Vec::new(),
            status_message: None,
            snapping_enabled: true,
            active_snap_guide: None,
            show_easing_modal: false,
        }
    }
}

pub fn collect_all_anchors(project: &ProjectState) -> Vec<AnchorItemView> {
    let mut list = Vec::new();
    for a in &project.timeline.global_anchors.anchors {
        list.push(AnchorItemView {
            name: a.name.clone(),
            description: if a.description.is_empty() {
                "全局轨道锚点".to_string()
            } else {
                a.description.clone()
            },
            position_us: a.position.0,
            scope_label: "[全局]".to_string(),
            is_global: true,
        });
    }
    for track in &project.timeline.tracks {
        for clip in &track.clips {
            for (name, a) in &clip.anchors {
                list.push(AnchorItemView {
                    name: name.clone(),
                    description: if a.description.is_empty() {
                        format!("切片 '{}' 局部锚点", clip.name)
                    } else {
                        a.description.clone()
                    },
                    position_us: clip.timeline_start.0 + a.position.0,
                    scope_label: format!("[切片: {}]", clip.name),
                    is_global: false,
                });
            }
        }
    }
    list
}

pub fn show(ui: &mut Ui, project: &mut ProjectState, state: &mut MainInterfaceUiState) {
    let full_rect = ui.max_rect();
    ui.painter().rect_filled(full_rect, 0.0, Theme::BG_APP);

    // 1. 顶部全局信息条
    let top_height = 42.0;
    let top_rect = Rect::from_min_size(full_rect.min, vec2(full_rect.width(), top_height));
    ui.painter().rect_filled(top_rect, 0.0, Theme::BG_PANEL);
    ui.painter().line_segment(
        [pos2(top_rect.min.x, top_rect.max.y), top_rect.max],
        Stroke::new(1.0, Theme::BORDER_SUBTLE),
    );

    ui.scope_builder(UiBuilder::new().max_rect(top_rect), |ui| {
        ui.horizontal_centered(|ui| {
            ui.add_space(14.0);
            ui.label(RichText::new("🎬").size(16.0));
            ui.label(
                RichText::new(&project.name)
                    .size(15.0)
                    .strong()
                    .color(Theme::TEXT_PRIMARY),
            );

            ui.add_space(10.0);
            ui.label(
                RichText::new("1080p 60fps")
                    .size(11.0)
                    .color(Theme::TEXT_MUTED),
            );

            ui.add_space(30.0);
            // 播放时间码显示
            let cur_time_str = format_timecode(state.playhead_us);
            let dur_time_str = format_timecode(15_300_000); // 假定总时长
            ui.label(
                RichText::new(format!("{} / {}", cur_time_str, dur_time_str))
                    .monospace()
                    .size(14.0)
                    .color(Theme::ACCENT_CYAN)
                    .strong(),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(14.0);

                let help_btn = ui.button(RichText::new(" [?] 快捷键帮助 ").size(12.0).color(
                    if state.show_help_modal {
                        Theme::ACCENT_CYAN
                    } else {
                        Theme::TEXT_SECONDARY
                    },
                ));
                if help_btn.clicked() {
                    state.show_help_modal = !state.show_help_modal;
                }

                ui.add_space(8.0);
                ui.label(
                    RichText::new("Shift+N: 导航 | Shift+E: 脚本编辑器 | Shift+F: 文件库")
                        .size(11.0)
                        .color(Theme::TEXT_MUTED),
                );
            });
        });
    });

    // 2. 中间区域：左边素材栏 (Media Pool) + 右边视频预览 (Viewport)
    let bottom_height = 260.0; // 时间线高度
    let status_height = 36.0; // 状态栏高度
    let middle_height = full_rect.height() - top_height - bottom_height - status_height;

    let media_pool_width = 300.0;
    let middle_rect = Rect::from_min_size(
        pos2(full_rect.min.x, full_rect.min.y + top_height),
        vec2(full_rect.width(), middle_height),
    );

    // 2.1 左侧素材库
    let media_rect = Rect::from_min_size(middle_rect.min, vec2(media_pool_width, middle_height));
    ui.painter()
        .rect_filled(media_rect, 0.0, Theme::BG_PANEL_ALT);
    ui.painter().line_segment(
        [pos2(media_rect.max.x, media_rect.min.y), media_rect.max],
        Stroke::new(1.0, Theme::BORDER_SUBTLE),
    );

    ui.scope_builder(UiBuilder::new().max_rect(media_rect), |ui| {
        ui.vertical(|ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                ui.label(
                    RichText::new("📁 素材库 (Media Pool)")
                        .strong()
                        .size(14.0)
                        .color(Theme::TEXT_PRIMARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(12.0);
                    let _import_btn = ui.button(
                        RichText::new(" + 导入 (i) ")
                            .size(11.0)
                            .color(Theme::ACCENT_CYAN),
                    );
                });
            });

            ui.add_space(8.0);
            // 搜索过滤框
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                ui.add(
                    egui::TextEdit::singleline(&mut state.media_search)
                        .hint_text("🔍 搜索素材...")
                        .desired_width(media_pool_width - 24.0),
                );
            });

            ui.add_space(8.0);
            ui.separator();

            // 素材列表
            let sample_media = [
                (
                    "Clip_001.mp4",
                    "MP4",
                    "00:04:12",
                    "215 MB",
                    Color32::from_rgb(40, 80, 140),
                ),
                (
                    "Drone_Landscape.mov",
                    "MOV",
                    "00:02:45",
                    "540 MB",
                    Color32::from_rgb(35, 100, 90),
                ),
                (
                    "Interview_Audio.wav",
                    "WAV",
                    "00:12:00",
                    "125 MB",
                    Color32::from_rgb(110, 85, 30),
                ),
                (
                    "B-Roll_City.mp4",
                    "MP4",
                    "00:01:30",
                    "95 MB",
                    Color32::from_rgb(80, 40, 130),
                ),
                (
                    "Logo_Overlay.png",
                    "PNG",
                    "--",
                    "4 MB",
                    Color32::from_rgb(120, 60, 40),
                ),
                (
                    "Theme_Music.mp3",
                    "MP3",
                    "03:15",
                    "8 MB",
                    Color32::from_rgb(90, 45, 80),
                ),
            ];

            egui::ScrollArea::vertical()
                .id_salt("media_pool_scroll")
                .show(ui, |ui| {
                    for (name, format, duration, size, color) in sample_media.iter() {
                        let item_rect = ui.allocate_space(vec2(media_pool_width - 20.0, 48.0)).1;
                        let painter = ui.painter_at(item_rect);
                        let is_hovered = ui.rect_contains_pointer(item_rect);

                        let bg = if is_hovered {
                            Theme::BG_CARD_HOVER
                        } else {
                            Theme::BG_CARD
                        };
                        painter.rect_filled(item_rect, CornerRadius::same(6), bg);
                        painter.rect_stroke(
                            item_rect,
                            CornerRadius::same(6),
                            Stroke::new(
                                1.0,
                                if is_hovered {
                                    Theme::ACCENT_BLUE
                                } else {
                                    Theme::BORDER_SUBTLE
                                },
                            ),
                            egui::StrokeKind::Inside,
                        );

                        // 缩略图块
                        let thumb_rect =
                            Rect::from_min_size(item_rect.min + vec2(6.0, 6.0), vec2(48.0, 36.0));
                        painter.rect_filled(thumb_rect, CornerRadius::same(4), *color);
                        let icon_text = match *format {
                            "WAV" | "MP3" => "🎵",
                            "PNG" | "JPG" => "🖼",
                            _ => "🎬",
                        };
                        painter.text(
                            thumb_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            icon_text,
                            egui::FontId::proportional(14.0),
                            Color32::WHITE,
                        );

                        // 名称与元数据
                        painter.text(
                            item_rect.min + vec2(60.0, 8.0),
                            egui::Align2::LEFT_TOP,
                            *name,
                            egui::FontId::proportional(13.0),
                            Theme::TEXT_PRIMARY,
                        );

                        painter.text(
                            item_rect.min + vec2(60.0, 26.0),
                            egui::Align2::LEFT_TOP,
                            format!("{}  |  {}  |  {}", format, duration, size),
                            egui::FontId::proportional(10.0),
                            Theme::TEXT_MUTED,
                        );
                    }
                });
        });
    });

    // 2.2 右侧视频监视器 (Viewport & Controls)
    let viewport_rect = Rect::from_min_size(
        pos2(middle_rect.min.x + media_pool_width, middle_rect.min.y),
        vec2(middle_rect.width() - media_pool_width, middle_height),
    );

    ui.scope_builder(UiBuilder::new().max_rect(viewport_rect), |ui| {
        ui.vertical_centered(|ui| {
            let avail_w = viewport_rect.width() - 40.0;
            let avail_h = viewport_rect.height() - 70.0;

            // 保持 16:9 比例
            let mut monitor_w = avail_w;
            let mut monitor_h = monitor_w * (9.0 / 16.0);
            if monitor_h > avail_h {
                monitor_h = avail_h;
                monitor_w = monitor_h * (16.0 / 9.0);
            }

            ui.add_space(10.0);
            let monitor_rect = ui.allocate_space(vec2(monitor_w, monitor_h)).1;
            let painter = ui.painter_at(monitor_rect);

            // 监视器黑色画框与阴影
            painter.rect_filled(
                monitor_rect,
                CornerRadius::same(8),
                Color32::from_rgb(10, 10, 12),
            );
            painter.rect_stroke(
                monitor_rect,
                CornerRadius::same(8),
                Stroke::new(1.5, Theme::BORDER_MEDIUM),
                egui::StrokeKind::Inside,
            );

            // 动态多轨道合成器图层查询 (VideoCompositor Layer Query)
            let video_inner = monitor_rect.shrink(2.0);
            let playhead = FrameTime(state.playhead_us);
            let mut active_layers = Vec::new();
            let mut active_audio_count = 0;

            for (t_idx, track) in project.timeline.tracks.iter().enumerate() {
                if let Some(clip) = track.clips.iter().find(|c| playhead >= c.timeline_start && playhead <= c.timeline_end()) {
                    if track.name.starts_with('A') || t_idx == 1 {
                        active_audio_count += 1;
                    } else {
                        active_layers.push(RenderLayer {
                            track_id: track.id,
                            clip_id: clip.id,
                            z_index: t_idx as i32,
                            position: (0.0, 0.0),
                            scale: (1.0, 1.0),
                            rotation_deg: 0.0,
                            opacity: 1.0,
                            crop_rect: None,
                            visible: true,
                        });
                    }
                }
            }

            let compositor = VideoCompositor::new(1920, 1080);
            let sorted_layers = compositor.sort_layers_for_render(active_layers);

            let bg_color = if sorted_layers.is_empty() {
                Color32::from_rgb(15, 16, 20)
            } else {
                Color32::from_rgb(25, 45, 65)
            };
            painter.rect_filled(video_inner, CornerRadius::same(6), bg_color);

            // 画十字安全线 (Safe Area)
            let center = video_inner.center();
            painter.line_segment(
                [center - vec2(12.0, 0.0), center + vec2(12.0, 0.0)],
                Stroke::new(1.0, Color32::from_white_alpha(100)),
            );
            painter.line_segment(
                [center - vec2(0.0, 12.0), center + vec2(0.0, 12.0)],
                Stroke::new(1.0, Color32::from_white_alpha(100)),
            );

            // 图层与黑场状态渲染
            if sorted_layers.is_empty() {
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    "[ 黑场 / 无激活视频切片 ]",
                    egui::FontId::proportional(12.5),
                    Theme::TEXT_MUTED,
                );
            } else {
                for (idx, layer) in sorted_layers.iter().enumerate() {
                    let badge_pos = video_inner.min + vec2(14.0, 14.0 + (idx as f32 * 18.0));
                    painter.text(
                        badge_pos,
                        egui::Align2::LEFT_TOP,
                        format!("Layer {}: Track #{} Clip #{} [100% 1920x1080]", idx + 1, layer.track_id.0, layer.clip_id.0),
                        egui::FontId::monospace(10.5),
                        Theme::ACCENT_CYAN,
                    );
                }
            }

            // 音频立体声电平指示 (Audio Master VU)
            if active_audio_count > 0 {
                let vu_rect = Rect::from_min_size(video_inner.max - vec2(80.0, 36.0), vec2(66.0, 8.0));
                painter.rect_filled(vu_rect, CornerRadius::same(2), Color32::from_rgb(20, 20, 20));
                let meter_fill = Rect::from_min_size(vu_rect.min, vec2(vu_rect.width() * 0.75, vu_rect.height()));
                painter.rect_filled(meter_fill, CornerRadius::same(2), Theme::ACCENT_GREEN);
                painter.text(
                    vu_rect.min - vec2(0.0, 12.0),
                    egui::Align2::LEFT_TOP,
                    "AUDIO MASTER",
                    egui::FontId::monospace(9.0),
                    Theme::ACCENT_GREEN,
                );
            }

            // 视口几何变换控制器 (Viewport Transform Gizmo & Handles)
            let selected_track_idx = state.selected_track_idx;
            if let Some(sel_track) = project.timeline.tracks.get(selected_track_idx) {
                if let Some(sel_clip) = sel_track.clips.iter().find(|c| playhead >= c.timeline_start && playhead <= c.timeline_end()) {
                    let gizmo_rect = video_inner.shrink(18.0);
                    // 绘制变换半透明边框
                    painter.rect_stroke(
                        gizmo_rect,
                        CornerRadius::same(4),
                        Stroke::new(1.5, Color32::from_rgba_unmultiplied(80, 200, 255, 200)),
                        egui::StrokeKind::Outside,
                    );

                    // 绘制 8 个缩放锚点手柄 (Corners & Edge Centers)
                    let handle_size = vec2(6.0, 6.0);
                    let handle_pts = [
                        gizmo_rect.min, // 左上
                        pos2(gizmo_rect.center().x, gizmo_rect.min.y), // 上中
                        pos2(gizmo_rect.max.x, gizmo_rect.min.y), // 右上
                        pos2(gizmo_rect.min.x, gizmo_rect.center().y), // 左中
                        pos2(gizmo_rect.max.x, gizmo_rect.center().y), // 右中
                        pos2(gizmo_rect.min.x, gizmo_rect.max.y), // 左下
                        pos2(gizmo_rect.center().x, gizmo_rect.max.y), // 下中
                        gizmo_rect.max, // 右下
                    ];
                    for pt in handle_pts {
                        painter.rect_filled(
                            Rect::from_center_size(pt, handle_size),
                            CornerRadius::same(1),
                            Color32::WHITE,
                        );
                        painter.rect_stroke(
                            Rect::from_center_size(pt, handle_size),
                            CornerRadius::same(1),
                            Stroke::new(1.0, Color32::BLACK),
                            egui::StrokeKind::Outside,
                        );
                    }

                    // 顶部旋转控制连线与手柄 (Rotation Handle & Stem)
                    let top_mid = pos2(gizmo_rect.center().x, gizmo_rect.min.y);
                    let rot_handle = top_mid - vec2(0.0, 16.0);
                    painter.line_segment([top_mid, rot_handle], Stroke::new(1.2, Theme::ACCENT_CYAN));
                    painter.circle_filled(rot_handle, 4.0, Theme::ACCENT_CYAN);
                    painter.circle_stroke(rot_handle, 4.0, Stroke::new(1.0, Color32::WHITE));

                    // 变换参数角标 Badge
                    if sel_clip.transform_rotation_deg.abs() > 0.01 || sel_clip.transform_flip_h || sel_clip.transform_flip_v || (sel_clip.transform_scale[0] - 1.0).abs() > 0.01 {
                        let mut badges = Vec::new();
                        if sel_clip.transform_rotation_deg.abs() > 0.01 {
                            badges.push(format!("⟳ {:.0}°", sel_clip.transform_rotation_deg));
                        }
                        if sel_clip.transform_flip_h {
                            badges.push("⇄ FlipH".into());
                        }
                        if sel_clip.transform_flip_v {
                            badges.push("⇅ FlipV".into());
                        }
                        if (sel_clip.transform_scale[0] - 1.0).abs() > 0.01 {
                            badges.push(format!("🔍 {:.2}x", sel_clip.transform_scale[0]));
                        }
                        painter.text(
                            gizmo_rect.min + vec2(8.0, 8.0),
                            egui::Align2::LEFT_TOP,
                            badges.join(" | "),
                            egui::FontId::monospace(10.5),
                            Theme::ACCENT_YELLOW,
                        );
                    }

                    // 色彩分级与 LUT 滤镜角标 Badge
                    if sel_clip.color_grading.is_active() {
                        let mut color_badges = Vec::new();
                        if sel_clip.color_grading.lut_preset != crate::effects::LutPreset::None {
                            color_badges.push(format!("🎨 LUT: {}", sel_clip.color_grading.lut_preset.name()));
                        }
                        if sel_clip.color_grading.brightness.abs() > 0.001 {
                            color_badges.push(format!("☀ {:+.2}", sel_clip.color_grading.brightness));
                        }
                        if (sel_clip.color_grading.contrast - 1.0).abs() > 0.001 {
                            color_badges.push(format!("◐ {:.2}x", sel_clip.color_grading.contrast));
                        }
                        if (sel_clip.color_grading.saturation - 1.0).abs() > 0.001 {
                            color_badges.push(format!("💧 {:.2}x", sel_clip.color_grading.saturation));
                        }
                        if sel_clip.color_grading.temperature.abs() > 0.001 {
                            color_badges.push(format!("🌡 {:+.2}", sel_clip.color_grading.temperature));
                        }
                        painter.text(
                            gizmo_rect.min + vec2(8.0, 24.0),
                            egui::Align2::LEFT_TOP,
                            color_badges.join(" | "),
                            egui::FontId::monospace(10.0),
                            Theme::ACCENT_CYAN,
                        );
                    }

                    // 字幕与文本覆盖渲染 (Text & Subtitle Overlay)
                    if let Some(ref text_overlay) = sel_clip.text_overlay {
                        if !text_overlay.is_empty() {
                            let text_pos = match text_overlay.alignment {
                                crate::effects::TextAlignment::BottomCenter => pos2(gizmo_rect.center().x, gizmo_rect.max.y - 28.0),
                                crate::effects::TextAlignment::TopCenter => pos2(gizmo_rect.center().x, gizmo_rect.min.y + 28.0),
                                crate::effects::TextAlignment::Center => gizmo_rect.center(),
                                crate::effects::TextAlignment::BottomLeft => pos2(gizmo_rect.min.x + 24.0, gizmo_rect.max.y - 28.0),
                                crate::effects::TextAlignment::BottomRight => pos2(gizmo_rect.max.x - 24.0, gizmo_rect.max.y - 28.0),
                            };
                            let align = match text_overlay.alignment {
                                crate::effects::TextAlignment::BottomCenter | crate::effects::TextAlignment::TopCenter | crate::effects::TextAlignment::Center => egui::Align2::CENTER_CENTER,
                                crate::effects::TextAlignment::BottomLeft => egui::Align2::LEFT_BOTTOM,
                                crate::effects::TextAlignment::BottomRight => egui::Align2::RIGHT_BOTTOM,
                            };

                            let font_id = egui::FontId::proportional(text_overlay.font_size.clamp(12.0, 36.0));
                            let text_color = Color32::from_rgba_unmultiplied(
                                text_overlay.color_rgba[0],
                                text_overlay.color_rgba[1],
                                text_overlay.color_rgba[2],
                                text_overlay.color_rgba[3],
                            );

                            // 计算文本包围盒与气泡底框
                            let galley = painter.layout_no_wrap(text_overlay.content.clone(), font_id.clone(), text_color);
                            let text_rect = align.anchor_size(text_pos, galley.size());

                            if text_overlay.has_background {
                                let bg_rect = text_rect.expand2(vec2(10.0, 6.0));
                                let bg_color = Color32::from_rgba_unmultiplied(
                                    text_overlay.bg_rgba[0],
                                    text_overlay.bg_rgba[1],
                                    text_overlay.bg_rgba[2],
                                    text_overlay.bg_rgba[3],
                                );
                                painter.rect_filled(bg_rect, CornerRadius::same(5), bg_color);
                                painter.rect_stroke(
                                    bg_rect,
                                    CornerRadius::same(5),
                                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 40)),
                                    egui::StrokeKind::Inside,
                                );
                            }

                            // 描边效果
                            if text_overlay.outline_width > 0.0 {
                                let outline_color = Color32::from_rgba_unmultiplied(
                                    text_overlay.outline_rgba[0],
                                    text_overlay.outline_rgba[1],
                                    text_overlay.outline_rgba[2],
                                    text_overlay.outline_rgba[3],
                                );
                                for (dx, dy) in &[(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
                                    painter.text(
                                        text_pos + vec2(*dx * text_overlay.outline_width, *dy * text_overlay.outline_width),
                                        align,
                                        &text_overlay.content,
                                        font_id.clone(),
                                        outline_color,
                                    );
                                }
                            }

                            // 主体文字渲染
                            painter.text(text_pos, align, &text_overlay.content, font_id, text_color);
                        }
                    }
                }
            }

            // 画面右下角渲染分辨率
            painter.text(
                video_inner.max - vec2(14.0, 14.0),
                egui::Align2::RIGHT_BOTTOM,
                "1920x1080 @ 60fps",
                egui::FontId::monospace(11.0),
                Theme::TEXT_MUTED,
            );

            // 播放器底栏控制区
            ui.add_space(8.0);
            ui.horizontal_centered(|ui| {
                let play_btn_text = if state.is_playing {
                    "⏸ 暂停 (Space)"
                } else {
                    "▶ 播放 (Space)"
                };
                let play_btn = ui.button(
                    RichText::new(play_btn_text)
                        .size(13.0)
                        .strong()
                        .color(Theme::ACCENT_CYAN),
                );
                if play_btn.clicked() {
                    state.is_playing = !state.is_playing;
                }

                ui.add_space(10.0);
                if ui.button(RichText::new("⏮ -1s (h)").size(12.0)).clicked() {
                    state.playhead_us = (state.playhead_us - 1_000_000).max(0);
                }
                if ui.button(RichText::new("◀ -1f (H)").size(12.0)).clicked() {
                    state.playhead_us = (state.playhead_us - 16_666).max(0);
                }
                if ui.button(RichText::new("1f ▶ (L)").size(12.0)).clicked() {
                    state.playhead_us += 16_666;
                }
                if ui.button(RichText::new("+1s ⏭ (l)").size(12.0)).clicked() {
                    state.playhead_us += 1_000_000;
                }

                ui.add_space(15.0);
                // 主音量与静音切换
                let mute_icon = if state.is_muted { "🔇" } else { "🔊" };
                if ui.button(RichText::new(mute_icon).size(13.0)).clicked() {
                    state.is_muted = !state.is_muted;
                }
                ui.label(
                    RichText::new(format!("{:.0}%", state.master_volume * 100.0))
                        .size(11.0)
                        .color(if state.is_muted { Theme::TEXT_MUTED } else { Theme::ACCENT_CYAN }),
                );

                ui.add_space(6.0);
                // 高精度立体声 VU 电平表 (Dynamic Stereo VU Meter with Peak Decay)
                let meter_w = 140.0;
                let meter_h = 16.0;
                let meter_rect = ui.allocate_space(vec2(meter_w, meter_h)).1;
                let painter = ui.painter_at(meter_rect);
                painter.rect_filled(meter_rect, CornerRadius::same(3), Theme::BG_INPUT);
                painter.rect_stroke(
                    meter_rect,
                    CornerRadius::same(3),
                    Stroke::new(1.0, Theme::BORDER_SUBTLE),
                    egui::StrokeKind::Inside,
                );

                let l_rms_w = (state.vu_meter.left_rms * (meter_w - 6.0)).clamp(0.0, meter_w - 6.0);
                let r_rms_w = (state.vu_meter.right_rms * (meter_w - 6.0)).clamp(0.0, meter_w - 6.0);
                let l_peak_x = meter_rect.min.x + 3.0 + (state.vu_meter.left_peak * (meter_w - 6.0)).clamp(0.0, meter_w - 6.0);
                let r_peak_x = meter_rect.min.x + 3.0 + (state.vu_meter.right_peak * (meter_w - 6.0)).clamp(0.0, meter_w - 6.0);

                let ch_l_color = if state.vu_meter.left_rms > 0.85 {
                    Color32::from_rgb(230, 70, 70)
                } else if state.vu_meter.left_rms > 0.65 {
                    Theme::ACCENT_ORANGE
                } else {
                    Theme::ACCENT_GREEN
                };

                let ch_r_color = if state.vu_meter.right_rms > 0.85 {
                    Color32::from_rgb(230, 70, 70)
                } else if state.vu_meter.right_rms > 0.65 {
                    Theme::ACCENT_ORANGE
                } else {
                    Theme::ACCENT_GREEN
                };

                // L 声道条
                let ch1_rect = Rect::from_min_size(meter_rect.min + vec2(3.0, 2.0), vec2(l_rms_w, 5.0));
                painter.rect_filled(ch1_rect, CornerRadius::same(1), ch_l_color);
                // L 峰值线
                if state.vu_meter.left_peak > 0.05 {
                    painter.line_segment(
                        [pos2(l_peak_x, meter_rect.min.y + 2.0), pos2(l_peak_x, meter_rect.min.y + 7.0)],
                        Stroke::new(1.5, Color32::WHITE),
                    );
                }

                // R 声道条
                let ch2_rect = Rect::from_min_size(meter_rect.min + vec2(3.0, 9.0), vec2(r_rms_w, 5.0));
                painter.rect_filled(ch2_rect, CornerRadius::same(1), ch_r_color);
                // R 峰值线
                if state.vu_meter.right_peak > 0.05 {
                    painter.line_segment(
                        [pos2(r_peak_x, meter_rect.min.y + 9.0), pos2(r_peak_x, meter_rect.min.y + 14.0)],
                        Stroke::new(1.5, Color32::WHITE),
                    );
                }

                if state.vu_meter.is_clipping {
                    // 红色削顶过载指示灯
                    let clip_rect = Rect::from_min_size(meter_rect.max - vec2(6.0, 14.0), vec2(4.0, 12.0));
                    painter.rect_filled(clip_rect, CornerRadius::same(1), Color32::from_rgb(255, 40, 40));
                }
            });
        });
    });

    // 3. 底部时间线面板 (Timeline & Multi-tracks)
    let timeline_rect = Rect::from_min_size(
        pos2(
            full_rect.min.x,
            full_rect.max.y - bottom_height - status_height,
        ),
        vec2(full_rect.width(), bottom_height),
    );

    ui.painter()
        .rect_filled(timeline_rect, 0.0, Theme::BG_PANEL);
    ui.painter().line_segment(
        [
            timeline_rect.min,
            pos2(timeline_rect.max.x, timeline_rect.min.y),
        ],
        Stroke::new(1.5, Theme::BORDER_MEDIUM),
    );

    ui.scope_builder(UiBuilder::new().max_rect(timeline_rect), |ui| {
        ui.vertical(|ui| {
            // 时间线控制头 (Zoom, Snapping, Tools)
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                ui.label(
                    RichText::new("⏱ 时间线 (Timeline)")
                        .size(13.0)
                        .strong()
                        .color(Theme::TEXT_PRIMARY),
                );
                ui.add_space(20.0);

                ui.label(RichText::new("缩放:").size(12.0).color(Theme::TEXT_MUTED));
                if ui.button(RichText::new(" - ").size(11.0)).clicked() {
                    state.zoom_level = (state.zoom_level * 0.8).max(20.0);
                }
                ui.add(egui::Slider::new(&mut state.zoom_level, 20.0..=400.0).show_value(false));
                if ui.button(RichText::new(" + ").size(11.0)).clicked() {
                    state.zoom_level = (state.zoom_level * 1.25).min(400.0);
                }

                ui.add_space(20.0);
                ui.label(
                    RichText::new(format!("当前位置: {}", format_timecode(state.playhead_us)))
                        .size(12.0)
                        .monospace()
                        .color(Theme::ACCENT_CYAN),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(14.0);
                    ui.label(
                        RichText::new("[s] 切割 | [v/V] 可视选择 | [m] 锚点 | [w/b] 跳波形")
                            .size(11.0)
                            .color(Theme::TEXT_MUTED),
                    );
                });
            });

            ui.separator();

            // 滚动时间线区域
            let track_header_width = 110.0;

            egui::ScrollArea::both()
                .id_salt("timeline_editor_scroll")
                .show(ui, |ui| {
                    let track_height = 56.0;
                    let track_count = project.timeline.tracks.len() + 1; // +1 为垃圾回收轨道
                    let virtual_width = 2500.0; // 虚拟时间线宽度

                    let (scroll_rect, timeline_resp) = ui.allocate_exact_size(
                        vec2(
                            virtual_width + track_header_width,
                            track_height * (track_count as f32) + 26.0,
                        ),
                        egui::Sense::click_and_drag(),
                    );

                    if timeline_resp.dragged() || timeline_resp.clicked() {
                        if let Some(mouse_pos) = timeline_resp.interact_pointer_pos() {
                            let ruler_start_x = scroll_rect.min.x + track_header_width;
                            if mouse_pos.x >= ruler_start_x {
                                let time_secs = (mouse_pos.x - ruler_start_x) / state.zoom_level;
                                let raw_time_us = (time_secs.max(0.0) * 1_000_000.0) as i64;
                                let raw_ft = crate::timeline::FrameTime(raw_time_us);

                                if state.snapping_enabled {
                                    let snap_engine = crate::timeline::SnapEngine::default();
                                    if let Some(snap) = snap_engine.find_snap_point(raw_ft, &project.timeline, &project.timeline.global_anchors, state.zoom_level) {
                                        state.playhead_us = snap.snapped_time.0;
                                        state.active_snap_guide = Some(snap);
                                    } else {
                                        state.playhead_us = raw_time_us;
                                        state.active_snap_guide = None;
                                    }
                                } else {
                                    state.playhead_us = raw_time_us;
                                    state.active_snap_guide = None;
                                }
                            }
                        }
                    } else if !timeline_resp.dragged() {
                        state.active_snap_guide = None;
                    }

                    let painter = ui.painter_at(scroll_rect);

                    // 3.1 绘制刻度尺 (Time Ruler)
                    let ruler_height = 24.0;
                    let ruler_rect = Rect::from_min_size(
                        pos2(scroll_rect.min.x + track_header_width, scroll_rect.min.y),
                        vec2(virtual_width, ruler_height),
                    );
                    painter.rect_filled(ruler_rect, 0.0, Theme::BG_INPUT);
                    painter.line_segment(
                        [pos2(ruler_rect.min.x, ruler_rect.max.y), ruler_rect.max],
                        Stroke::new(1.0, Theme::BORDER_SUBTLE),
                    );

                    // 刻度线与时间文字
                    let sec_step = if state.zoom_level > 150.0 { 1 } else { 2 };
                    for sec in (0..120).step_by(sec_step) {
                        let x = ruler_rect.min.x + (sec as f32 * state.zoom_level);
                        if x > ruler_rect.max.x {
                            break;
                        }

                        painter.line_segment(
                            [pos2(x, ruler_rect.max.y - 8.0), pos2(x, ruler_rect.max.y)],
                            Stroke::new(1.0, Theme::TEXT_MUTED),
                        );
                        painter.text(
                            pos2(x + 4.0, ruler_rect.min.y + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("00:{:02}", sec),
                            egui::FontId::monospace(10.0),
                            Theme::TEXT_MUTED,
                        );
                    }

                    // 绘制全局锚点标记 (Global Anchors on Ruler)
                    for anchor in &project.timeline.global_anchors.anchors {
                        let ax = ruler_rect.min.x + ((anchor.position.0 as f32 / 1_000_000.0) * state.zoom_level);
                        if ax >= ruler_rect.min.x && ax <= ruler_rect.max.x {
                            painter.line_segment(
                                [pos2(ax, ruler_rect.min.y), pos2(ax, ruler_rect.max.y)],
                                Stroke::new(1.5, Theme::ACCENT_CYAN),
                            );
                            let pin_rect = Rect::from_min_size(pos2(ax - 6.0, ruler_rect.min.y + 2.0), vec2(12.0, 12.0));
                            painter.rect_filled(pin_rect, CornerRadius::same(3), Theme::ACCENT_CYAN);
                            painter.text(
                                pin_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                &anchor.name,
                                egui::FontId::monospace(10.0),
                                Color32::BLACK,
                            );
                        }
                    }

                    // 3.2 绘制轨道与剪辑块
                    for (i, track) in project.timeline.tracks.iter().enumerate() {
                        let y_offset = scroll_rect.min.y + ruler_height + (i as f32) * track_height;
                        let is_track_selected = state.selected_track_idx == i;

                        // 轨道左侧 Header
                        let header_rect = Rect::from_min_size(
                            pos2(scroll_rect.min.x, y_offset),
                            vec2(track_header_width, track_height),
                        );
                        let header_bg = if is_track_selected {
                            Theme::BG_CARD_ACTIVE
                        } else {
                            Theme::BG_PANEL_ALT
                        };
                        painter.rect_filled(header_rect, CornerRadius::same(0), header_bg);
                        painter.line_segment(
                            [pos2(header_rect.max.x, header_rect.min.y), header_rect.max],
                            Stroke::new(1.0, Theme::BORDER_MEDIUM),
                        );

                        // 轨道表头鼠标交互与右键菜单
                        let header_resp = ui.interact(
                            header_rect,
                            ui.id().with(("track_hdr", track.id.0)),
                            egui::Sense::click(),
                        );
                        if header_resp.clicked() {
                            state.selected_track_idx = i;
                        }
                        header_resp.context_menu(|ui| {
                            ui.label(
                                RichText::new(format!("🎚 轨道: {}", track.name))
                                    .strong()
                                    .color(Theme::ACCENT_CYAN),
                            );
                            ui.separator();
                            if ui.button("✏ 重命名轨道 (:name)").clicked() {
                                state.command_input = format!(":name {}", track.name);
                                state.is_command_mode = true;
                                ui.close();
                            }
                            let pin_label = if track.is_pinned { "📌 取消固定置顶" } else { "📌 固定置顶轨道 (P)" };
                            if ui.button(pin_label).clicked() {
                                state.command_input = if track.is_pinned { ":unpin".into() } else { ":pin".into() };
                                state.is_command_mode = true;
                                ui.close();
                            }
                            if ui.button("⬆ 向上移动轨道 (Shift+K)").clicked() {
                                ui.close();
                            }
                            if ui.button("⬇ 向下移动轨道 (Shift+J)").clicked() {
                                ui.close();
                            }
                            ui.separator();
                            if ui.button("➕ 在下方插入新轨道 (O)").clicked() {
                                ui.close();
                            }
                            if ui.button("➕ 在上方插入新轨道 (Shift+O)").clicked() {
                                ui.close();
                            }
                            ui.separator();
                            if ui.button(RichText::new("❌ 删除此轨道").color(Color32::from_rgb(240, 80, 80))).clicked() {
                                ui.close();
                            }
                        });

                        // 轨道标题与图标
                        let track_icon = if track.name.starts_with('V') {
                            "📹"
                        } else {
                            "🔊"
                        };
                        painter.text(
                            pos2(header_rect.min.x + 8.0, header_rect.min.y + 10.0),
                            egui::Align2::LEFT_TOP,
                            format!("{} {}", track_icon, track.name),
                            egui::FontId::proportional(12.0),
                            if is_track_selected {
                                Theme::ACCENT_CYAN
                            } else {
                                Theme::TEXT_PRIMARY
                            },
                        );

                        // 轨道操作小图标 (Mute, Lock)
                        painter.text(
                            pos2(header_rect.min.x + 8.0, header_rect.min.y + 32.0),
                            egui::Align2::LEFT_TOP,
                            "👁 🔒 M S",
                            egui::FontId::proportional(10.0),
                            Theme::TEXT_MUTED,
                        );

                        // 轨道内容区域
                        let content_rect = Rect::from_min_size(
                            pos2(scroll_rect.min.x + track_header_width, y_offset),
                            vec2(virtual_width, track_height),
                        );
                        let track_bg = if i % 2 == 0 {
                            Color32::from_rgb(22, 24, 29)
                        } else {
                            Color32::from_rgb(26, 28, 35)
                        };
                        painter.rect_filled(content_rect, 0.0, track_bg);
                        painter.line_segment(
                            [
                                pos2(content_rect.min.x, content_rect.max.y),
                                content_rect.max,
                            ],
                            Stroke::new(1.0, Theme::BORDER_SUBTLE),
                        );

                        // 绘制剪辑块并附加鼠标右键上下文菜单
                        for clip in &track.clips {
                            let start_x = content_rect.min.x
                                + ((clip.timeline_start.0 as f32 / 1_000_000.0) * state.zoom_level);
                            let width = ((clip.duration().0 as f32 / 1_000_000.0) * state.zoom_level)
                                .max(12.0);
                            let clip_card_rect = Rect::from_min_size(
                                pos2(start_x, content_rect.min.y + 4.0),
                                vec2(width, content_rect.height() - 8.0),
                            );

                            draw_clip_card(&painter, content_rect, clip, state, i);

                            let clip_resp = ui.interact(
                                clip_card_rect,
                                ui.id().with(("clip_ctx", track.id.0, clip.id.0)),
                                egui::Sense::click(),
                            );
                            if clip_resp.clicked() {
                                state.selected_track_idx = i;
                                state.playhead_us = clip.timeline_start.0;
                            }
                            clip_resp.context_menu(|ui| {
                                ui.label(
                                    RichText::new(format!("🎬 切片: {}", clip.name))
                                        .strong()
                                        .color(Theme::ACCENT_CYAN),
                                );
                                ui.separator();
                                if ui.button("✂ 在当前播放头分割 (s)").clicked() {
                                    state.command_input = ":split".into();
                                    state.is_command_mode = true;
                                    ui.close();
                                }
                                if ui.button("🗑 删除至垃圾箱 (d)").clicked() {
                                    ui.close();
                                }
                                let lock_label = if clip.locked {
                                    "🔓 解除锁定"
                                } else {
                                    "🔒 锁定切片"
                                };
                                if ui.button(lock_label).clicked() {
                                    state.command_input = if clip.locked {
                                        ":unlock".into()
                                    } else {
                                        ":lock".into()
                                    };
                                    state.is_command_mode = true;
                                    ui.close();
                                }
                                ui.menu_button("⚡ 播放速度倍率", |ui| {
                                    if ui.button("0.5x (慢速)").clicked() {
                                        state.command_input = ":speed 0.5".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("1.0x (原速)").clicked() {
                                        state.command_input = ":speed 1.0".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("1.5x (中速)").clicked() {
                                        state.command_input = ":speed 1.5".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("2.0x (倍速)").clicked() {
                                        state.command_input = ":speed 2.0".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                });
                                ui.menu_button("🔊 音频淡入淡出", |ui| {
                                    if ui.button("淡入 0.5 秒 (:fadein 0.5)").clicked() {
                                        state.command_input = ":fadein 0.5".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("淡入 1.0 秒 (:fadein 1.0)").clicked() {
                                        state.command_input = ":fadein 1.0".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    ui.separator();
                                    if ui.button("淡出 0.5 秒 (:fadeout 0.5)").clicked() {
                                        state.command_input = ":fadeout 0.5".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("淡出 1.0 秒 (:fadeout 1.0)").clicked() {
                                        state.command_input = ":fadeout 1.0".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    ui.separator();
                                    if ui.button("清除淡入淡出").clicked() {
                                        state.command_input = ":fadein 0".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                });
                                ui.menu_button("📐 画面几何变换", |ui| {
                                    if ui.button("顺时针旋转 90° (:rotate 90)").clicked() {
                                        state.command_input = ":rotate 90".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("逆时针旋转 90° (:rotate -90)").clicked() {
                                        state.command_input = ":rotate -90".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    ui.separator();
                                    if ui.button("水平翻转 (:flip h)").clicked() {
                                        state.command_input = ":flip h".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("垂直翻转 (:flip v)").clicked() {
                                        state.command_input = ":flip v".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    ui.separator();
                                    if ui.button("画面缩放 1.5x (:scale_clip 1.5)").clicked() {
                                        state.command_input = ":scale_clip 1.5".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("重置几何变换 (:reset_transform)").clicked() {
                                        state.command_input = ":reset_transform".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                });
                                ui.menu_button("🎨 颜色分级与滤镜", |ui| {
                                    if ui.button("鲜艳增强 (:lut vibrant)").clicked() {
                                        state.command_input = ":lut vibrant".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("暖调电影 (:lut cinematic)").clicked() {
                                        state.command_input = ":lut cinematic".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("青橙电影 (:lut teal_orange)").clicked() {
                                        state.command_input = ":lut teal_orange".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("经典黑白 (:lut bw)").clicked() {
                                        state.command_input = ":lut bw".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("复古胶片 (:lut vintage)").clicked() {
                                        state.command_input = ":lut vintage".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("冷色科幻 (:lut cool)").clicked() {
                                        state.command_input = ":lut cool".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    ui.separator();
                                    if ui.button("重置色彩分级 (:reset_color)").clicked() {
                                        state.command_input = ":reset_color".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                });
                                ui.menu_button("✨ 视频转场特效", |ui| {
                                    if ui.button("交叉溶解 1.0s (:transition dissolve 1.0)").clicked() {
                                        state.command_input = ":transition dissolve 1.0".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("左划像 0.8s (:transition wipe_left 0.8)").clicked() {
                                        state.command_input = ":transition wipe_left 0.8".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("右划像 0.8s (:transition wipe_right 0.8)").clicked() {
                                        state.command_input = ":transition wipe_right 0.8".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("黑场闪烁 0.5s (:transition dip_black 0.5)").clicked() {
                                        state.command_input = ":transition dip_black 0.5".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("白场闪烁 0.5s (:transition dip_white 0.5)").clicked() {
                                        state.command_input = ":transition dip_white 0.5".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    ui.separator();
                                    if ui.button("清除转场 (:transition none)").clicked() {
                                        state.command_input = ":transition none".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                });
                                ui.menu_button("💬 文本与字幕", |ui| {
                                    if ui.button("添加/编辑文本 (:text ...)").clicked() {
                                        state.command_input = ":text 欢迎使用 VideoCut".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    ui.separator();
                                    if ui.button("大字号 32px (:fontsize 32)").clicked() {
                                        state.command_input = ":fontsize 32".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("中字号 24px (:fontsize 24)").clicked() {
                                        state.command_input = ":fontsize 24".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("小字号 18px (:fontsize 18)").clicked() {
                                        state.command_input = ":fontsize 18".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    ui.separator();
                                    if ui.button("开启气泡底框 (:bgbox on)").clicked() {
                                        state.command_input = ":bgbox on".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("关闭气泡底框 (:bgbox off)").clicked() {
                                        state.command_input = ":bgbox off".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    ui.separator();
                                    if ui.button("清除文本 (:clear_text)").clicked() {
                                        state.command_input = ":clear_text".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                });
                                ui.menu_button("📈 缓动曲线 (Easing Curve)", |ui| {
                                    if ui.button("打开曲线编辑器 (:easing)").clicked() {
                                        state.show_easing_modal = true;
                                        ui.close();
                                    }
                                    ui.separator();
                                    if ui.button("平滑缓入缓出 (:easing ease_in_out)").clicked() {
                                        state.command_input = ":easing ease_in_out".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("平滑加速 (:easing ease_in)").clicked() {
                                        state.command_input = ":easing ease_in".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("平滑减速 (:easing ease_out)").clicked() {
                                        state.command_input = ":easing ease_out".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("弹力弹跳 (:easing bounce)").clicked() {
                                        state.command_input = ":easing bounce".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("弹性阻尼 (:easing elastic)").clicked() {
                                        state.command_input = ":easing elastic".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("匀速直线 (:easing linear)").clicked() {
                                        state.command_input = ":easing linear".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                });
                                ui.menu_button("🖼 画中画与分屏布局 (PIP & Split)", |ui| {
                                    if ui.button("右下角画中画 (:pip corner_br)").clicked() {
                                        state.command_input = ":pip corner_br".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("右上角画中画 (:pip corner_tr)").clicked() {
                                        state.command_input = ":pip corner_tr".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("左下角画中画 (:pip corner_bl)").clicked() {
                                        state.command_input = ":pip corner_bl".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("居中悬浮浮窗 (:pip center)").clicked() {
                                        state.command_input = ":pip center".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    ui.separator();
                                    if ui.button("左半屏分屏 (:pip split_left)").clicked() {
                                        state.command_input = ":pip split_left".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("右半屏分屏 (:pip split_right)").clicked() {
                                        state.command_input = ":pip split_right".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("上半屏分屏 (:pip split_top)").clicked() {
                                        state.command_input = ":pip split_top".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("下半屏分屏 (:pip split_bottom)").clicked() {
                                        state.command_input = ":pip split_bottom".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    ui.separator();
                                    if ui.button("四宫格左上 (:pip grid_tl)").clicked() {
                                        state.command_input = ":pip grid_tl".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("四宫格右上 (:pip grid_tr)").clicked() {
                                        state.command_input = ":pip grid_tr".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("四宫格左下 (:pip grid_bl)").clicked() {
                                        state.command_input = ":pip grid_bl".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    if ui.button("四宫格右下 (:pip grid_br)").clicked() {
                                        state.command_input = ":pip grid_br".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                    ui.separator();
                                    if ui.button("重置为全屏充满 (:pip reset)").clicked() {
                                        state.command_input = ":pip reset".into();
                                        state.is_command_mode = true;
                                        ui.close();
                                    }
                                });
                                if ui.button("🏷 添加局部锚点 (m)").clicked() {
                                    state.anchor_mark_session = Some(AnchorMarkSession {
                                        scope: AnchorScope::Local,
                                        is_multichar: false,
                                        input_buffer: String::new(),
                                    });
                                    ui.close();
                                }
                                if ui.button("📝 编辑 Lua 特效脚本 (e)").clicked() {
                                    state.command_input = format!(":editor {}", clip.name);
                                    state.is_command_mode = true;
                                    ui.close();
                                }
                                ui.separator();
                                if ui.button("🎬 导出此切片 (:export)").clicked() {
                                    state.export_state.output_path = format!("{}_export.mp4", clip.name);
                                    state.show_export_modal = true;
                                    ui.close();
                                }
                            });
                        }
                    }

                    // 3.3 绘制垃圾回收轨道 (Trash Track - 始终在最底部)
                    let trash_y = scroll_rect.min.y
                        + ruler_height
                        + (project.timeline.tracks.len() as f32) * track_height;
                    let trash_header_rect = Rect::from_min_size(
                        pos2(scroll_rect.min.x, trash_y),
                        vec2(track_header_width, track_height),
                    );
                    painter.rect_filled(
                        trash_header_rect,
                        CornerRadius::same(0),
                        Theme::TRACK_TRASH,
                    );
                    painter.line_segment(
                        [
                            pos2(trash_header_rect.max.x, trash_header_rect.min.y),
                            trash_header_rect.max,
                        ],
                        Stroke::new(1.0, Theme::BORDER_MEDIUM),
                    );

                    painter.text(
                        pos2(
                            trash_header_rect.min.x + 8.0,
                            trash_header_rect.min.y + 12.0,
                        ),
                        egui::Align2::LEFT_TOP,
                        "🗑 垃圾轨道",
                        egui::FontId::proportional(12.0),
                        Theme::TEXT_MUTED,
                    );
                    painter.text(
                        pos2(
                            trash_header_rect.min.x + 8.0,
                            trash_header_rect.min.y + 32.0,
                        ),
                        egui::Align2::LEFT_TOP,
                        "已删除片段暂存",
                        egui::FontId::proportional(10.0),
                        Theme::TEXT_MUTED,
                    );

                    let trash_content_rect = Rect::from_min_size(
                        pos2(scroll_rect.min.x + track_header_width, trash_y),
                        vec2(virtual_width, track_height),
                    );
                    painter.rect_filled(trash_content_rect, 0.0, Color32::from_rgb(18, 19, 22));
                    painter.line_segment(
                        [
                            pos2(trash_content_rect.min.x, trash_content_rect.max.y),
                            trash_content_rect.max,
                        ],
                        Stroke::new(1.0, Theme::BORDER_SUBTLE),
                    );

                    // 3.4 绘制 Visual 视觉选区 (如果在 Visual 模式且有选区)
                    if let Some(v_start) = state.visual_start_us {
                        let v_end = state.visual_end_us.unwrap_or(state.playhead_us);
                        let v_left = v_start.min(v_end);
                        let v_right = v_start.max(v_end);
                        let x1 =
                            ruler_rect.min.x + ((v_left as f32 / 1_000_000.0) * state.zoom_level);
                        let x2 =
                            ruler_rect.min.x + ((v_right as f32 / 1_000_000.0) * state.zoom_level);
                        let visual_rect = Rect::from_min_max(
                            pos2(x1, scroll_rect.min.y),
                            pos2(x2, scroll_rect.max.y),
                        );
                        painter.rect_filled(
                            visual_rect,
                            CornerRadius::ZERO,
                            Color32::from_rgba_unmultiplied(160, 100, 240, 45),
                        );
                        painter.rect_stroke(
                            visual_rect,
                            CornerRadius::ZERO,
                            Stroke::new(1.5, Theme::ACCENT_PURPLE),
                            egui::StrokeKind::Inside,
                        );
                    }

                    // 3.5 绘制 Biset 二分法决策动态分割图层 (两半区 + 标签)
                    if let Some(ref biset) = state.biset_session {
                        let s_us = biset.current_range.0 .0;
                        let e_us = biset.current_range.1 .0;
                        let mid_us = biset.midpoint().0;

                        let x_s =
                            ruler_rect.min.x + ((s_us as f32 / 1_000_000.0) * state.zoom_level);
                        let x_mid =
                            ruler_rect.min.x + ((mid_us as f32 / 1_000_000.0) * state.zoom_level);
                        let x_e =
                            ruler_rect.min.x + ((e_us as f32 / 1_000_000.0) * state.zoom_level);

                        // 左半区 (Cyan)
                        let left_biset_rect = Rect::from_min_max(
                            pos2(x_s, scroll_rect.min.y),
                            pos2(x_mid, scroll_rect.max.y),
                        );
                        painter.rect_filled(
                            left_biset_rect,
                            CornerRadius::ZERO,
                            Color32::from_rgba_unmultiplied(0, 210, 255, 55),
                        );
                        // 左半区标签 [-]
                        let left_badge_rect = Rect::from_min_size(
                            pos2(x_s + 4.0, scroll_rect.min.y + 4.0),
                            vec2(48.0, 20.0),
                        );
                        painter.rect_filled(
                            left_badge_rect,
                            CornerRadius::same(4),
                            Color32::from_rgb(0, 30, 45),
                        );
                        painter.text(
                            left_badge_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "[-] 左侧",
                            egui::FontId::proportional(11.0),
                            Theme::ACCENT_CYAN,
                        );

                        // 右半区 (Orange)
                        let right_biset_rect = Rect::from_min_max(
                            pos2(x_mid, scroll_rect.min.y),
                            pos2(x_e, scroll_rect.max.y),
                        );
                        painter.rect_filled(
                            right_biset_rect,
                            CornerRadius::ZERO,
                            Color32::from_rgba_unmultiplied(255, 170, 0, 55),
                        );
                        // 右半区标签 [=]
                        let right_badge_rect = Rect::from_min_size(
                            pos2(x_mid + 4.0, scroll_rect.min.y + 4.0),
                            vec2(48.0, 20.0),
                        );
                        painter.rect_filled(
                            right_badge_rect,
                            CornerRadius::same(4),
                            Color32::from_rgb(45, 25, 0),
                        );
                        painter.text(
                            right_badge_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "[=] 右侧",
                            egui::FontId::proportional(11.0),
                            Theme::ACCENT_ORANGE,
                        );

                        // 中线分割线
                        painter.line_segment(
                            [
                                pos2(x_mid, scroll_rect.min.y),
                                pos2(x_mid, scroll_rect.max.y),
                            ],
                            Stroke::new(2.0, Color32::WHITE),
                        );
                    }

                    // 3.6 绘制播放指针 (Playhead Scrubber Line)
                    let playhead_x = ruler_rect.min.x
                        + ((state.playhead_us as f32 / 1_000_000.0) * state.zoom_level);
                    let playhead_stroke = Stroke::new(2.0, Theme::ACCENT_CYAN);
                    painter.line_segment(
                        [
                            pos2(playhead_x, scroll_rect.min.y),
                            pos2(playhead_x, scroll_rect.max.y),
                        ],
                        playhead_stroke,
                    );

                    // 播放头顶部小三角把手
                    let handle_rect = Rect::from_center_size(
                        pos2(playhead_x, ruler_rect.min.y + 6.0),
                        vec2(14.0, 12.0),
                    );
                    painter.rect_filled(handle_rect, CornerRadius::same(3), Theme::ACCENT_CYAN);

                    // 3.7 绘制磁性吸附对齐标尺辅助线 (Magnetic Smart Guide Line)
                    if let Some(ref snap) = state.active_snap_guide {
                        let snap_x = ruler_rect.min.x + ((snap.snapped_time.0 as f32 / 1_000_000.0) * state.zoom_level);
                        painter.line_segment(
                            [pos2(snap_x, scroll_rect.min.y), pos2(snap_x, scroll_rect.max.y)],
                            Stroke::new(1.5, Theme::ACCENT_ORANGE),
                        );
                        let guide_text = format!("🧲 对齐: {}", snap.snap_point.description);
                        let badge_rect = Rect::from_min_size(pos2(snap_x + 6.0, ruler_rect.min.y + 2.0), vec2(170.0, 18.0));
                        painter.rect_filled(badge_rect, CornerRadius::same(3), Color32::from_rgb(40, 25, 0));
                        painter.rect_stroke(badge_rect, CornerRadius::same(3), Stroke::new(1.0, Theme::ACCENT_ORANGE), egui::StrokeKind::Inside);
                        painter.text(
                            badge_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            guide_text,
                            egui::FontId::proportional(10.0),
                            Theme::ACCENT_ORANGE,
                        );
                    }
                });
        });
    });

    // 4. 底部 Vim 状态栏 & 命令栏
    let status_height = 32.0;
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

            // Vim Mode Pill
            let (mode_label, mode_bg) = if state.biset_session.is_some() {
                (" BISET ", Color32::from_rgb(255, 170, 0))
            } else if state.anchor_mark_session.is_some() {
                (" MARK ", Theme::ACCENT_ORANGE)
            } else if state.anchor_jump_session.is_some() {
                (" GOTO ", Theme::ACCENT_CYAN)
            } else if state.is_command_mode {
                (" COMMAND ", Theme::ACCENT_CYAN)
            } else {
                match state.current_mode {
                    Mode::Visual => (" VISUAL ", Theme::ACCENT_PURPLE),
                    Mode::VisualLine => (" V-LINE ", Theme::ACCENT_PURPLE),
                    Mode::Insert => (" INSERT ", Color32::from_rgb(80, 200, 120)),
                    _ => (" NORMAL ", Theme::ACCENT_CYAN),
                }
            };
            let mode_badge = RichText::new(mode_label).size(12.0).strong().color(Color32::BLACK).background_color(mode_bg);
            ui.label(mode_badge);

            ui.add_space(10.0);
            if let Some(reg) = state.macro_recorder.current_register() {
                ui.label(
                    RichText::new(format!("● RECORDING @{}", reg))
                        .size(12.5)
                        .color(Theme::ACCENT_ORANGE)
                        .strong(),
                );
                ui.add_space(8.0);
            } else if let Some(prefix) = state.macro_pending_prefix {
                let hint = if prefix == 'q' {
                    "宏录制: 请输入目标寄存器 (a-z)..."
                } else {
                    "宏回放: 请输入寄存器 (a-z) 或再次按 @ 重复执行..."
                };
                ui.label(RichText::new(hint).size(12.0).color(Theme::ACCENT_ORANGE).strong());
                ui.add_space(8.0);
            }

            if let Some(ref mark) = state.anchor_mark_session {
                let scope_text = if mark.scope == AnchorScope::Global { "全局" } else { "切片" };
                if mark.is_multichar {
                    ui.label(RichText::new(format!("锚点命名 ({}) : {}_", scope_text, mark.input_buffer)).size(12.5).color(Theme::ACCENT_ORANGE).strong());
                    ui.label(RichText::new("(按 Enter 确认命名，按 Esc 取消)").size(11.0).color(Theme::TEXT_MUTED));
                } else {
                    ui.label(RichText::new(format!("设置锚点 ({}) : 请输入锚点字母/数字（按 ':' 进行多字符命名）", scope_text)).size(12.0).color(Theme::ACCENT_ORANGE).strong());
                }
            } else if state.biset_session.is_some() {
                ui.label(RichText::new("二分区间定位中: [-] 选左半区 | [=] 选右半区 | [Backspace] 上一级 | [Enter] 确定 | [Esc] 退出").size(12.0).color(Theme::ACCENT_CYAN).strong());
            } else if state.is_command_mode {
                ui.label(RichText::new(":").size(14.0).strong().color(Theme::ACCENT_CYAN));
                let cmd_edit = ui.add(egui::TextEdit::singleline(&mut state.command_input).hint_text("输入命令 (split, merge, mergecut, biset, goto, name, Marks, message, history)...").desired_width(400.0));
                cmd_edit.request_focus();
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    state.is_command_mode = false;
                }
            } else if let Some(ref msg) = state.status_message {
                ui.label(RichText::new(msg).size(12.0).color(Theme::ACCENT_ORANGE).strong());
            } else if state.current_mode == Mode::VisualLine {
                ui.label(RichText::new(format!("VISUAL LINE 模式: 已选中 {} 个切片 | [h/l] 扩选左右切片 | [d] 移至垃圾箱 | [:merge] 合并 | [v/Esc] 退出", state.visual_line_selected_clips.len().max(1))).size(12.0).color(Theme::ACCENT_PURPLE));
            } else if state.current_mode == Mode::Visual {
                ui.label(RichText::new("VISUAL 模式: 移动播放头扩选 | [:biset] 二分法定位 | [:mergecut] 选区切断合并 | [d] 移至垃圾箱 | [v] 退出").size(12.0).color(Theme::ACCENT_PURPLE));
            } else {
                ui.label(RichText::new("按冒号 ':' 输入指令 | 按 '?' 帮助 | [m/M] 锚点 | ['] 跳锚点 | 空格 播放/暂停 | 's' 分割 | 'v/V' 选区").size(12.0).color(Theme::TEXT_MUTED));
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(14.0);
                ui.label(RichText::new("UTF-8 | 60 FPS | Audio Master Clock").size(11.0).color(Theme::TEXT_MUTED));
                ui.add_space(8.0);
                let snap_text = if state.snapping_enabled { "🧲 SNAP: ON" } else { "🧲 SNAP: OFF" };
                let snap_color = if state.snapping_enabled { Theme::ACCENT_ORANGE } else { Theme::TEXT_MUTED };
                if ui.button(RichText::new(snap_text).size(11.0).color(snap_color).strong()).clicked() {
                    state.snapping_enabled = !state.snapping_enabled;
                }
            });
        });
    });

    // 5. 浮动历史/消息浮窗 (用于 :message, :history, :Marks)
    if state.show_message_window {
        draw_message_window(ui, state);
    }

    // 6. 浮动帮助面板 (Help Modal Overlay - 按 '?' 唤出)
    if state.show_help_modal {
        draw_help_modal(ui, state);
    }

    // 7. 浮动锚点跳转面板 (Anchor Jump Modal - 按 '\'' 唤出)
    if state.anchor_jump_session.is_some() {
        draw_anchor_jump_modal(ui, project, state);
    }

    // 8. 浮动独立命令帮助参考面板 (Command Help Modal - :help / :h 唤出)
    if state.show_command_help_modal {
        draw_command_help_modal(ui, state);
    }

    // 9. 浮动独立锚点管理与描述编辑面板 (Marks Manager Modal - :Marks / :marks 唤出)
    if state.show_marks_manager_modal {
        draw_marks_manager_modal(ui, project, state);
    }

    // 10. 浮动独立历史命令记录面板 (History Modal - :history 唤出)
    if state.show_history_modal {
        draw_history_modal(ui, state);
    }

    // 11. 浮动独立视频渲染导出面板 (Export Modal - :export 唤出)
    if state.show_export_modal {
        draw_export_modal(ui, project, state);
    }

    // 12. 浮动独立关键帧贝塞尔缓动曲线可视化编辑器 (Easing Modal - :easing / :curve 唤出)
    if state.show_easing_modal {
        draw_easing_modal(ui, project, state);
    }
}

/// 绘制单个剪辑卡片
fn draw_clip_card(
    painter: &egui::Painter,
    content_rect: Rect,
    clip: &Clip,
    state: &MainInterfaceUiState,
    track_idx: usize,
) {
    let us_per_sec = 1_000_000.0;
    let x_start = content_rect.min.x + ((clip.timeline_start.0 as f32 / us_per_sec) * state.zoom_level);
    let width = ((clip.duration().0 as f32 / us_per_sec) * state.zoom_level).max(6.0);

    let clip_rect = Rect::from_min_size(
        pos2(x_start, content_rect.min.y + 6.0),
        vec2(width, content_rect.height() - 12.0),
    );

    let is_vline_selected = state.current_mode == Mode::VisualLine && state.visual_line_selected_clips.contains(&clip.id);

    let (fill_color, border_color) = if is_vline_selected {
        (Color32::from_rgb(55, 30, 85), Theme::ACCENT_PURPLE)
    } else if track_idx == 0 {
        (Theme::TRACK_V1, Theme::TRACK_V1_BORDER)
    } else if track_idx == 1 {
        (Theme::TRACK_A1, Theme::TRACK_A1_BORDER)
    } else {
        (Theme::TRACK_V2, Theme::TRACK_V2_BORDER)
    };

    // 剪辑背景与精致发光边框
    painter.rect_filled(clip_rect, CornerRadius::same(5), fill_color);
    painter.rect_stroke(
        clip_rect,
        CornerRadius::same(5),
        Stroke::new(if is_vline_selected { 2.0 } else { 1.0 }, border_color),
        egui::StrokeKind::Inside,
    );

    // 如果处于 V-LINE 选区，加一层半透明高亮
    if is_vline_selected {
        painter.rect_filled(clip_rect, CornerRadius::same(5), Color32::from_rgba_unmultiplied(180, 100, 255, 45));
    }

    // 如果是音频轨道，绘制波形折线模拟
    if track_idx == 1 {
        let center_y = clip_rect.center().y;
        let mut x = clip_rect.min.x + 4.0;
        while x < clip_rect.max.x - 4.0 {
            let height = ((x * 0.15).sin().abs() * (clip_rect.height() * 0.35)).max(2.0);
            painter.line_segment(
                [pos2(x, center_y - height), pos2(x, center_y + height)],
                Stroke::new(1.5, Color32::from_rgb(120, 240, 200)),
            );
            x += 4.0;
        }
    } else {
        // 视频轨道绘制缩略图小格模拟
        let thumb_strip = Rect::from_min_size(
            clip_rect.min + vec2(3.0, 3.0),
            vec2(32.0, clip_rect.height() - 6.0),
        );
        painter.rect_filled(
            thumb_strip,
            CornerRadius::same(3),
            Color32::from_black_alpha(100),
        );
        painter.text(
            thumb_strip.center(),
            egui::Align2::CENTER_CENTER,
            "🎞",
            egui::FontId::proportional(12.0),
            Color32::WHITE,
        );
    }

    // 绘制切片内的局部锚点 (Local Anchors on Clip)
    for (name, anchor) in &clip.anchors {
        let a_us = anchor.position.0;
        let ax = clip_rect.min.x + ((a_us as f32 / us_per_sec) * state.zoom_level);
        if ax >= clip_rect.min.x && ax <= clip_rect.max.x {
            painter.circle_filled(pos2(ax, clip_rect.min.y + 4.0), 3.5, Theme::ACCENT_ORANGE);
            painter.text(
                pos2(ax + 3.0, clip_rect.min.y + 1.0),
                egui::Align2::LEFT_TOP,
                name,
                egui::FontId::monospace(9.0),
                Theme::ACCENT_ORANGE,
            );
        }
    }

    // 剪辑名称文本
    let text_pos = if track_idx == 1 {
        clip_rect.min + vec2(8.0, 4.0)
    } else {
        clip_rect.min + vec2(40.0, 6.0)
    };

    painter.with_clip_rect(clip_rect).text(
        text_pos,
        egui::Align2::LEFT_TOP,
        &clip.name,
        egui::FontId::proportional(12.0),
        Color32::WHITE,
    );

    // 如果切片锁定，绘制锁定角标
    if clip.locked {
        painter.text(
            clip_rect.max - vec2(14.0, 14.0),
            egui::Align2::RIGHT_BOTTOM,
            "🔒",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(255, 200, 80),
        );
    }

    // 如果切片经过变速设置，绘制速度角标
    if (clip.speed - 1.0).abs() > 0.01 {
        let offset_x = if clip.locked { 28.0 } else { 10.0 };
        painter.text(
            clip_rect.max - vec2(offset_x, 14.0),
            egui::Align2::RIGHT_BOTTOM,
            format!("⚡{:.1}x", clip.speed),
            egui::FontId::monospace(9.5),
            Theme::ACCENT_CYAN,
        );
    }

    // 绘制音频淡入淡出几何曲线包络蒙版 (Audio Fade Envelopes & Curve Ramps)
    if clip.audio_fade_in.0 > 0 {
        let in_w = ((clip.audio_fade_in.0 as f32 / us_per_sec) * state.zoom_level).min(clip_rect.width());
        // 斜向淡入斜坡线
        painter.line_segment(
            [pos2(clip_rect.min.x, clip_rect.max.y), pos2(clip_rect.min.x + in_w, clip_rect.min.y)],
            Stroke::new(1.5, Color32::from_rgb(255, 230, 100)),
        );
        // 淡入顶点小手柄
        painter.circle_filled(pos2(clip_rect.min.x + in_w, clip_rect.min.y + 2.0), 3.0, Color32::WHITE);
    }

    if clip.audio_fade_out.0 > 0 {
        let out_w = ((clip.audio_fade_out.0 as f32 / us_per_sec) * state.zoom_level).min(clip_rect.width());
        let out_start_x = clip_rect.max.x - out_w;
        // 斜向淡出斜坡线
        painter.line_segment(
            [pos2(out_start_x, clip_rect.min.y), pos2(clip_rect.max.x, clip_rect.max.y)],
            Stroke::new(1.5, Color32::from_rgb(255, 230, 100)),
        );
        // 淡出起始小手柄
        painter.circle_filled(pos2(out_start_x, clip_rect.min.y + 2.0), 3.0, Color32::WHITE);
    }

    // 如果切片尾部设置了转场特效，在接缝右端绘制转场标志 Badge
    if let Some(ref trans) = clip.transition_out {
        let trans_w = ((trans.duration.0 as f32 / us_per_sec) * state.zoom_level).clamp(16.0, 48.0);
        let trans_rect = Rect::from_min_size(
            pos2(clip_rect.max.x - trans_w * 0.5, clip_rect.min.y + 4.0),
            vec2(trans_w, clip_rect.height() - 8.0),
        );
        painter.rect_filled(
            trans_rect,
            CornerRadius::same(3),
            Color32::from_rgba_unmultiplied(130, 80, 240, 190),
        );
        painter.rect_stroke(
            trans_rect,
            CornerRadius::same(3),
            Stroke::new(1.0, Color32::WHITE),
            egui::StrokeKind::Outside,
        );
        painter.text(
            trans_rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("✨{}", trans.transition_type.short_code()),
            egui::FontId::proportional(9.0),
            Color32::WHITE,
        );
    }

    // 如果切片包含文本覆盖，绘制字幕角标
    if let Some(ref text) = clip.text_overlay {
        if !text.is_empty() {
            let preview: String = text.content.chars().take(8).collect();
            let label = if text.content.chars().count() > 8 {
                format!("💬{}...", preview)
            } else {
                format!("💬{}", preview)
            };
            painter.text(
                pos2(clip_rect.min.x + 8.0, clip_rect.max.y - 12.0),
                egui::Align2::LEFT_BOTTOM,
                label,
                egui::FontId::proportional(10.0),
                Theme::ACCENT_YELLOW,
            );
        }
    }
}

/// 绘制浮动只读消息/历史输出编辑器窗口
fn draw_message_window(ui: &mut Ui, state: &mut MainInterfaceUiState) {
    let full_rect = ui.max_rect();
    let win_w = 600.0;
    let win_h = 280.0;
    let win_rect = Rect::from_min_size(
        pos2(full_rect.min.x + 20.0, full_rect.max.y - 45.0 - win_h),
        vec2(win_w, win_h),
    );

    let painter = ui.painter();
    painter.rect_filled(win_rect, CornerRadius::same(8), Theme::BG_PANEL_ALT);
    painter.rect_stroke(
        win_rect,
        CornerRadius::same(8),
        Stroke::new(1.5, Theme::ACCENT_CYAN),
        egui::StrokeKind::Inside,
    );

    ui.scope_builder(UiBuilder::new().max_rect(win_rect.shrink(12.0)), |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("📋 系统输出与历史记录 (:message)")
                        .strong()
                        .size(13.0)
                        .color(Theme::ACCENT_CYAN),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(RichText::new(" ✕ 关闭 (Esc) ").size(11.0))
                        .clicked()
                    {
                        state.show_message_window = false;
                    }
                });
            });
            ui.separator();
            ui.add_space(4.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                if state.history_output.is_empty() {
                    ui.label(
                        RichText::new("（暂无历史输出）")
                            .color(Theme::TEXT_MUTED)
                            .size(12.0),
                    );
                } else {
                    for (i, line) in state.history_output.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("{:>2}. ", i + 1))
                                    .monospace()
                                    .color(Theme::TEXT_MUTED)
                                    .size(11.0),
                            );
                            ui.label(
                                RichText::new(line)
                                    .monospace()
                                    .color(Theme::TEXT_PRIMARY)
                                    .size(12.0),
                            );
                        });
                    }
                }
            });
        });
    });
}

/// 绘制浮动帮助面板
fn draw_help_modal(ui: &mut Ui, state: &mut MainInterfaceUiState) {
    let full_rect = ui.max_rect();
    let modal_w = 680.0;
    let modal_h = 360.0;
    let modal_rect = Rect::from_center_size(full_rect.center(), vec2(modal_w, modal_h));

    ui.painter()
        .rect_filled(full_rect, 0.0, Color32::from_black_alpha(150)); // 半透明蒙层

    ui.painter()
        .rect_filled(modal_rect, CornerRadius::same(10), Theme::BG_PANEL_ALT);
    ui.painter().rect_stroke(
        modal_rect,
        CornerRadius::same(10),
        Stroke::new(1.5, Theme::BORDER_MEDIUM),
        egui::StrokeKind::Inside,
    );

    ui.scope_builder(UiBuilder::new().max_rect(modal_rect.shrink(16.0)), |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("⌨ 全局快捷键指南 (Cheat Sheet)")
                        .size(16.0)
                        .strong()
                        .color(Theme::ACCENT_CYAN),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(RichText::new(" ✕ 关闭 (Esc) ").size(12.0))
                        .clicked()
                    {
                        state.show_help_modal = false;
                    }
                });
            });

            ui.separator();
            ui.add_space(8.0);

            ui.columns(3, |columns| {
                // 第 1 列: 导航与光标
                columns[0].vertical(|ui| {
                    ui.label(
                        RichText::new("【 移动与导航 】")
                            .strong()
                            .color(Theme::TEXT_PRIMARY),
                    );
                    ui.add_space(4.0);
                    let keys = [
                        ("Space", "播放 / 暂停"),
                        ("h / l", "左 / 右微调光标"),
                        ("j / k", "上 / 下切换轨道"),
                        ("J / K", "跳到上一/下一断点"),
                        ("w / b", "跳到上一/下一音频波形"),
                        ("W / B", "跳到上一/下一锚点"),
                    ];
                    for (k, d) in keys {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(k)
                                    .monospace()
                                    .color(Theme::ACCENT_ORANGE)
                                    .strong(),
                            );
                            ui.label(RichText::new(d).size(11.0).color(Theme::TEXT_SECONDARY));
                        });
                    }
                });

                // 第 2 列: 切割与编辑
                columns[1].vertical(|ui| {
                    ui.label(
                        RichText::new("【 编辑与切片 】")
                            .strong()
                            .color(Theme::TEXT_PRIMARY),
                    );
                    ui.add_space(4.0);
                    let keys = [
                        ("s", "在当前光标分割 (:split)"),
                        ("i / +", "导入媒体素材"),
                        ("m<key>", "设置局部锚点"),
                        ("M<key>", "设置轨道全局锚点"),
                        ("'<key>", "快速跳转到锚点"),
                        ("Shift+Enter", "新建一条下方轨道"),
                        ("Alt+j/k", "上移/下移当前轨道"),
                    ];
                    for (k, d) in keys {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(k)
                                    .monospace()
                                    .color(Theme::ACCENT_CYAN)
                                    .strong(),
                            );
                            ui.label(RichText::new(d).size(11.0).color(Theme::TEXT_SECONDARY));
                        });
                    }
                });

                // 第 3 列: 选择与模式
                columns[2].vertical(|ui| {
                    ui.label(
                        RichText::new("【 选择与高级指令 】")
                            .strong()
                            .color(Theme::TEXT_PRIMARY),
                    );
                    ui.add_space(4.0);
                    let keys = [
                        ("v", "进入连续时间范围选择"),
                        ("V", "进入切片多选模式 (V5l)"),
                        (":", "输入命令 (:editor, :goto)"),
                        (":goto <t>", "精确跳转到时间点"),
                        (":biset", "二分法快速区间筛选"),
                        (":merge", "合并选中的切片"),
                    ];
                    for (k, d) in keys {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(k)
                                    .monospace()
                                    .color(Theme::ACCENT_PURPLE)
                                    .strong(),
                            );
                            ui.label(RichText::new(d).size(11.0).color(Theme::TEXT_SECONDARY));
                        });
                    }
                });
            });
        });
    });
}

fn format_timecode(us: i64) -> String {
    let total_secs = us / 1_000_000;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    let frames = (us % 1_000_000) / 16_666; // 60fps 假定
    format!("00:{:02}:{:02}:{:02}", mins, secs, frames)
}

/// 绘制浮动锚点跳转面板 (按 ' 或 中文 ’ 呼出)
fn draw_anchor_jump_modal(ui: &mut Ui, project: &ProjectState, state: &mut MainInterfaceUiState) {
    let full_rect = ui.max_rect();
    let modal_w = 720.0;
    let modal_h = 420.0;
    let modal_rect = Rect::from_center_size(full_rect.center(), vec2(modal_w, modal_h));

    ui.painter().rect_filled(full_rect, 0.0, Color32::from_black_alpha(160)); // 半透明背景遮罩
    ui.painter().rect_filled(modal_rect, CornerRadius::same(10), Theme::BG_PANEL_ALT);
    ui.painter().rect_stroke(
        modal_rect,
        CornerRadius::same(10),
        Stroke::new(1.5, Theme::ACCENT_CYAN),
        egui::StrokeKind::Inside,
    );

    let all_anchors = collect_all_anchors(project);
    let mut jump_target_pos = None;
    let mut close_modal = false;

    ui.scope_builder(UiBuilder::new().max_rect(modal_rect.shrink(16.0)), |ui| {
        ui.vertical(|ui| {
            // 顶部标题栏
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("🎯 锚点跳转 (Anchor Jump - ' / Marks)")
                        .size(16.0)
                        .strong()
                        .color(Theme::ACCENT_CYAN),
                );
                ui.add_space(10.0);
                ui.label(
                    RichText::new("按单键立即跳转 | 按 ':' 或 '\'' 进入多字符搜索")
                        .size(12.0)
                        .color(Theme::TEXT_MUTED),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new(" ✕ 关闭 (Esc) ").size(12.0)).clicked() {
                        close_modal = true;
                    }
                });
            });
            ui.separator();
            ui.add_space(6.0);

            if let Some(ref mut session) = state.anchor_jump_session {
                // 如果是多字符搜索模式或输入了 query
                if session.is_multichar || !session.query.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🔍 搜索锚点:").size(13.0).color(Theme::ACCENT_CYAN).strong());
                        let search_edit = ui.add(
                            egui::TextEdit::singleline(&mut session.query)
                                .hint_text("输入锚点名称或拼音 (Alt+j/k 选择, Enter 跳转)...")
                                .desired_width(450.0),
                        );
                        search_edit.request_focus();
                    });
                    ui.add_space(8.0);

                    // 过滤与排序
                    let mut matched: Vec<(AnchorItemView, u32, Vec<usize>)> = all_anchors
                        .into_iter()
                        .filter_map(|item| {
                            if session.query.trim().is_empty() {
                                Some((item, 100, Vec::new()))
                            } else {
                                crate::search::PinyinFuzzyMatcher::match_query(&item.name, &session.query)
                                    .map(|res| (item, res.score, res.matched_indices))
                            }
                        })
                        .collect();
                    matched.sort_by_key(|b| std::cmp::Reverse(b.1));

                    if matched.is_empty() {
                        ui.label(RichText::new("未找到匹配的锚点。按 Esc 退出。").color(Theme::TEXT_MUTED).size(13.0));
                    } else {
                        if session.selected_idx >= matched.len() {
                            session.selected_idx = 0;
                        }

                        egui::ScrollArea::vertical().id_salt("anchor_jump_multichar_scroll").show(ui, |ui| {
                            for (idx, (item, _score, _matched_indices)) in matched.iter().enumerate() {
                                let is_selected = idx == session.selected_idx;
                                let item_rect = ui.allocate_space(vec2(modal_w - 50.0, 44.0)).1;
                                let painter = ui.painter_at(item_rect);

                                let bg = if is_selected {
                                    Theme::BG_CARD_ACTIVE
                                } else {
                                    Theme::BG_CARD
                                };
                                painter.rect_filled(item_rect, CornerRadius::same(6), bg);
                                painter.rect_stroke(
                                    item_rect,
                                    CornerRadius::same(6),
                                    Stroke::new(if is_selected { 1.5 } else { 1.0 }, if is_selected { Theme::ACCENT_CYAN } else { Theme::BORDER_SUBTLE }),
                                    egui::StrokeKind::Inside,
                                );

                                // 标识 Badge
                                let badge_rect = Rect::from_min_size(item_rect.min + vec2(8.0, 8.0), vec2(28.0, 28.0));
                                painter.rect_filled(badge_rect, CornerRadius::same(4), if item.is_global { Theme::ACCENT_CYAN } else { Theme::ACCENT_ORANGE });
                                painter.text(
                                    badge_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    &item.name,
                                    egui::FontId::monospace(13.0),
                                    Color32::BLACK,
                                );

                                // 名称
                                let text_pos = item_rect.min + vec2(46.0, 8.0);
                                painter.text(
                                    text_pos,
                                    egui::Align2::LEFT_TOP,
                                    &item.name,
                                    egui::FontId::proportional(13.5),
                                    Theme::TEXT_PRIMARY,
                                );

                                // 作用域与描述
                                painter.text(
                                    item_rect.min + vec2(46.0, 26.0),
                                    egui::Align2::LEFT_TOP,
                                    format!("{}  |  {}", item.scope_label, item.description),
                                    egui::FontId::proportional(11.0),
                                    Theme::TEXT_MUTED,
                                );

                                // 右侧时间码
                                painter.text(
                                    item_rect.max - vec2(12.0, 14.0),
                                    egui::Align2::RIGHT_BOTTOM,
                                    format_timecode(item.position_us),
                                    egui::FontId::monospace(13.0),
                                    Theme::ACCENT_CYAN,
                                );

                                if ui.rect_contains_pointer(item_rect) && ui.input(|i| i.pointer.primary_clicked()) {
                                    jump_target_pos = Some(item.position_us);
                                }
                            }
                        });
                    }
                } else {
                    // 单键快捷选择网格模式
                    egui::ScrollArea::vertical().id_salt("anchor_jump_grid_scroll").show(ui, |ui| {
                        ui.columns(2, |cols| {
                            // 前两项固定项：' 与 :
                            cols[0].vertical(|ui| {
                                let rect1 = ui.allocate_space(vec2(310.0, 48.0)).1;
                                let p1 = ui.painter_at(rect1);
                                p1.rect_filled(rect1, CornerRadius::same(6), Theme::BG_CARD);
                                p1.rect_stroke(rect1, CornerRadius::same(6), Stroke::new(1.0, Theme::BORDER_SUBTLE), egui::StrokeKind::Inside);
                                p1.text(rect1.min + vec2(12.0, 12.0), egui::Align2::LEFT_TOP, "['] 或 [:] 多字符搜索锚点", egui::FontId::proportional(13.0), Theme::ACCENT_CYAN);
                                p1.text(rect1.min + vec2(12.0, 30.0), egui::Align2::LEFT_TOP, "输入字符全文过滤匹配", egui::FontId::proportional(11.0), Theme::TEXT_MUTED);
                                if ui.rect_contains_pointer(rect1) && ui.input(|i| i.pointer.primary_clicked()) {
                                    session.is_multichar = true;
                                }
                            });

                            cols[1].vertical(|ui| {
                                let rect2 = ui.allocate_space(vec2(310.0, 48.0)).1;
                                let p2 = ui.painter_at(rect2);
                                p2.rect_filled(rect2, CornerRadius::same(6), Theme::BG_CARD);
                                p2.rect_stroke(rect2, CornerRadius::same(6), Stroke::new(1.0, Theme::BORDER_SUBTLE), egui::StrokeKind::Inside);
                                p2.text(rect2.min + vec2(12.0, 12.0), egui::Align2::LEFT_TOP, format!("共有 {} 个锚点", all_anchors.len()), egui::FontId::proportional(13.0), Theme::TEXT_PRIMARY);
                                p2.text(rect2.min + vec2(12.0, 30.0), egui::Align2::LEFT_TOP, "直接敲击对应字母跳跃", egui::FontId::proportional(11.0), Theme::TEXT_MUTED);
                            });
                        });

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // 渲染所有锚点卡片
                        let card_w = 320.0;
                        let card_h = 56.0;
                        ui.horizontal_wrapped(|ui| {
                            for item in &all_anchors {
                                let item_rect = ui.allocate_space(vec2(card_w, card_h)).1;
                                let painter = ui.painter_at(item_rect);
                                let is_hover = ui.rect_contains_pointer(item_rect);

                                let bg = if is_hover { Theme::BG_CARD_HOVER } else { Theme::BG_CARD };
                                painter.rect_filled(item_rect, CornerRadius::same(6), bg);
                                painter.rect_stroke(
                                    item_rect,
                                    CornerRadius::same(6),
                                    Stroke::new(1.0, if is_hover { Theme::ACCENT_CYAN } else { Theme::BORDER_SUBTLE }),
                                    egui::StrokeKind::Inside,
                                );

                                // 左侧字母角标
                                let badge_rect = Rect::from_min_size(item_rect.min + vec2(8.0, 8.0), vec2(28.0, 28.0));
                                painter.rect_filled(badge_rect, CornerRadius::same(4), if item.is_global { Theme::ACCENT_CYAN } else { Theme::ACCENT_ORANGE });
                                painter.text(
                                    badge_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    &item.name,
                                    egui::FontId::monospace(14.0),
                                    Color32::BLACK,
                                );

                                // 中间描述与作用域
                                painter.text(
                                    item_rect.min + vec2(44.0, 8.0),
                                    egui::Align2::LEFT_TOP,
                                    format!("{} {}", item.scope_label, item.name),
                                    egui::FontId::proportional(13.0),
                                    Theme::TEXT_PRIMARY,
                                );
                                painter.text(
                                    item_rect.min + vec2(44.0, 28.0),
                                    egui::Align2::LEFT_TOP,
                                    &item.description,
                                    egui::FontId::proportional(10.5),
                                    Theme::TEXT_MUTED,
                                );

                                // 右侧时间码
                                painter.text(
                                    item_rect.max - vec2(10.0, 10.0),
                                    egui::Align2::RIGHT_BOTTOM,
                                    format_timecode(item.position_us),
                                    egui::FontId::monospace(12.0),
                                    Theme::ACCENT_CYAN,
                                );

                                if is_hover && ui.input(|i| i.pointer.primary_clicked()) {
                                    jump_target_pos = Some(item.position_us);
                                }
                            }
                        });
                    });
                }
            }
        });
    });

    if let Some(pos) = jump_target_pos {
        state.playhead_us = pos;
        state.status_message = Some(format!("已成功跳转到锚点: {}", format_timecode(pos)));
        close_modal = true;
    }

    if close_modal {
        state.anchor_jump_session = None;
    }
}

/// 绘制独立命令帮助手册弹窗 (:help / :h)
fn draw_command_help_modal(ui: &mut Ui, state: &mut MainInterfaceUiState) {
    let full_rect = ui.max_rect();
    let modal_w = 780.0;
    let modal_h = 520.0;
    let modal_rect = Rect::from_center_size(full_rect.center(), vec2(modal_w, modal_h));

    // 1. 半透明暗色背景遮罩
    ui.painter().rect_filled(full_rect, 0.0, Color32::from_black_alpha(160));

    // 2. 弹窗面板背景与发光边框
    ui.painter().rect_filled(modal_rect, CornerRadius::same(10), Theme::BG_PANEL_ALT);
    ui.painter().rect_stroke(
        modal_rect,
        CornerRadius::same(10),
        Stroke::new(1.5, Theme::ACCENT_CYAN),
        egui::StrokeKind::Inside,
    );

    let all_items = get_all_command_help_items();
    let query = state.command_help_search.trim().to_lowercase();
    let filtered_items: Vec<&CommandHelpItem> = all_items
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

    let mut fill_command = None;
    let mut close_modal = false;

    ui.scope_builder(UiBuilder::new().max_rect(modal_rect.shrink(18.0)), |ui| {
        ui.vertical(|ui| {
            // 顶栏：标题与关闭按钮
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("📖 VideoCut 命令行参考手册 (:help)")
                        .size(16.0)
                        .strong()
                        .color(Theme::ACCENT_CYAN),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!("(显示 {}/{} 条命令)", filtered_items.len(), all_items.len()))
                        .size(12.0)
                        .color(Theme::TEXT_MUTED),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new(" ✕ 关闭 (Esc) ").size(12.0)).clicked() {
                        close_modal = true;
                    }
                });
            });

            ui.add_space(8.0);

            // 搜索栏
            ui.horizontal(|ui| {
                ui.label(RichText::new("🔍 筛选:").color(Theme::ACCENT_CYAN).strong());
                let search_resp = ui.add(
                    egui::TextEdit::singleline(&mut state.command_help_search)
                        .hint_text("输入命令名/描述关键词 (按 / 键聚焦，按 Esc 退出)...")
                        .desired_width(ui.available_width() - 80.0),
                );
                if state.command_help_search_active {
                    search_resp.request_focus();
                    state.command_help_search_active = false;
                }
                if !state.command_help_search.is_empty() && ui.button("✕ 清空").clicked() {
                    state.command_help_search.clear();
                }
            });

            ui.add_space(4.0);
            ui.label(
                RichText::new("💡 快捷提示: 按 / 键聚焦搜索框 | j/k 上下选择 | Enter 填入命令行 | Esc 关闭")
                    .size(11.0)
                    .color(Theme::TEXT_MUTED),
            );
            ui.separator();
            ui.add_space(4.0);

            // 命令列表表头
            ui.horizontal(|ui| {
                ui.label(RichText::new("命令 (Command)").strong().size(12.0).color(Theme::ACCENT_CYAN));
                ui.add_space(80.0);
                ui.label(RichText::new("参数 (Args)").strong().size(12.0).color(Theme::TEXT_SECONDARY));
                ui.add_space(80.0);
                ui.label(RichText::new("功能描述 (Description)").strong().size(12.0).color(Theme::TEXT_PRIMARY));
            });
            ui.separator();

            // 命令列表滚动区
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if filtered_items.is_empty() {
                        ui.add_space(40.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("未找到匹配的命令")
                                    .size(14.0)
                                    .color(Theme::TEXT_MUTED),
                            );
                        });
                    } else {
                        for (idx, item) in filtered_items.iter().enumerate() {
                            let is_selected = idx == state.command_help_selected_idx;
                            let row_bg = if is_selected {
                                Color32::from_rgb(35, 45, 65)
                            } else if idx % 2 == 0 {
                                Color32::from_rgb(22, 25, 32)
                            } else {
                                Color32::from_rgb(26, 30, 40)
                            };

                            let (rect, resp) = ui.allocate_exact_size(
                                vec2(ui.available_width(), 34.0),
                                egui::Sense::click(),
                            );

                            if resp.hovered() {
                                ui.painter().rect_filled(rect, CornerRadius::same(4), Color32::from_rgb(45, 55, 75));
                            } else {
                                ui.painter().rect_filled(rect, CornerRadius::same(4), row_bg);
                            }

                            if is_selected {
                                ui.painter().rect_stroke(
                                    rect,
                                    CornerRadius::same(4),
                                    Stroke::new(1.5, Theme::ACCENT_CYAN),
                                    egui::StrokeKind::Inside,
                                );
                            }

                            // 绘制行内容
                            let y_center = rect.center().y;

                            // 1. 命令名称
                            ui.painter().text(
                                pos2(rect.min.x + 8.0, y_center),
                                egui::Align2::LEFT_CENTER,
                                item.name,
                                egui::FontId::monospace(13.0),
                                Theme::ACCENT_CYAN,
                            );

                            // 2. 别名 (若有)
                            let x_offset = rect.min.x + 85.0;
                            if !item.alias.is_empty() {
                                ui.painter().text(
                                    pos2(x_offset, y_center),
                                    egui::Align2::LEFT_CENTER,
                                    format!("[{}]", item.alias),
                                    egui::FontId::monospace(11.0),
                                    Theme::ACCENT_ORANGE,
                                );
                            }

                            // 3. 参数
                            ui.painter().text(
                                pos2(rect.min.x + 160.0, y_center),
                                egui::Align2::LEFT_CENTER,
                                item.args,
                                egui::FontId::proportional(11.0),
                                Theme::TEXT_SECONDARY,
                            );

                            // 4. 功能描述
                            ui.painter().text(
                                pos2(rect.min.x + 300.0, y_center),
                                egui::Align2::LEFT_CENTER,
                                item.description,
                                egui::FontId::proportional(12.0),
                                Theme::TEXT_PRIMARY,
                            );

                            // 5. 分类标签
                            let cat_rect = Rect::from_center_size(
                                pos2(rect.max.x - 45.0, y_center),
                                vec2(65.0, 20.0),
                            );
                            ui.painter().rect_filled(
                                cat_rect,
                                CornerRadius::same(3),
                                Color32::from_rgb(30, 40, 50),
                            );
                            ui.painter().text(
                                cat_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                item.category,
                                egui::FontId::proportional(10.0),
                                Theme::ACCENT_CYAN,
                            );

                            if resp.clicked() {
                                fill_command = Some(item.name.to_string());
                            }
                        }
                    }
                });
        });
    });

    if let Some(cmd) = fill_command {
        state.command_input = format!("{} ", cmd);
        state.is_command_mode = true;
        close_modal = true;
    }

    if close_modal {
        state.show_command_help_modal = false;
    }
}

/// 绘制独立锚点管理与描述编辑弹窗 (:Marks / :marks)
fn draw_marks_manager_modal(
    ui: &mut Ui,
    project: &mut ProjectState,
    state: &mut MainInterfaceUiState,
) {
    let full_rect = ui.max_rect();
    let modal_w = 840.0;
    let modal_h = 520.0;
    let modal_rect = Rect::from_center_size(full_rect.center(), vec2(modal_w, modal_h));

    // 1. 半透明暗色遮罩
    ui.painter().rect_filled(full_rect, 0.0, Color32::from_black_alpha(160));

    // 2. 面板背景与发光青色边框
    ui.painter().rect_filled(modal_rect, CornerRadius::same(10), Theme::BG_PANEL_ALT);
    ui.painter().rect_stroke(
        modal_rect,
        CornerRadius::same(10),
        Stroke::new(1.5, Theme::ACCENT_CYAN),
        egui::StrokeKind::Inside,
    );

    let mut jump_target_pos = None;
    let mut close_modal = false;
    let mut delete_global_anchor = None;
    let mut delete_local_anchor = None;

    ui.scope_builder(UiBuilder::new().max_rect(modal_rect.shrink(18.0)), |ui| {
        ui.vertical(|ui| {
            // 顶栏：标题与关闭按钮
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("📍 VideoCut 锚点管理与描述编辑 (:Marks)")
                        .size(16.0)
                        .strong()
                        .color(Theme::ACCENT_CYAN),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new(" ✕ 关闭 (Esc) ").size(12.0)).clicked() {
                        close_modal = true;
                    }
                });
            });

            ui.add_space(8.0);

            // 搜索栏
            ui.horizontal(|ui| {
                ui.label(RichText::new("🔍 筛选:").color(Theme::ACCENT_CYAN).strong());
                let search_resp = ui.add(
                    egui::TextEdit::singleline(&mut state.marks_manager_search)
                        .hint_text("输入锚点名称/描述关键词 (按 / 聚焦，按 Esc 退出)...")
                        .desired_width(ui.available_width() - 80.0),
                );
                if state.marks_manager_search_active {
                    search_resp.request_focus();
                    state.marks_manager_search_active = false;
                }
                if !state.marks_manager_search.is_empty() && ui.button("✕ 清空").clicked() {
                    state.marks_manager_search.clear();
                }
            });

            ui.add_space(4.0);
            ui.label(
                RichText::new("💡 快捷提示: 点击 [🚀 跳转] 或行内跳转定位 | 直接在描述框修改文本 | 按 / 搜索 | Esc 关闭")
                    .size(11.0)
                    .color(Theme::TEXT_MUTED),
            );
            ui.separator();
            ui.add_space(4.0);

            // 表头
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label(RichText::new("锚点名").strong().size(12.0).color(Theme::ACCENT_CYAN));
                ui.add_space(50.0);
                ui.label(RichText::new("时间码").strong().size(12.0).color(Theme::TEXT_SECONDARY));
                ui.add_space(60.0);
                ui.label(RichText::new("作用域").strong().size(12.0).color(Theme::ACCENT_PURPLE));
                ui.add_space(70.0);
                ui.label(RichText::new("描述 (可直接在此编辑修改)").strong().size(12.0).color(Theme::TEXT_PRIMARY));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(20.0);
                    ui.label(RichText::new("操作").strong().size(12.0).color(Theme::TEXT_MUTED));
                });
            });
            ui.separator();

            let query = state.marks_manager_search.trim().to_lowercase();

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let mut rendered_any = false;

                    // 1. 全局锚点
                    for anchor in project.timeline.global_anchors.anchors.iter_mut() {
                        let name = &anchor.name;
                        if !query.is_empty()
                            && !name.to_lowercase().contains(&query)
                            && !anchor.description.to_lowercase().contains(&query)
                            && !"全局".contains(&query)
                        {
                            continue;
                        }
                        rendered_any = true;

                        ui.horizontal(|ui| {
                            ui.label(RichText::new(name).strong().monospace().color(Theme::ACCENT_CYAN));
                            ui.add_space(10.0);
                            ui.label(RichText::new(format_timecode(anchor.position.0)).monospace().color(Theme::TEXT_SECONDARY));
                            ui.add_space(10.0);
                            ui.label(RichText::new("[全局轨道]").color(Theme::ACCENT_PURPLE));
                            ui.add_space(10.0);

                            // 内联编辑描述
                            ui.add(egui::TextEdit::singleline(&mut anchor.description).desired_width(280.0));

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(RichText::new("✕ 删除").color(Theme::ACCENT_ORANGE).size(11.0)).clicked() {
                                    delete_global_anchor = Some(name.clone());
                                }
                                if ui.button(RichText::new("🚀 跳转").color(Theme::ACCENT_CYAN).size(11.0)).clicked() {
                                    jump_target_pos = Some(anchor.position.0);
                                }
                            });
                        });
                        ui.separator();
                    }

                    // 2. 切片局部锚点
                    for track in &mut project.timeline.tracks {
                        for clip in &mut track.clips {
                            for (name, anchor) in clip.anchors.iter_mut() {
                                if !query.is_empty()
                                    && !name.to_lowercase().contains(&query)
                                    && !anchor.description.to_lowercase().contains(&query)
                                    && !clip.name.to_lowercase().contains(&query)
                                {
                                    continue;
                                }
                                rendered_any = true;
                                let abs_pos = clip.timeline_start.0 + anchor.position.0;

                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(name).strong().monospace().color(Theme::ACCENT_CYAN));
                                    ui.add_space(10.0);
                                    ui.label(RichText::new(format_timecode(abs_pos)).monospace().color(Theme::TEXT_SECONDARY));
                                    ui.add_space(10.0);
                                    ui.label(RichText::new(format!("[切片:{}]", clip.name)).color(Theme::ACCENT_PURPLE));
                                    ui.add_space(10.0);

                                    // 内联编辑描述
                                    ui.add(egui::TextEdit::singleline(&mut anchor.description).desired_width(280.0));

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button(RichText::new("✕ 删除").color(Theme::ACCENT_ORANGE).size(11.0)).clicked() {
                                            delete_local_anchor = Some((track.id, clip.id, name.clone()));
                                        }
                                        if ui.button(RichText::new("🚀 跳转").color(Theme::ACCENT_CYAN).size(11.0)).clicked() {
                                            jump_target_pos = Some(abs_pos);
                                        }
                                    });
                                });
                                ui.separator();
                            }
                        }
                    }

                    if !rendered_any {
                        ui.add_space(40.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("暂无匹配的锚点记录 (在主界面按 m 或 M 添加锚点)")
                                    .size(14.0)
                                    .color(Theme::TEXT_MUTED),
                            );
                        });
                    }
                });
        });
    });

    if let Some(name) = delete_global_anchor {
        project.timeline.global_anchors.anchors.retain(|a| a.name != name);
        state.status_message = Some(format!("已删除全局锚点 '{}'", name));
    }

    if let Some((track_id, clip_id, name)) = delete_local_anchor {
        if let Some(track) = project.timeline.tracks.iter_mut().find(|t| t.id == track_id) {
            if let Some(clip) = track.clips.iter_mut().find(|c| c.id == clip_id) {
                clip.anchors.remove(&name);
                state.status_message = Some(format!("已删除切片局部锚点 '{}'", name));
            }
        }
    }

    if let Some(pos) = jump_target_pos {
        state.playhead_us = pos;
        state.status_message = Some(format!("已跳转至锚点: {}", format_timecode(pos)));
        close_modal = true;
    }

    if close_modal {
        state.show_marks_manager_modal = false;
    }
}

/// 绘制独立交互式历史命令记录弹窗 (:history)
fn draw_history_modal(ui: &mut Ui, state: &mut MainInterfaceUiState) {
    let full_rect = ui.max_rect();
    let modal_w = 720.0;
    let modal_h = 480.0;
    let modal_rect = Rect::from_center_size(full_rect.center(), vec2(modal_w, modal_h));

    // 1. 半透明暗色遮罩
    ui.painter().rect_filled(full_rect, 0.0, Color32::from_black_alpha(160));

    // 2. 面板背景与发光琥珀橙色边框
    ui.painter().rect_filled(modal_rect, CornerRadius::same(10), Theme::BG_PANEL_ALT);
    ui.painter().rect_stroke(
        modal_rect,
        CornerRadius::same(10),
        Stroke::new(1.5, Theme::ACCENT_ORANGE),
        egui::StrokeKind::Inside,
    );

    let mut fill_command = None;
    let mut close_modal = false;

    let query = state.history_search.trim().to_lowercase();
    let filtered_history: Vec<(usize, &String)> = state
        .command_history_list
        .iter()
        .enumerate()
        .filter(|(_, cmd)| {
            if query.is_empty() {
                true
            } else {
                cmd.to_lowercase().contains(&query)
            }
        })
        .collect();

    ui.scope_builder(UiBuilder::new().max_rect(modal_rect.shrink(18.0)), |ui| {
        ui.vertical(|ui| {
            // 顶栏：标题与关闭按钮
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("📜 VideoCut 历史命令记录 (:history)")
                        .size(16.0)
                        .strong()
                        .color(Theme::ACCENT_ORANGE),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!("(共 {} 条历史记录)", filtered_history.len()))
                        .size(12.0)
                        .color(Theme::TEXT_MUTED),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new(" ✕ 关闭 (Esc) ").size(12.0)).clicked() {
                        close_modal = true;
                    }
                });
            });

            ui.add_space(8.0);

            // 搜索栏
            ui.horizontal(|ui| {
                ui.label(RichText::new("🔍 筛选:").color(Theme::ACCENT_ORANGE).strong());
                let search_resp = ui.add(
                    egui::TextEdit::singleline(&mut state.history_search)
                        .hint_text("输入命令关键词筛选 (按 / 聚焦，按 Esc 退出)...")
                        .desired_width(ui.available_width() - 80.0),
                );
                if state.history_search_active {
                    search_resp.request_focus();
                    state.history_search_active = false;
                }
                if !state.history_search.is_empty() && ui.button("✕ 清空").clicked() {
                    state.history_search.clear();
                }
            });

            ui.add_space(4.0);
            ui.label(
                RichText::new("💡 快捷提示: 点击或按 Enter 填入命令进入编辑 | j/k 上下选择 | 按 / 搜索 | Esc 关闭")
                    .size(11.0)
                    .color(Theme::TEXT_MUTED),
            );
            ui.separator();
            ui.add_space(4.0);

            // 表头
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label(RichText::new("序号").strong().size(12.0).color(Theme::TEXT_MUTED));
                ui.add_space(30.0);
                ui.label(RichText::new("执行命令 (Command Line)").strong().size(12.0).color(Theme::ACCENT_ORANGE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(20.0);
                    ui.label(RichText::new("操作").strong().size(12.0).color(Theme::TEXT_MUTED));
                });
            });
            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if filtered_history.is_empty() {
                        ui.add_space(40.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("暂无历史命令记录")
                                    .size(14.0)
                                    .color(Theme::TEXT_MUTED),
                            );
                        });
                    } else {
                        for (filter_idx, (orig_idx, cmd)) in filtered_history.iter().enumerate() {
                            let is_selected = filter_idx == state.history_selected_idx;
                            let row_bg = if is_selected {
                                Color32::from_rgb(50, 38, 25)
                            } else if filter_idx % 2 == 0 {
                                Color32::from_rgb(22, 25, 32)
                            } else {
                                Color32::from_rgb(26, 30, 40)
                            };

                            let (rect, resp) = ui.allocate_exact_size(
                                vec2(ui.available_width(), 32.0),
                                egui::Sense::click(),
                            );

                            if resp.hovered() {
                                ui.painter().rect_filled(rect, CornerRadius::same(4), Color32::from_rgb(60, 48, 30));
                            } else {
                                ui.painter().rect_filled(rect, CornerRadius::same(4), row_bg);
                            }

                            if is_selected {
                                ui.painter().rect_stroke(
                                    rect,
                                    CornerRadius::same(4),
                                    Stroke::new(1.5, Theme::ACCENT_ORANGE),
                                    egui::StrokeKind::Inside,
                                );
                            }

                            let y_center = rect.center().y;

                            // 序号
                            ui.painter().text(
                                pos2(rect.min.x + 10.0, y_center),
                                egui::Align2::LEFT_CENTER,
                                format!("#{}", orig_idx + 1),
                                egui::FontId::monospace(11.0),
                                Theme::TEXT_MUTED,
                            );

                            // 命令行
                            let formatted_cmd = if cmd.starts_with(':') {
                                cmd.to_string()
                            } else {
                                format!(":{}", cmd)
                            };

                            ui.painter().text(
                                pos2(rect.min.x + 60.0, y_center),
                                egui::Align2::LEFT_CENTER,
                                &formatted_cmd,
                                egui::FontId::monospace(13.0),
                                Theme::TEXT_PRIMARY,
                            );

                            // 复用按钮
                            let btn_rect = Rect::from_center_size(
                                pos2(rect.max.x - 50.0, y_center),
                                vec2(60.0, 20.0),
                            );
                            ui.painter().rect_filled(
                                btn_rect,
                                CornerRadius::same(3),
                                Color32::from_rgb(45, 35, 20),
                            );
                            ui.painter().text(
                                btn_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "复用 ↵",
                                egui::FontId::proportional(11.0),
                                Theme::ACCENT_ORANGE,
                            );

                            if resp.clicked() {
                                fill_command = Some(formatted_cmd);
                            }
                        }
                    }
                });
        });
    });

    if let Some(cmd) = fill_command {
        state.command_input = cmd;
        state.is_command_mode = true;
        close_modal = true;
    }

    if close_modal {
        state.show_history_modal = false;
    }
}

/// 绘制独立视频渲染导出交互弹窗 (:export)
fn draw_export_modal(ui: &mut Ui, project: &ProjectState, state: &mut MainInterfaceUiState) {
    let full_rect = ui.max_rect();
    let modal_w = 640.0;
    let modal_h = 420.0;
    let modal_rect = Rect::from_center_size(full_rect.center(), vec2(modal_w, modal_h));

    // 1. 半透明暗色遮罩
    ui.painter().rect_filled(full_rect, 0.0, Color32::from_black_alpha(160));

    // 2. 面板背景与发光青色边框
    ui.painter().rect_filled(modal_rect, CornerRadius::same(10), Theme::BG_PANEL_ALT);
    ui.painter().rect_stroke(
        modal_rect,
        CornerRadius::same(10),
        Stroke::new(1.5, Theme::ACCENT_CYAN),
        egui::StrokeKind::Inside,
    );

    let mut do_close = false;
    let mut do_start_export = false;

    ui.scope_builder(UiBuilder::new().max_rect(modal_rect.shrink(20.0)), |ui| {
        ui.vertical(|ui| {
            // 顶栏：标题与关闭按钮
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("🎬 视频渲染与极速拼接导出 (:export)")
                        .size(16.0)
                        .strong()
                        .color(Theme::ACCENT_CYAN),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new(" ✕ 关闭 (Esc) ").size(12.0)).clicked() {
                        do_close = true;
                    }
                });
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(8.0);

            // 输出路径
            ui.label(RichText::new("📁 输出文件路径:").strong().color(Theme::TEXT_PRIMARY));
            ui.add(
                egui::TextEdit::singleline(&mut state.export_state.output_path)
                    .hint_text("例如: output/final_render.mp4")
                    .desired_width(ui.available_width()),
            );

            ui.add_space(10.0);
            // 编码预设选择
            ui.label(RichText::new("⚙ 编码预设 (Preset):").strong().color(Theme::TEXT_PRIMARY));
            ui.horizontal(|ui| {
                let presets = [
                    (crate::rendering::smart_export::ExportPreset::H264Mp4, "H.264 (Web兼容)"),
                    (crate::rendering::smart_export::ExportPreset::HevcMp4, "H.265 (HEVC高压缩)"),
                    (crate::rendering::smart_export::ExportPreset::ProResMov, "ProRes 422 (母带级)"),
                ];

                for (p, label) in presets {
                    let is_active = state.export_state.preset == p;
                    let btn_text = RichText::new(label).color(if is_active { Theme::ACCENT_CYAN } else { Theme::TEXT_SECONDARY });
                    if ui.selectable_label(is_active, btn_text).clicked() {
                        state.export_state.preset = p;
                        let ext = p.container_extension();
                        if !state.export_state.output_path.ends_with(ext) {
                            if let Some(stem) = std::path::Path::new(&state.export_state.output_path).file_stem() {
                                state.export_state.output_path = format!("{}.{}", stem.to_string_lossy(), ext);
                            }
                        }
                    }
                }
            });

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            // 导出进度展示
            if state.export_state.is_exporting {
                ui.label(
                    RichText::new(format!(
                        "⚡ 正在渲染导出... {:.1}% (帧数: {}/{}, 速率: {:.0} fps, ETA: {}s)",
                        state.export_state.progress * 100.0,
                        state.export_state.current_frame,
                        state.export_state.total_frames,
                        state.export_state.fps,
                        state.export_state.eta_seconds
                    ))
                    .color(Theme::ACCENT_ORANGE)
                    .strong(),
                );
                ui.add_space(6.0);
                ui.add(
                    egui::ProgressBar::new(state.export_state.progress)
                        .show_percentage()
                        .animate(true),
                );
            } else if state.export_state.is_completed {
                ui.label(
                    RichText::new(format!("🎉 导出完成！文件已保存至: {}", state.export_state.output_path))
                        .color(Theme::ACCENT_GREEN)
                        .strong(),
                );
            } else {
                let dur = project.timeline.duration;
                ui.label(
                    RichText::new(format!(
                        "总时长: {} | 预计帧数: {} 帧 @ 60fps | 智能分片预渲染秒级拼接已就绪",
                        format_timecode(dur.0),
                        (dur.0 as f64 / 1_000_000.0 * 60.0) as u64
                    ))
                    .size(11.5)
                    .color(Theme::TEXT_MUTED),
                );
            }

            ui.add_space(16.0);
            ui.horizontal(|ui| {
                if !state.export_state.is_exporting {
                    if ui
                        .button(
                            RichText::new(" 🚀 开始极速渲染导出 ")
                                .size(13.0)
                                .color(Theme::ACCENT_CYAN)
                                .strong(),
                        )
                        .clicked()
                    {
                        do_start_export = true;
                    }
                } else if ui.button(RichText::new(" ⏹ 取消导出 ").size(12.0).color(Theme::TEXT_MUTED)).clicked() {
                    state.export_state.is_exporting = false;
                    state.status_message = Some("已取消导出任务".into());
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new(" 关闭 (Esc) ").size(12.0)).clicked() {
                        do_close = true;
                    }
                });
            });
        });
    });

    if do_start_export {
        state.export_state.is_exporting = true;
        state.export_state.progress = 0.0;
        state.export_state.current_frame = 0;
        let total_frames = ((project.timeline.duration.0 as f64 / 1_000_000.0) * 60.0).max(60.0) as u64;
        state.export_state.total_frames = total_frames;
        state.export_state.fps = 145.0;
        state.export_state.eta_seconds = (total_frames as f32 / 145.0) as u32;
        state.export_state.is_completed = false;
    }

    if do_close {
        state.show_export_modal = false;
    }
}

/// 绘制 12. 贝塞尔缓动曲线可视化编辑器弹窗
fn draw_easing_modal(
    ui: &mut Ui,
    project: &mut ProjectState,
    state: &mut MainInterfaceUiState,
) {
    let full_rect = ui.max_rect();
    let modal_width = 540.0;
    let modal_height = 430.0;
    let modal_rect = Rect::from_center_size(
        full_rect.center(),
        vec2(modal_width, modal_height),
    );

    // 绘制暗色半透明背景遮罩
    ui.painter().rect_filled(
        full_rect,
        0.0,
        Color32::from_rgba_unmultiplied(0, 0, 0, 175),
    );

    // 绘制弹窗背景与边框
    ui.painter().rect_filled(modal_rect, CornerRadius::same(10), Theme::BG_PANEL);
    ui.painter().rect_stroke(
        modal_rect,
        CornerRadius::same(10),
        Stroke::new(1.5, Theme::ACCENT_CYAN),
        egui::StrokeKind::Inside,
    );

    let mut do_close = false;
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        do_close = true;
    }

    let track_idx = state.selected_track_idx;
    let playhead = FrameTime(state.playhead_us);

    // 获取当前切片的可变引用
    let current_clip = project
        .timeline
        .tracks
        .get_mut(track_idx)
        .and_then(|t| t.clips.iter_mut().find(|c| playhead >= c.timeline_start && playhead <= c.timeline_end()));

    ui.scope_builder(UiBuilder::new().max_rect(modal_rect), |ui| {
        ui.vertical(|ui| {
            ui.add_space(14.0);

            // 标题栏
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    RichText::new("📈 关键帧贝塞尔缓动曲线编辑器")
                        .size(16.0)
                        .strong()
                        .color(Theme::ACCENT_CYAN),
                );
                ui.label(
                    RichText::new("(Cubic Bezier Visualizer)")
                        .size(11.0)
                        .color(Theme::TEXT_MUTED),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(14.0);
                    if ui.button(RichText::new(" ✕ ").size(14.0).color(Theme::TEXT_MUTED)).clicked() {
                        do_close = true;
                    }
                });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(10.0);

            if let Some(clip) = current_clip {
                let mut current_curve = clip.easing_curve;

                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    ui.label(RichText::new(format!("当前切片: {}", clip.name)).size(13.0).color(Theme::TEXT_PRIMARY).strong());
                    ui.add_space(12.0);
                    ui.label(RichText::new(format!("当前缓动: {}", current_curve.name())).size(12.0).color(Theme::ACCENT_ORANGE));
                });

                ui.add_space(10.0);

                // 中间主区域：左侧 230x230 交互画布，右侧 预设选择与控制点数值
                ui.horizontal(|ui| {
                    ui.add_space(20.0);

                    // 1. 230x230 曲线画布
                    let canvas_size = 230.0;
                    let (canvas_rect, response) = ui.allocate_exact_size(vec2(canvas_size, canvas_size), egui::Sense::click_and_drag());
                    let painter = ui.painter_at(canvas_rect);

                    // 画布背景与网格
                    painter.rect_filled(canvas_rect, CornerRadius::same(6), Color32::from_rgb(18, 20, 24));
                    painter.rect_stroke(canvas_rect, CornerRadius::same(6), Stroke::new(1.0, Theme::BORDER_MEDIUM), egui::StrokeKind::Inside);

                    // 辅助参考网格 (0.25, 0.5, 0.75)
                    for step in 1..4 {
                        let frac = step as f32 * 0.25;
                        let gx = canvas_rect.min.x + frac * canvas_size;
                        let gy = canvas_rect.max.y - frac * canvas_size;
                        painter.line_segment([pos2(gx, canvas_rect.min.y), pos2(gx, canvas_rect.max.y)], Stroke::new(0.5, Color32::from_rgb(35, 38, 45)));
                        painter.line_segment([pos2(canvas_rect.min.x, gy), pos2(canvas_rect.max.x, gy)], Stroke::new(0.5, Color32::from_rgb(35, 38, 45)));
                    }

                    // 对角线性参考线 (0,0) -> (1,1)
                    painter.line_segment([pos2(canvas_rect.min.x, canvas_rect.max.y), pos2(canvas_rect.max.x, canvas_rect.min.y)], Stroke::new(1.0, Color32::from_rgb(45, 50, 60)));

                    // 采样并绘制缓动曲线 (100 段)
                    let samples = 100;
                    let mut prev_pt = pos2(canvas_rect.min.x, canvas_rect.max.y);
                    for i in 1..=samples {
                        let t = i as f32 / samples as f32;
                        let val = current_curve.evaluate(t);
                        let px = canvas_rect.min.x + t * canvas_size;
                        let py = canvas_rect.max.y - val * canvas_size;
                        let cur_pt = pos2(px, py);
                        painter.line_segment([prev_pt, cur_pt], Stroke::new(2.5, Theme::ACCENT_CYAN));
                        prev_pt = cur_pt;
                    }

                    // 绘制控制点手柄 P1, P2 (如果是 CubicBezier 或标准预设)
                    let p1_screen = pos2(canvas_rect.min.x + current_curve.p1[0] * canvas_size, canvas_rect.max.y - current_curve.p1[1] * canvas_size);
                    let p2_screen = pos2(canvas_rect.min.x + current_curve.p2[0] * canvas_size, canvas_rect.max.y - current_curve.p2[1] * canvas_size);

                    // 控制线
                    painter.line_segment([pos2(canvas_rect.min.x, canvas_rect.max.y), p1_screen], Stroke::new(1.5, Color32::from_rgb(255, 140, 0)));
                    painter.line_segment([pos2(canvas_rect.max.x, canvas_rect.min.y), p2_screen], Stroke::new(1.5, Color32::from_rgb(180, 80, 255)));

                    // 控制柄端点
                    painter.circle_filled(p1_screen, 6.0, Color32::from_rgb(255, 140, 0));
                    painter.circle_filled(p2_screen, 6.0, Color32::from_rgb(180, 80, 255));

                    // 鼠标拖拽控制点交互
                    if response.dragged() {
                        if let Some(m_pos) = response.interact_pointer_pos() {
                            let rel_x = ((m_pos.x - canvas_rect.min.x) / canvas_size).clamp(0.0, 1.0);
                            let rel_y = ((canvas_rect.max.y - m_pos.y) / canvas_size).clamp(-0.5, 1.5);

                            let dist_p1 = m_pos.distance(p1_screen);
                            let dist_p2 = m_pos.distance(p2_screen);

                            if dist_p1 < dist_p2 {
                                current_curve.p1 = [rel_x, rel_y];
                                current_curve.easing_type = crate::effects::EasingType::CubicBezier;
                            } else {
                                current_curve.p2 = [rel_x, rel_y];
                                current_curve.easing_type = crate::effects::EasingType::CubicBezier;
                            }
                            clip.easing_curve = current_curve;
                        }
                    }

                    // 实时动画物理运动预览 (Bouncing Ball Preview)
                    let time = ui.input(|i| i.time as f32);
                    let cycle = (time % 2.0) / 2.0; // 0.0 ~ 1.0
                    let pingpong = if cycle < 0.5 { cycle * 2.0 } else { 2.0 - cycle * 2.0 };
                    let anim_val = current_curve.evaluate(pingpong);
                    let ball_x = canvas_rect.min.x + pingpong * canvas_size;
                    let ball_y = canvas_rect.max.y - anim_val * canvas_size;
                    painter.circle_filled(pos2(ball_x, ball_y), 5.0, Color32::WHITE);

                    ui.add_space(20.0);

                    // 2. 右侧预设按钮与参数
                    ui.vertical(|ui| {
                        ui.label(RichText::new("标准缓动预设:").strong().color(Theme::TEXT_PRIMARY));
                        ui.add_space(6.0);

                        if ui.selectable_label(current_curve.easing_type == crate::effects::EasingType::Linear, "匀速直线 (Linear)").clicked() {
                            clip.easing_curve = crate::effects::EasingCurve::linear();
                        }
                        if ui.selectable_label(current_curve.easing_type == crate::effects::EasingType::EaseIn, "平滑加速 (Ease In)").clicked() {
                            clip.easing_curve = crate::effects::EasingCurve::ease_in();
                        }
                        if ui.selectable_label(current_curve.easing_type == crate::effects::EasingType::EaseOut, "平滑减速 (Ease Out)").clicked() {
                            clip.easing_curve = crate::effects::EasingCurve::ease_out();
                        }
                        if ui.selectable_label(current_curve.easing_type == crate::effects::EasingType::EaseInOut, "平滑缓入缓出 (Ease In-Out)").clicked() {
                            clip.easing_curve = crate::effects::EasingCurve::ease_in_out();
                        }
                        if ui.selectable_label(current_curve.easing_type == crate::effects::EasingType::BounceOut, "弹力弹跳 (Bounce)").clicked() {
                            clip.easing_curve = crate::effects::EasingCurve::bounce_out();
                        }
                        if ui.selectable_label(current_curve.easing_type == crate::effects::EasingType::ElasticOut, "弹性阻尼 (Elastic)").clicked() {
                            clip.easing_curve = crate::effects::EasingCurve::elastic_out();
                        }

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(6.0);

                        ui.label(RichText::new(format!("P1: [{:.2}, {:.2}]", current_curve.p1[0], current_curve.p1[1])).size(11.0).color(Color32::from_rgb(255, 140, 0)));
                        ui.label(RichText::new(format!("P2: [{:.2}, {:.2}]", current_curve.p2[0], current_curve.p2[1])).size(11.0).color(Color32::from_rgb(180, 80, 255)));
                    });
                });
            } else {
                ui.add_space(30.0);
                ui.label(RichText::new("当前播放头下未选中任何切片").size(14.0).color(Theme::TEXT_MUTED));
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            // 底部操作栏
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                ui.label(RichText::new("提示: 可在画布中直接拖拽橙/紫色手柄调节贝塞尔曲线").size(11.0).color(Theme::TEXT_MUTED));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(16.0);
                    if ui.button(RichText::new(" 完成 (Esc) ").size(12.0)).clicked() {
                        do_close = true;
                    }
                });
            });
        });
    });

    if do_close {
        state.show_easing_modal = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::{AnchorPoint, AssetId, Clip, ClipId, FrameTime, Track, TrackId};

    #[test]
    fn test_command_help_items_and_search_filter() {
        let items = get_all_command_help_items();
        assert!(items.len() >= 15);

        // 搜索 "split"
        let split_results: Vec<&CommandHelpItem> = items
            .iter()
            .filter(|i| i.name.contains("split") || i.description.contains("split"))
            .collect();
        assert_eq!(split_results.len(), 1);
        assert_eq!(split_results[0].name, ":split");
        assert_eq!(split_results[0].alias, ":s");

        // 搜索 "lua"
        let lua_results: Vec<&CommandHelpItem> = items
            .iter()
            .filter(|i| i.name.contains("lua") || i.description.to_lowercase().contains("lua"))
            .collect();
        assert!(lua_results.iter().any(|i| i.name == ":export_lua"));
        assert!(lua_results.iter().any(|i| i.name == ":editor"));
    }

    #[test]
    fn test_collect_all_anchors_global_and_local() {
        let mut project = ProjectState::new("Test Anchors Project");
        
        // 1. 全局锚点
        project.timeline.global_anchors.set_anchor(
            AnchorPoint::new("Intro", FrameTime(2_000_000), AnchorScope::Global)
                .with_description("视频开场"),
        );

        // 2. 切片局部锚点
        let track_id = TrackId(1);
        let mut track = Track::new(track_id, "Video 1");
        let mut clip = Clip::new(
            ClipId(101),
            "Camera_A.mp4".into(),
            AssetId(1),
            FrameTime(5_000_000),
            FrameTime(10_000_000),
        );
        clip.anchors.insert(
            "Highlight".into(),
            AnchorPoint::new("Highlight", FrameTime(3_000_000), AnchorScope::Local)
                .with_description("高光时刻"),
        );
        track.add_clip(clip);
        project.timeline.tracks.push(track);

        let all_anchors = collect_all_anchors(&project);
        assert_eq!(all_anchors.len(), 2);

        let global_item = all_anchors.iter().find(|a| a.name == "Intro").unwrap();
        assert_eq!(global_item.position_us, 2_000_000);
        assert!(global_item.is_global);
        assert_eq!(global_item.scope_label, "[全局]");

        let local_item = all_anchors.iter().find(|a| a.name == "Highlight").unwrap();
        // 局部锚点相对 5s + 3s = 8s
        assert_eq!(local_item.position_us, 8_000_000);
        assert!(!local_item.is_global);
        assert_eq!(local_item.scope_label, "[切片: Camera_A.mp4]");
    }

    #[test]
    fn test_anchor_sessions_default_state() {
        let mark_session = AnchorMarkSession {
            scope: AnchorScope::Global,
            is_multichar: true,
            input_buffer: "Chorus_Start".into(),
        };
        assert!(mark_session.is_multichar);
        assert_eq!(mark_session.input_buffer, "Chorus_Start");

        let jump_session = AnchorJumpSession {
            query: "intro".into(),
            selected_idx: 0,
            is_multichar: true,
        };
        assert_eq!(jump_session.query, "intro");
        assert_eq!(jump_session.selected_idx, 0);
    }
}
