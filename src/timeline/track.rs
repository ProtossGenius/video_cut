use serde::{Deserialize, Serialize};

use super::clip::{Clip, ClipId};

/// 轨道ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrackId(pub u64);

/// 轨道数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: TrackId,
    pub name: String,

    /// 轨道上的所有切片
    /// 应当维持按 timeline_start 排序
    pub clips: Vec<Clip>,

    /// 是否置顶
    pub is_pinned: bool,
    /// 置顶顺序 (若为None则非置顶，或按自然顺序)
    pub pin_order: Option<u32>,
    /// 轨道独立音量推子 (0.0 ~ 2.0, 默认 1.0)
    pub volume: f32,
    /// 立体声声相平衡 (-1.0 全左 ~ +1.0 全右, 默认 0.0 居中)
    pub pan: f32,
    /// 是否静音
    pub is_muted: bool,
    /// 是否独奏
    pub is_solo: bool,
    /// 轨道级音频处理链（3 段 EQ + EBU R128 响度标准化）
    pub audio_processor: crate::media::audio_pipeline::TrackAudioProcessor,
    /// 轨道色彩标签 (Color Tag)
    pub color_tag: super::clip::ColorTagPreset,
}

impl Track {
    pub fn new(id: TrackId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            clips: Vec::new(),
            is_pinned: false,
            pin_order: None,
            volume: 1.0,
            pan: 0.0,
            is_muted: false,
            is_solo: false,
            audio_processor: crate::media::audio_pipeline::TrackAudioProcessor::default(),
            color_tag: super::clip::ColorTagPreset::None,
        }
    }

    pub fn with_volume(mut self, vol: f32) -> Self {
        self.volume = vol.clamp(0.0, 2.0);
        self
    }

    pub fn with_pan(mut self, pan: f32) -> Self {
        self.pan = pan.clamp(-1.0, 1.0);
        self
    }

    /// 根据常数能量等功率声相法则计算左右声道输出乘数
    pub fn stereo_pan_gains(&self) -> (f32, f32) {
        let theta = (self.pan.clamp(-1.0, 1.0) + 1.0) * (std::f32::consts::PI / 4.0);
        let left_gain = theta.cos() * self.volume;
        let right_gain = theta.sin() * self.volume;
        (left_gain, right_gain)
    }

    pub fn add_clip(&mut self, clip: Clip) {
        self.clips.push(clip);
        self.sort_clips();
    }

    pub fn remove_clip(&mut self, id: ClipId) -> Option<Clip> {
        if let Some(pos) = self.clips.iter().position(|c| c.id == id) {
            Some(self.clips.remove(pos))
        } else {
            None
        }
    }

    pub fn sort_clips(&mut self) {
        self.clips.sort_by_key(|c| c.timeline_start);
    }
}
