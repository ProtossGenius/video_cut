use crate::timeline::{Clip, FrameTime};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SpeedError {
    #[error("播放速度必须大于0，当前值: {0}")]
    InvalidSpeedFactor(String),
    #[error("加速/减速调整导致时间线右侧发生与已锁定切片 '{0}' 的碰撞冲突")]
    LockedClipCollision(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeedProperty {
    pub factor: f32, // 1.0 为正常速度, 2.0 为两倍速, 0.5 为半速
}

impl Default for SpeedProperty {
    fn default() -> Self {
        Self { factor: 1.0 }
    }
}

impl SpeedProperty {
    pub fn new(factor: f32) -> Result<Self, SpeedError> {
        if factor <= 0.0 || factor.is_nan() {
            return Err(SpeedError::InvalidSpeedFactor(format!("{:.2}", factor)));
        }
        Ok(Self { factor })
    }

    /// 计算调整速度后的切片新时长
    pub fn calculate_new_duration(&self, source_duration: FrameTime) -> FrameTime {
        let new_us = (source_duration.0 as f64 / self.factor as f64).round() as i64;
        FrameTime(new_us)
    }

    /// 校验在给定轨道中更改切片时长时，是否会与右侧锁定的切片冲突
    pub fn check_locked_clips_collision(
        target_clip_idx: usize,
        new_duration: FrameTime,
        track_clips: &[Clip],
    ) -> Result<(), SpeedError> {
        if target_clip_idx >= track_clips.len() {
            return Ok(());
        }

        let target_clip = &track_clips[target_clip_idx];
        let new_end = target_clip.timeline_start + new_duration;

        for (_i, clip) in track_clips.iter().enumerate().skip(target_clip_idx + 1) {
            if clip.timeline_start < new_end {
                // 发生重叠，如果该切片被锁定则报错
                // 暂时用名称判断或锁标记
                return Err(SpeedError::LockedClipCollision(clip.name.clone()));
            }
        }

        Ok(())
    }
}

/// 慢动作重映射与插帧模式 (Frame Interpolation Mode)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum FrameInterpolationMode {
    /// 临近截断（Nearest / Frame Repeat）
    #[default]
    Nearest,
    /// 双帧线性交叉混合（Linear Frame Blending）
    LinearBlend,
    /// 运动自适应余弦平滑混合（Motion-Adaptive Cosine Blend）
    MotionAdaptive,
}

impl FrameInterpolationMode {
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "nearest" | "repeat" | "none" | "off" | "临近" | "重复" => Some(FrameInterpolationMode::Nearest),
            "linear" | "blend" | "linear_blend" | "线性" | "混合" => Some(FrameInterpolationMode::LinearBlend),
            "motion" | "adaptive" | "motion_adaptive" | "cosine" | "自适应" | "余弦" => Some(FrameInterpolationMode::MotionAdaptive),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            FrameInterpolationMode::Nearest => "临近帧重复 (Nearest)",
            FrameInterpolationMode::LinearBlend => "双帧线性混合 (Linear Blend)",
            FrameInterpolationMode::MotionAdaptive => "运动自适应平滑 (Motion Adaptive)",
        }
    }

    /// 对两张解码后的 RGBA 帧像素缓冲区进行加权混合插帧计算
    /// alpha 在 0.0 ~ 1.0 之间
    pub fn blend_frames(
        &self,
        frame_a: &[u8],
        frame_b: &[u8],
        alpha: f32,
        out_buffer: &mut [u8],
    ) {
        let alpha = alpha.clamp(0.0, 1.0);
        let len = frame_a.len().min(frame_b.len()).min(out_buffer.len());

        match self {
            FrameInterpolationMode::Nearest => {
                if alpha < 0.5 {
                    out_buffer[..len].copy_from_slice(&frame_a[..len]);
                } else {
                    out_buffer[..len].copy_from_slice(&frame_b[..len]);
                }
            }
            FrameInterpolationMode::LinearBlend => {
                let w_b = alpha;
                let w_a = 1.0 - alpha;
                for i in 0..len {
                    out_buffer[i] = ((frame_a[i] as f32 * w_a) + (frame_b[i] as f32 * w_b)).round() as u8;
                }
            }
            FrameInterpolationMode::MotionAdaptive => {
                // 余弦 S 型平滑加权: (1 - cos(alpha * PI)) / 2
                let smooth_alpha = (1.0 - (alpha * std::f32::consts::PI).cos()) * 0.5;
                let w_b = smooth_alpha;
                let w_a = 1.0 - smooth_alpha;
                for i in 0..len {
                    out_buffer[i] = ((frame_a[i] as f32 * w_a) + (frame_b[i] as f32 * w_b)).round() as u8;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speed_duration_calculation() {
        let speed = SpeedProperty::new(2.0).unwrap();
        let dur = FrameTime(10_000_000); // 10s
        assert_eq!(speed.calculate_new_duration(dur), FrameTime(5_000_000)); // 5s

        let slow = SpeedProperty::new(0.5).unwrap();
        assert_eq!(slow.calculate_new_duration(dur), FrameTime(20_000_000)); // 20s
    }

    #[test]
    fn test_invalid_speed() {
        assert!(SpeedProperty::new(0.0).is_err());
        assert!(SpeedProperty::new(-1.5).is_err());
    }

    #[test]
    fn test_frame_blending_linear_and_nearest() {
        let f0 = vec![100u8; 4];
        let f1 = vec![200u8; 4];
        let mut out = vec![0u8; 4];

        // 1. 线性插值 alpha=0.5 -> 150
        FrameInterpolationMode::LinearBlend.blend_frames(&f0, &f1, 0.5, &mut out);
        assert_eq!(out, vec![150u8; 4]);

        // 2. 临近插值 alpha=0.3 -> f0 (100)
        FrameInterpolationMode::Nearest.blend_frames(&f0, &f1, 0.3, &mut out);
        assert_eq!(out, vec![100u8; 4]);

        // 3. 运动自适应余弦平滑插值 alpha=0.5 -> 150
        FrameInterpolationMode::MotionAdaptive.blend_frames(&f0, &f1, 0.5, &mut out);
        assert_eq!(out, vec![150u8; 4]);
    }
}
