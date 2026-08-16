use serde::{Deserialize, Serialize};
use crate::timeline::FrameTime;

/// 视频转场类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionType {
    /// 交叉溶解 (前后素材平滑混叠)
    CrossDissolve,
    /// 自右向左划像 (Push/Wipe Left)
    WipeLeft,
    /// 自左向右划像 (Push/Wipe Right)
    WipeRight,
    /// 闪黑 / 黑场过渡 (Dip to Black)
    DipToBlack,
    /// 闪白 / 过曝过渡 (Dip to White)
    DipToWhite,
}

impl TransitionType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::CrossDissolve => "交叉溶解 (Cross Dissolve)",
            Self::WipeLeft => "左向划像 (Wipe Left)",
            Self::WipeRight => "右向划像 (Wipe Right)",
            Self::DipToBlack => "黑场过渡 (Dip to Black)",
            Self::DipToWhite => "白场过渡 (Dip to White)",
        }
    }

    pub fn short_code(&self) -> &'static str {
        match self {
            Self::CrossDissolve => "Dissolve",
            Self::WipeLeft => "WipeLeft",
            Self::WipeRight => "WipeRight",
            Self::DipToBlack => "DipBlack",
            Self::DipToWhite => "DipWhite",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().replace(['_', '-'], "").as_str() {
            "crossdissolve" | "dissolve" | "cross" | "fade" => Some(Self::CrossDissolve),
            "wipeleft" | "wipel" | "left" => Some(Self::WipeLeft),
            "wiperight" | "wiper" | "right" => Some(Self::WipeRight),
            "diptoblack" | "dipblack" | "black" => Some(Self::DipToBlack),
            "diptowhite" | "dipwhite" | "white" | "flash" => Some(Self::DipToWhite),
            _ => None,
        }
    }
}

/// 转场对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionAlignment {
    /// 居中对齐接缝
    Center,
    /// 从切点开始向后
    StartOnCut,
    /// 从切点向前结束
    EndOnCut,
}

/// 视频转场实例配置
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    pub transition_type: TransitionType,
    /// 转场持续时间 (微秒，默认 1_000_000 = 1.0s)
    pub duration: FrameTime,
    /// 对齐方式
    pub alignment: TransitionAlignment,
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            transition_type: TransitionType::CrossDissolve,
            duration: FrameTime(1_000_000),
            alignment: TransitionAlignment::Center,
        }
    }
}

impl Transition {
    pub fn new(transition_type: TransitionType, duration_secs: f64) -> Self {
        Self {
            transition_type,
            duration: FrameTime::from_seconds(duration_secs.max(0.05)),
            alignment: TransitionAlignment::Center,
        }
    }

    /// 计算给定相对接缝时间的转场进度 (0.0 ~ 1.0)
    /// offset_from_cut: 相对于接缝切点的时间偏移 (负数在切点前，正数在切点后)
    pub fn progress_at_offset(&self, offset_from_cut: FrameTime) -> f32 {
        let half_dur = self.duration.0 as f64 / 2.0;
        let t = offset_from_cut.0 as f64;

        match self.alignment {
            TransitionAlignment::Center => {
                let norm = (t + half_dur) / (self.duration.0 as f64);
                norm.clamp(0.0, 1.0) as f32
            }
            TransitionAlignment::StartOnCut => {
                let norm = t / (self.duration.0 as f64);
                norm.clamp(0.0, 1.0) as f32
            }
            TransitionAlignment::EndOnCut => {
                let norm = (t + self.duration.0 as f64) / (self.duration.0 as f64);
                norm.clamp(0.0, 1.0) as f32
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_progress_calculation() {
        let trans = Transition::new(TransitionType::CrossDissolve, 1.0); // 1.0s, Center 对齐

        // 切点前 0.5s: 进度 0.0
        assert_eq!(trans.progress_at_offset(FrameTime(-500_000)), 0.0);
        // 切点处 0.0s: 进度 0.5 (正中心)
        assert_eq!(trans.progress_at_offset(FrameTime(0)), 0.5);
        // 切点后 0.5s: 进度 1.0
        assert_eq!(trans.progress_at_offset(FrameTime(500_000)), 1.0);
    }

    #[test]
    fn test_transition_from_str_loose() {
        assert_eq!(TransitionType::from_str_loose("dissolve"), Some(TransitionType::CrossDissolve));
        assert_eq!(TransitionType::from_str_loose("wipe_left"), Some(TransitionType::WipeLeft));
        assert_eq!(TransitionType::from_str_loose("wipe-right"), Some(TransitionType::WipeRight));
        assert_eq!(TransitionType::from_str_loose("dip_black"), Some(TransitionType::DipToBlack));
        assert_eq!(TransitionType::from_str_loose("flash"), Some(TransitionType::DipToWhite));
        assert_eq!(TransitionType::from_str_loose("unknown"), None);
    }
}
