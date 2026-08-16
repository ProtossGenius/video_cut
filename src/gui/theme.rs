use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, Stroke};

pub struct Theme;

impl Theme {
    // 基础背景色调 (Dark Obsidian / Carbon)
    pub const BG_APP: Color32 = Color32::from_rgb(14, 15, 18);
    pub const BG_PANEL: Color32 = Color32::from_rgb(20, 21, 26);
    pub const BG_PANEL_ALT: Color32 = Color32::from_rgb(25, 27, 33);
    pub const BG_CARD: Color32 = Color32::from_rgb(30, 33, 40);
    pub const BG_CARD_HOVER: Color32 = Color32::from_rgb(38, 42, 52);
    pub const BG_CARD_ACTIVE: Color32 = Color32::from_rgb(45, 52, 66);
    pub const BG_INPUT: Color32 = Color32::from_rgb(16, 17, 21);

    // 边框与分割线
    pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(40, 43, 53);
    pub const BORDER_MEDIUM: Color32 = Color32::from_rgb(55, 60, 75);
    pub const BORDER_FOCUS: Color32 = Color32::from_rgb(74, 158, 255);

    // 强调色 (Accents)
    pub const ACCENT_BLUE: Color32 = Color32::from_rgb(56, 139, 253); // #388bfd
    pub const ACCENT_CYAN: Color32 = Color32::from_rgb(0, 210, 255); // #00d2ff
    pub const ACCENT_ORANGE: Color32 = Color32::from_rgb(240, 136, 62); // #f0883e
    pub const ACCENT_GREEN: Color32 = Color32::from_rgb(63, 185, 80); // #3fb950
    pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(187, 128, 247); // #bb80f7
    pub const ACCENT_RED: Color32 = Color32::from_rgb(248, 81, 73); // #f85149
    pub const ACCENT_YELLOW: Color32 = Color32::from_rgb(210, 153, 34); // #d29922

    // 轨道与剪辑块专用颜色
    pub const TRACK_V1: Color32 = Color32::from_rgb(35, 75, 120);
    pub const TRACK_V1_BORDER: Color32 = Color32::from_rgb(60, 140, 230);
    pub const TRACK_V2: Color32 = Color32::from_rgb(85, 55, 120);
    pub const TRACK_V2_BORDER: Color32 = Color32::from_rgb(150, 100, 220);
    pub const TRACK_A1: Color32 = Color32::from_rgb(20, 90, 80);
    pub const TRACK_A1_BORDER: Color32 = Color32::from_rgb(40, 170, 150);
    pub const TRACK_A2: Color32 = Color32::from_rgb(90, 70, 20);
    pub const TRACK_A2_BORDER: Color32 = Color32::from_rgb(180, 140, 40);
    pub const TRACK_TRASH: Color32 = Color32::from_rgb(35, 36, 40);
    pub const TRACK_TRASH_BORDER: Color32 = Color32::from_rgb(65, 68, 76);

    // 文字颜色
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(240, 243, 246);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(160, 168, 178);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(105, 112, 122);
    pub const TEXT_ACCENT: Color32 = Color32::from_rgb(88, 166, 255);

    /// 应用全局 Visuals 主题
    pub fn apply_visuals(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();

        visuals.override_text_color = Some(Self::TEXT_PRIMARY);
        visuals.panel_fill = Self::BG_PANEL;
        visuals.window_fill = Self::BG_PANEL_ALT;
        visuals.faint_bg_color = Self::BG_APP;
        visuals.extreme_bg_color = Self::BG_INPUT;

        // 窗口圆角与阴影
        visuals.window_corner_radius = CornerRadius::same(8);
        visuals.menu_corner_radius = CornerRadius::same(6);

        // 控件样式
        visuals.widgets.noninteractive.bg_fill = Self::BG_CARD;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Self::BORDER_SUBTLE);
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(4);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);

        visuals.widgets.inactive.bg_fill = Self::BG_CARD;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Self::BORDER_SUBTLE);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(4);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);

        visuals.widgets.hovered.bg_fill = Self::BG_CARD_HOVER;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Self::ACCENT_BLUE);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(4);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);

        visuals.widgets.active.bg_fill = Self::BG_CARD_ACTIVE;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5, Self::ACCENT_CYAN);
        visuals.widgets.active.corner_radius = CornerRadius::same(4);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);

        visuals.selection.bg_fill = Self::ACCENT_BLUE;
        visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);

        ctx.set_visuals(visuals);
    }

    /// 卡片通用 Frame
    pub fn card_frame(is_selected: bool, is_hovered: bool) -> Frame {
        let bg = if is_selected {
            Self::BG_CARD_ACTIVE
        } else if is_hovered {
            Self::BG_CARD_HOVER
        } else {
            Self::BG_CARD
        };

        let border_color = if is_selected {
            Self::ACCENT_BLUE
        } else if is_hovered {
            Self::BORDER_MEDIUM
        } else {
            Self::BORDER_SUBTLE
        };

        let stroke_width = if is_selected { 1.5 } else { 1.0 };

        Frame::new()
            .fill(bg)
            .corner_radius(CornerRadius::same(6))
            .stroke(Stroke::new(stroke_width, border_color))
            .inner_margin(Margin::same(10))
    }

    /// 模态面板 / 浮窗 Frame
    pub fn modal_frame() -> Frame {
        Frame::new()
            .fill(Color32::from_rgba_premultiplied(20, 22, 28, 245))
            .corner_radius(CornerRadius::same(8))
            .stroke(Stroke::new(1.5, Self::BORDER_MEDIUM))
            .inner_margin(Margin::same(14))
    }
}
