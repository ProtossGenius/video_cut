use serde::{Deserialize, Serialize};

/// 文本排版对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TextAlignment {
    /// 底部居中（最经典电影字幕位置）
    #[default]
    BottomCenter,
    /// 顶部居中（片头/注释位置）
    TopCenter,
    /// 画面正中央（强调词/标题位置）
    Center,
    /// 左下角（发言人名称/台标位置）
    BottomLeft,
    /// 右下角
    BottomRight,
}

/// 文本与字幕覆盖参数
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextOverlayParams {
    /// 文本内容
    pub content: String,
    /// 字体字号 (像素，默认 24.0)
    pub font_size: f32,
    /// 文字主色 [R, G, B, A] (默认纯白)
    pub color_rgba: [u8; 4],
    /// 描边粗细 (默认 1.5)
    pub outline_width: f32,
    /// 描边颜色 [R, G, B, A] (默认纯黑半透明)
    pub outline_rgba: [u8; 4],
    /// 是否开启背景气泡框
    pub has_background: bool,
    /// 背景气泡颜色 [R, G, B, A] (默认半透明黑)
    pub bg_rgba: [u8; 4],
    /// 排版对齐位置
    pub alignment: TextAlignment,
}

impl Default for TextOverlayParams {
    fn default() -> Self {
        Self {
            content: String::new(),
            font_size: 24.0,
            color_rgba: [255, 255, 255, 255],
            outline_width: 1.5,
            outline_rgba: [0, 0, 0, 220],
            has_background: true,
            bg_rgba: [15, 15, 20, 190],
            alignment: TextAlignment::BottomCenter,
        }
    }
}

impl TextOverlayParams {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            ..Default::default()
        }
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size.clamp(8.0, 120.0);
        self
    }

    pub fn with_color(mut self, rgba: [u8; 4]) -> Self {
        self.color_rgba = rgba;
        self
    }

    pub fn with_background(mut self, has_bg: bool) -> Self {
        self.has_background = has_bg;
        self
    }

    pub fn with_alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.content.trim().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_overlay_default_and_builder() {
        let text = TextOverlayParams::new("字幕测试")
            .with_font_size(32.0)
            .with_color([255, 255, 0, 255])
            .with_alignment(TextAlignment::Center);

        assert_eq!(text.content, "字幕测试");
        assert_eq!(text.font_size, 32.0);
        assert_eq!(text.color_rgba, [255, 255, 0, 255]);
        assert_eq!(text.alignment, TextAlignment::Center);
        assert!(!text.is_empty());
    }

    #[test]
    fn test_text_overlay_empty_checking() {
        let text = TextOverlayParams::new("   ");
        assert!(text.is_empty());
    }
}
