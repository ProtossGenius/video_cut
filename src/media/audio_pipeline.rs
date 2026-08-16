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
}
