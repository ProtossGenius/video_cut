use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::anchor::AnchorPoint;

/// 唯一标识媒体资产
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(pub u64);

/// 唯一标识一个切片
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClipId(pub u64);

/// 时间单位（帧）。为了避免浮点数精度问题，建议底层使用帧或采样点数计算。
/// 考虑到音频主时钟，这里可以作为一个相对时钟的微秒或帧数。
/// 这里暂时使用毫秒(ms)或者微秒(us)，为了适配高精度，使用微秒 i64。
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct FrameTime(pub i64);

impl FrameTime {
    pub const ZERO: FrameTime = FrameTime(0);

    pub fn from_seconds(secs: f64) -> Self {
        Self((secs * 1_000_000.0).round() as i64)
    }

    pub fn as_seconds(&self) -> f64 {
        self.0 as f64 / 1_000_000.0
    }
}

impl std::ops::Add for FrameTime {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        FrameTime(self.0 + rhs.0)
    }
}

impl std::ops::Sub for FrameTime {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        FrameTime(self.0 - rhs.0)
    }
}

impl std::fmt::Display for FrameTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.3}s", self.as_seconds())
    }
}

/// 切片数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: ClipId,
    pub name: String,

    /// 素材在原始文件中的起始时间
    pub source_in: FrameTime,
    /// 素材在原始文件中的结束时间
    pub source_out: FrameTime,
    /// 素材在时间线上的摆放起始时间
    pub timeline_start: FrameTime,

    /// 引用的源资产
    pub source: AssetId,

    /// 局部锚点
    pub anchors: HashMap<String, AnchorPoint>,
    /// 是否锁定（防止被涟漪效应或移动操作移位）
    pub locked: bool,
    /// 图层 Z 轴高度
    pub z_index: i32,
    /// 播放速度倍率 (默认 1.0)
    pub speed: f32,
    /// 音频淡入时长 (微秒)
    pub audio_fade_in: FrameTime,
    /// 音频淡出时长 (微秒)
    pub audio_fade_out: FrameTime,
}

impl Clip {
    pub fn new(
        id: ClipId,
        name: String,
        source: AssetId,
        timeline_start: FrameTime,
        duration: FrameTime,
    ) -> Self {
        Self {
            id,
            name,
            source,
            source_in: FrameTime::ZERO,
            source_out: duration,
            timeline_start,
            anchors: HashMap::new(),
            locked: false,
            z_index: 0,
            speed: 1.0,
            audio_fade_in: FrameTime::ZERO,
            audio_fade_out: FrameTime::ZERO,
        }
    }

    pub fn with_lock(mut self, locked: bool) -> Self {
        self.locked = locked;
        self
    }

    pub fn with_z_index(mut self, z: i32) -> Self {
        self.z_index = z;
        self
    }

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed.max(0.01);
        self
    }

    pub fn with_fade(mut self, fade_in: FrameTime, fade_out: FrameTime) -> Self {
        self.audio_fade_in = fade_in;
        self.audio_fade_out = fade_out;
        self
    }

    /// 计算切片在指定相对时间点 (相对于切片起始点) 的淡入淡出音量增益 (0.0 ~ 1.0)
    pub fn calculate_audio_fade_gain(&self, relative_time: FrameTime) -> f32 {
        let dur = self.duration().0;
        let t = relative_time.0.clamp(0, dur);

        let in_gain = if self.audio_fade_in.0 > 0 {
            (t as f32 / self.audio_fade_in.0 as f32).min(1.0)
        } else {
            1.0
        };

        let out_gain = if self.audio_fade_out.0 > 0 {
            ((dur - t) as f32 / self.audio_fade_out.0 as f32).min(1.0)
        } else {
            1.0
        };

        in_gain.min(out_gain).clamp(0.0, 1.0)
    }

    /// 计算切片在时间线上的时长（已考虑速度倍率）
    pub fn duration(&self) -> FrameTime {
        let raw_us = (self.source_out.0 - self.source_in.0) as f64;
        let speed_factor = (self.speed as f64).max(0.01);
        FrameTime((raw_us / speed_factor).round() as i64)
    }

    /// 计算切片在时间线上的结束时间
    pub fn timeline_end(&self) -> FrameTime {
        FrameTime(self.timeline_start.0 + self.duration().0)
    }
}
