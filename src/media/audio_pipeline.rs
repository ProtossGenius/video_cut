use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::timeline::FrameTime;

/// 音频主时钟（Audio Master Clock）
/// 保证整个剪辑软件的时间基准始终对齐硬件声卡消耗的采样数
#[derive(Debug, Clone)]
pub struct AudioMasterClock {
    consumed_samples: Arc<AtomicU64>,
    sample_rate: u32,
}

impl AudioMasterClock {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            consumed_samples: Arc::new(AtomicU64::new(0)),
            sample_rate: sample_rate.max(1),
        }
    }

    /// 获取当前物理音频主时钟对应的微秒时间 (FrameTime)
    pub fn current_time(&self) -> FrameTime {
        let samples = self.consumed_samples.load(Ordering::Acquire);
        let us = (samples as u128 * 1_000_000) / (self.sample_rate as u128);
        FrameTime(us as i64)
    }

    /// 音频回调驱动推进采样点计数
    pub fn advance(&self, sample_frames: u64) {
        self.consumed_samples
            .fetch_add(sample_frames, Ordering::Release);
    }

    /// 跳转/定位到指定时间
    pub fn seek_to(&self, time: FrameTime) {
        let target_us = time.0.max(0) as u128;
        let target_samples = (target_us * self.sample_rate as u128) / 1_000_000;
        self.consumed_samples
            .store(target_samples as u64, Ordering::Release);
    }

    /// 获取采样率
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

/// 音频混音器（多轨道立体声线性加权与软饱和截断）
pub struct AudioMixer;

impl AudioMixer {
    /// 混合两个立体声通道 (左声道, 右声道)
    pub fn mix_stereo_samples(tracks: &[([f32; 2], f32)]) -> [f32; 2] {
        let mut left_sum = 0.0f32;
        let mut right_sum = 0.0f32;

        for &([l, r], gain) in tracks {
            left_sum += l * gain;
            right_sum += r * gain;
        }

        // 软截断防爆音
        [left_sum.clamp(-1.0, 1.0), right_sum.clamp(-1.0, 1.0)]
    }

    /// 应用主音量增益与静音状态
    pub fn apply_master_gain(samples: [f32; 2], gain: f32, is_muted: bool) -> [f32; 2] {
        if is_muted {
            [0.0, 0.0]
        } else {
            let clamped_gain = gain.clamp(0.0, 2.0);
            [
                (samples[0] * clamped_gain).clamp(-1.0, 1.0),
                (samples[1] * clamped_gain).clamp(-1.0, 1.0),
            ]
        }
    }
}

/// 立体声 VU 峰值电平表物理衰减计算状态
#[derive(Debug, Clone, Default)]
pub struct VuMeterState {
    pub left_rms: f32,
    pub right_rms: f32,
    pub left_peak: f32,
    pub right_peak: f32,
    pub is_clipping: bool,
}

impl VuMeterState {
    pub fn update(&mut self, target_left: f32, target_right: f32, dt_seconds: f32) {
        let t_left = target_left.clamp(0.0, 1.0);
        let t_right = target_right.clamp(0.0, 1.0);

        // RMS 快速追踪
        let attack = 15.0 * dt_seconds;
        self.left_rms += (t_left - self.left_rms) * attack.min(1.0);
        self.right_rms += (t_right - self.right_rms) * attack.min(1.0);

        // 峰值衰减 (Ballistic Decay, ~20dB/sec)
        let decay = (dt_seconds * 0.85).min(1.0);
        if t_left >= self.left_peak {
            self.left_peak = t_left;
        } else {
            self.left_peak = (self.left_peak - decay).max(0.0);
        }

        if t_right >= self.right_peak {
            self.right_peak = t_right;
        } else {
            self.right_peak = (self.right_peak - decay).max(0.0);
        }

        self.is_clipping = self.left_peak >= 0.98 || self.right_peak >= 0.98;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_master_clock_pts_calculation() {
        let clock = AudioMasterClock::new(48000);
        assert_eq!(clock.current_time(), FrameTime(0));

        // 消耗 48000 个采样帧 -> 恰好 1 秒 (1_000_000 us)
        clock.advance(48000);
        assert_eq!(clock.current_time(), FrameTime(1_000_000));

        // 再次消耗 24000 个采样 -> 1.5 秒
        clock.advance(24000);
        assert_eq!(clock.current_time(), FrameTime(1_500_000));

        // 跳转到 5 秒
        clock.seek_to(FrameTime(5_000_000));
        assert_eq!(clock.current_time(), FrameTime(5_000_000));
    }

    #[test]
    fn test_audio_mixer_clamping() {
        let track1 = ([0.8, 0.8], 1.0);
        let track2 = ([0.5, 0.5], 1.0);

        let mixed = AudioMixer::mix_stereo_samples(&[track1, track2]);
        // 0.8 + 0.5 = 1.3 -> clamp 到 1.0
        assert_eq!(mixed, [1.0, 1.0]);
    }

    #[test]
    fn test_audio_master_gain_and_vu_meter() {
        let samples = [0.5, 0.5];
        // 增益 1.5 倍
        let boosted = AudioMixer::apply_master_gain(samples, 1.5, false);
        assert_eq!(boosted, [0.75, 0.75]);

        // 静音
        let muted = AudioMixer::apply_master_gain(samples, 1.5, true);
        assert_eq!(muted, [0.0, 0.0]);

        // VU 电平表物理衰减测试
        let mut vu = VuMeterState::default();
        vu.update(0.8, 0.9, 0.016);
        assert!(vu.left_rms > 0.0);
        assert_eq!(vu.left_peak, 0.8);
        assert_eq!(vu.right_peak, 0.9);

        // 下一帧目标为 0，Peak 平滑衰减
        vu.update(0.0, 0.0, 0.016);
        assert!(vu.left_peak < 0.8);
        assert!(vu.left_peak > 0.7);
    }
}
