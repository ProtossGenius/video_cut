use serde::{Deserialize, Serialize};
use crate::timeline::Clip;

/// 画中画与分屏排布预设类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipLayoutPreset {
    /// 右下角画中画浮窗
    CornerBottomRight,
    /// 右上角画中画浮窗
    CornerTopRight,
    /// 左下角画中画浮窗
    CornerBottomLeft,
    /// 左上角画中画浮窗
    CornerTopLeft,
    /// 居中悬浮画中画
    CenterFloating,
    /// 水平左分屏
    SplitLeft,
    /// 水平右分屏
    SplitRight,
    /// 垂直上分屏
    SplitTop,
    /// 垂直下分屏
    SplitBottom,
    /// 四宫格左上
    GridTopLeft,
    /// 四宫格右上
    GridTopRight,
    /// 四宫格左下
    GridBottomLeft,
    /// 四宫格右下
    GridBottomRight,
    /// 重置充满全屏
    FullscreenReset,
}

impl Default for PipLayoutPreset {
    fn default() -> Self {
        Self::CornerBottomRight
    }
}

impl PipLayoutPreset {
    pub fn name(&self) -> &'static str {
        match self {
            PipLayoutPreset::CornerBottomRight => "右下角画中画 (Bottom-Right PIP)",
            PipLayoutPreset::CornerTopRight => "右上角画中画 (Top-Right PIP)",
            PipLayoutPreset::CornerBottomLeft => "左下角画中画 (Bottom-Left PIP)",
            PipLayoutPreset::CornerTopLeft => "左上角画中画 (Top-Left PIP)",
            PipLayoutPreset::CenterFloating => "居中悬浮画中画 (Center Floating PIP)",
            PipLayoutPreset::SplitLeft => "左半屏 (Split Left)",
            PipLayoutPreset::SplitRight => "右半屏 (Split Right)",
            PipLayoutPreset::SplitTop => "上半屏 (Split Top)",
            PipLayoutPreset::SplitBottom => "下半屏 (Split Bottom)",
            PipLayoutPreset::GridTopLeft => "四宫格左上 (Quad Top-Left)",
            PipLayoutPreset::GridTopRight => "四宫格右上 (Quad Top-Right)",
            PipLayoutPreset::GridBottomLeft => "四宫格左下 (Quad Bottom-Left)",
            PipLayoutPreset::GridBottomRight => "四宫格右下 (Quad Bottom-Right)",
            PipLayoutPreset::FullscreenReset => "全屏充满 (Fullscreen Reset)",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().replace(['_', '-'], "").as_str() {
            "cornerbr" | "br" | "corner" | "bottomright" => Some(Self::CornerBottomRight),
            "cornertr" | "tr" | "topright" => Some(Self::CornerTopRight),
            "cornerbl" | "bl" | "bottomleft" => Some(Self::CornerBottomLeft),
            "cornertl" | "tl" | "topleft" => Some(Self::CornerTopLeft),
            "center" | "floating" | "float" => Some(Self::CenterFloating),
            "splitleft" | "left" | "splithleft" => Some(Self::SplitLeft),
            "splitright" | "right" | "splithright" => Some(Self::SplitRight),
            "splittop" | "top" | "splitvtop" => Some(Self::SplitTop),
            "splitbottom" | "bottom" | "splitvbottom" => Some(Self::SplitBottom),
            "gridtl" | "quadtl" | "grid1" => Some(Self::GridTopLeft),
            "gridtr" | "quadtr" | "grid2" => Some(Self::GridTopRight),
            "gridbl" | "quadbl" | "grid3" => Some(Self::GridBottomLeft),
            "gridbr" | "quadbr" | "grid4" => Some(Self::GridBottomRight),
            "reset" | "full" | "fullscreen" | "none" => Some(Self::FullscreenReset),
            _ => None,
        }
    }

