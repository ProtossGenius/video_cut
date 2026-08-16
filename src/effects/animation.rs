use crate::timeline::FrameTime;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AnimationError {
    #[error("关键帧时间必须单调递增，当前时间 {current:?} 小于等于上一帧时间 {previous:?}")]
    NonMonotonicTime {
        current: FrameTime,
        previous: FrameTime,
    },
    #[error("无效的时间表达式或相对时间: {0}")]
    InvalidTimeExpression(String),
}

/// 坐标值（支持百分比与绝对像素）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CoordVal {
    Pixel(f32),
    Percent(f32), // 0.0 ~ 1.0 (例如 50% = 0.5)
}

impl CoordVal {
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        if let Some(stripped) = s.strip_suffix('%') {
            let num: f32 = stripped.parse().ok()?;
            Some(CoordVal::Percent(num / 100.0))
        } else {
            let num: f32 = s.parse().ok()?;
            Some(CoordVal::Pixel(num))
        }
    }

    pub fn to_pixel(&self, total_size: f32) -> f32 {
        match self {
            CoordVal::Pixel(px) => *px,
            CoordVal::Percent(pct) => pct * total_size,
        }
    }
}

/// 关键帧中的具体动作
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnimationAction {
    MoveTo {
        x: CoordVal,
        y: CoordVal,
    },
    Resize {
        scale_x: f32,
        scale_y: f32,
        interpolate: bool,
    },
    Rotate {
        angle_deg: f32,
        interpolate: bool,
    },
    Opacity {
        alpha: f32, // 0.0 ~ 1.0
        interpolate: bool,
    },
}

/// 单个关键帧
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    pub time_offset: FrameTime,
    pub actions: Vec<AnimationAction>,
}

/// 计算出的变换状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluatedTransform {
    pub x: f32,
    pub y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation_deg: f32,
    pub opacity: f32,
}

impl Default for EvaluatedTransform {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation_deg: 0.0,
            opacity: 1.0,
        }
    }
}

/// 动画序列管理器
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AnimationSequence {
    pub keyframes: Vec<Keyframe>,
}

impl AnimationSequence {
    pub fn new() -> Self {
        Self {
            keyframes: Vec::new(),
        }
    }

    /// 添加关键帧，严格校验时间单调递增
    pub fn add_keyframe(&mut self, keyframe: Keyframe) -> Result<(), AnimationError> {
        if let Some(last) = self.keyframes.last() {
            if keyframe.time_offset <= last.time_offset {
                return Err(AnimationError::NonMonotonicTime {
                    current: keyframe.time_offset,
                    previous: last.time_offset,
                });
            }
        }
        self.keyframes.push(keyframe);
        Ok(())
    }

    /// 在指定时间点评估插值后的变换参数
    pub fn evaluate_at(
        &self,
        time: FrameTime,
        canvas_width: f32,
        canvas_height: f32,
    ) -> EvaluatedTransform {
        if self.keyframes.is_empty() {
            return EvaluatedTransform::default();
        }

        // 如果在第一个关键帧之前，使用首帧状态
        if time <= self.keyframes[0].time_offset {
            return Self::apply_actions(&self.keyframes[0].actions, canvas_width, canvas_height);
        }

        // 如果在最后一个关键帧之后，使用末帧状态
        let last_kf = self.keyframes.last().unwrap();
        if time >= last_kf.time_offset {
            return Self::apply_actions(&last_kf.actions, canvas_width, canvas_height);
        }

        // 在两个相邻关键帧之间进行线性插值
        for i in 0..self.keyframes.len() - 1 {
            let kf0 = &self.keyframes[i];
            let kf1 = &self.keyframes[i + 1];

            if time >= kf0.time_offset && time <= kf1.time_offset {
                let t0 = kf0.time_offset.0 as f32;
                let t1 = kf1.time_offset.0 as f32;
                let factor = if (t1 - t0).abs() < f32::EPSILON {
                    1.0
                } else {
                    ((time.0 as f32) - t0) / (t1 - t0)
                };

                let state0 = Self::apply_actions(&kf0.actions, canvas_width, canvas_height);
                let state1 = Self::apply_actions(&kf1.actions, canvas_width, canvas_height);

                return EvaluatedTransform {
                    x: state0.x + (state1.x - state0.x) * factor,
                    y: state0.y + (state1.y - state0.y) * factor,
                    scale_x: state0.scale_x + (state1.scale_x - state0.scale_x) * factor,
                    scale_y: state0.scale_y + (state1.scale_y - state0.scale_y) * factor,
                    rotation_deg: state0.rotation_deg
                        + (state1.rotation_deg - state0.rotation_deg) * factor,
                    opacity: state0.opacity + (state1.opacity - state0.opacity) * factor,
                };
            }
        }

        EvaluatedTransform::default()
    }

    fn apply_actions(
        actions: &[AnimationAction],
        canvas_width: f32,
        canvas_height: f32,
    ) -> EvaluatedTransform {
        let mut transform = EvaluatedTransform::default();
        for action in actions {
            match action {
                AnimationAction::MoveTo { x, y } => {
                    transform.x = x.to_pixel(canvas_width);
                    transform.y = y.to_pixel(canvas_height);
                }
                AnimationAction::Resize {
                    scale_x, scale_y, ..
                } => {
                    transform.scale_x = *scale_x;
                    transform.scale_y = *scale_y;
                }
                AnimationAction::Rotate { angle_deg, .. } => {
                    transform.rotation_deg = *angle_deg;
                }
                AnimationAction::Opacity { alpha, .. } => {
                    transform.opacity = *alpha;
                }
            }
        }
        transform
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monotonic_keyframes() {
        let mut seq = AnimationSequence::new();
        let kf1 = Keyframe {
            time_offset: FrameTime(1_000_000), // 1s
            actions: vec![AnimationAction::MoveTo {
                x: CoordVal::Percent(0.3),
                y: CoordVal::Pixel(400.0),
            }],
        };
        assert!(seq.add_keyframe(kf1).is_ok());

        // 试图插入更早的关键帧，应报错
        let kf_earlier = Keyframe {
            time_offset: FrameTime(500_000), // 0.5s
            actions: vec![],
        };
        let err = seq.add_keyframe(kf_earlier);
        assert!(err.is_err());
        assert!(matches!(
            err.unwrap_err(),
            AnimationError::NonMonotonicTime { .. }
        ));
    }

    #[test]
    fn test_interpolation_at_halfway() {
        let mut seq = AnimationSequence::new();
        seq.add_keyframe(Keyframe {
            time_offset: FrameTime(0),
            actions: vec![AnimationAction::MoveTo {
                x: CoordVal::Pixel(0.0),
                y: CoordVal::Pixel(0.0),
            }],
        })
        .unwrap();

        seq.add_keyframe(Keyframe {
            time_offset: FrameTime(2_000_000), // 2s
            actions: vec![AnimationAction::MoveTo {
                x: CoordVal::Pixel(100.0),
                y: CoordVal::Pixel(200.0),
            }],
        })
        .unwrap();

        // 1s 处应该刚好在中间 (50, 100)
        let eval = seq.evaluate_at(FrameTime(1_000_000), 1920.0, 1080.0);
        assert_eq!(eval.x, 50.0);
        assert_eq!(eval.y, 100.0);
    }
}
