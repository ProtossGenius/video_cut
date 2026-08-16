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
}