    /// 将画中画变换参数（缩放与基于视口中心像素的位移）应用到切片上
    pub fn apply_to_clip(&self, clip: &mut Clip, viewport_w: f32, viewport_h: f32) {
        let r = |v: f32| (v * 100.0).round() / 100.0;
        match self {
            PipLayoutPreset::CornerBottomRight => {
                clip.transform_scale = [0.35, 0.35];
                clip.transform_offset = [r(viewport_w * 0.30), r(viewport_h * 0.30)];
            }
            PipLayoutPreset::CornerTopRight => {
                clip.transform_scale = [0.35, 0.35];
                clip.transform_offset = [r(viewport_w * 0.30), r(-viewport_h * 0.30)];
            }
            PipLayoutPreset::CornerBottomLeft => {
                clip.transform_scale = [0.35, 0.35];
                clip.transform_offset = [r(-viewport_w * 0.30), r(viewport_h * 0.30)];
            }
            PipLayoutPreset::CornerTopLeft => {
                clip.transform_scale = [0.35, 0.35];
                clip.transform_offset = [r(-viewport_w * 0.30), r(-viewport_h * 0.30)];
            }
            PipLayoutPreset::CenterFloating => {
                clip.transform_scale = [0.5, 0.5];
                clip.transform_offset = [0.0, 0.0];
            }
            PipLayoutPreset::SplitLeft => {
                clip.transform_scale = [0.5, 0.5];
                clip.transform_offset = [r(-viewport_w * 0.25), 0.0];
            }
            PipLayoutPreset::SplitRight => {
                clip.transform_scale = [0.5, 0.5];
                clip.transform_offset = [r(viewport_w * 0.25), 0.0];
            }
            PipLayoutPreset::SplitTop => {
                clip.transform_scale = [0.5, 0.5];
                clip.transform_offset = [0.0, r(-viewport_h * 0.25)];
            }
            PipLayoutPreset::SplitBottom => {
                clip.transform_scale = [0.5, 0.5];
                clip.transform_offset = [0.0, r(viewport_h * 0.25)];
            }
            PipLayoutPreset::GridTopLeft => {
                clip.transform_scale = [0.5, 0.5];
                clip.transform_offset = [r(-viewport_w * 0.25), r(-viewport_h * 0.25)];
            }
            PipLayoutPreset::GridTopRight => {
                clip.transform_scale = [0.5, 0.5];
                clip.transform_offset = [r(viewport_w * 0.25), r(-viewport_h * 0.25)];
            }
            PipLayoutPreset::GridBottomLeft => {
                clip.transform_scale = [0.5, 0.5];
                clip.transform_offset = [r(-viewport_w * 0.25), r(viewport_h * 0.25)];
            }
            PipLayoutPreset::GridBottomRight => {
                clip.transform_scale = [0.5, 0.5];
                clip.transform_offset = [r(viewport_w * 0.25), r(viewport_h * 0.25)];
            }
            PipLayoutPreset::FullscreenReset => {
                clip.transform_scale = [1.0, 1.0];
                clip.transform_offset = [0.0, 0.0];
                clip.transform_rotation_deg = 0.0;
                clip.transform_flip_h = false;
                clip.transform_flip_v = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::{AssetId, ClipId, FrameTime};

    #[test]
    fn test_pip_preset_from_str_loose() {
        assert_eq!(PipLayoutPreset::from_str_loose("corner_br"), Some(PipLayoutPreset::CornerBottomRight));
        assert_eq!(PipLayoutPreset::from_str_loose("top_left"), Some(PipLayoutPreset::CornerTopLeft));
        assert_eq!(PipLayoutPreset::from_str_loose("split_left"), Some(PipLayoutPreset::SplitLeft));
        assert_eq!(PipLayoutPreset::from_str_loose("grid_tr"), Some(PipLayoutPreset::GridTopRight));
        assert_eq!(PipLayoutPreset::from_str_loose("reset"), Some(PipLayoutPreset::FullscreenReset));
        assert_eq!(PipLayoutPreset::from_str_loose("unknown"), None);
    }

    #[test]
    fn test_pip_apply_transforms() {
        let mut clip = Clip::new(ClipId(1), "test.mp4".into(), AssetId(1), FrameTime(0), FrameTime(5_000_000));
        let w = 800.0;
        let h = 450.0;

        PipLayoutPreset::CornerBottomRight.apply_to_clip(&mut clip, w, h);
        assert_eq!(clip.transform_scale, [0.35, 0.35]);
        assert_eq!(clip.transform_offset, [240.0, 135.0]);

        PipLayoutPreset::FullscreenReset.apply_to_clip(&mut clip, w, h);
        assert_eq!(clip.transform_scale, [1.0, 1.0]);
        assert_eq!(clip.transform_offset, [0.0, 0.0]);
    }
}
