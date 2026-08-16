use crate::timeline::FrameTime;
use serde::{Deserialize, Serialize};

/// 单个音频波形采样点（包含正负峰值与均方根 RMS）
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WaveformPeak {
    pub min_amplitude: f32, // -1.0 ~ 0.0
    pub max_amplitude: f32, // 0.0 ~ 1.0
    pub rms: f32,           // 0.0 ~ 1.0 (音量感知均方根)
}

/// 音频波形概要数据（用于时间线可视化）
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AudioWaveform {
    /// 采样率（例如 44100 或 48000 Hz）
    pub sample_rate: u32,
    /// 每个波形峰值点对应的微秒数 (us)
    pub us_per_peak: u32,
    /// 连续波形峰值点序列
    pub peaks: Vec<WaveformPeak>,
}

impl AudioWaveform {
    pub fn new(sample_rate: u32, us_per_peak: u32) -> Self {
        Self {
            sample_rate,
            us_per_peak,
            peaks: Vec::new(),
        }
    }

    /// 从原始 f32 PCM 音频样本流中构建波形概要
    pub fn from_pcm_samples(samples: &[f32], sample_rate: u32, target_peaks_per_sec: u32) -> Self {
        if samples.is_empty() || sample_rate == 0 || target_peaks_per_sec == 0 {
            return Self::new(sample_rate.max(48000), 10_000);
        }

        let samples_per_peak = (sample_rate / target_peaks_per_sec).max(1) as usize;
        let us_per_peak = (1_000_000 / target_peaks_per_sec).max(1);

        let mut peaks = Vec::with_capacity(samples.len() / samples_per_peak + 1);

        for chunk in samples.chunks(samples_per_peak) {
            let mut min_val = 0.0f32;
            let mut max_val = 0.0f32;
            let mut sum_sq = 0.0f32;

            for &s in chunk {
                if s < min_val {
                    min_val = s;
                }
                if s > max_val {
                    max_val = s;
                }
                sum_sq += s * s;
            }

            let rms = (sum_sq / chunk.len() as f32).sqrt().min(1.0);
            peaks.push(WaveformPeak {
                min_amplitude: min_val.clamp(-1.0, 0.0),
                max_amplitude: max_val.clamp(0.0, 1.0),
                rms,
            });
        }

        Self {
            sample_rate,
            us_per_peak,
            peaks,
        }
    }

    /// 查询指定时间段的波形数据切片
    pub fn get_peaks_range(&self, start: FrameTime, duration: FrameTime) -> &[WaveformPeak] {
        if self.us_per_peak == 0 || self.peaks.is_empty() {
            return &[];
        }

        let start_idx = (start.0 as usize / self.us_per_peak as usize).min(self.peaks.len());
        let count = (duration.0 as usize / self.us_per_peak as usize).max(1);
        let end_idx = (start_idx + count).min(self.peaks.len());

        &self.peaks[start_idx..end_idx]
    }

    /// 寻找下一个有声音 (高于阈值) 的时间点 (用于快捷键 w/b 跳转)
    pub fn find_next_audible_point(
        &self,
        current_time: FrameTime,
        threshold: f32,
    ) -> Option<FrameTime> {
        let cur_idx = (current_time.0 as usize / self.us_per_peak as usize).min(self.peaks.len());

        // 先跳出当前的活动区，再找下一个发声点
        let mut in_silence = false;
        for i in cur_idx..self.peaks.len() {
            let peak = &self.peaks[i];
            let is_audible = peak.max_amplitude > threshold || peak.min_amplitude.abs() > threshold;
            if !is_audible {
                in_silence = true;
            } else if in_silence {
                return Some(FrameTime(i as i64 * self.us_per_peak as i64));
            }
        }
        None
    }

    /// 寻找上一个有声音的时间点
    pub fn find_prev_audible_point(
        &self,
        current_time: FrameTime,
        threshold: f32,
    ) -> Option<FrameTime> {
        let cur_idx = (current_time.0 as usize / self.us_per_peak as usize).min(self.peaks.len());
        if cur_idx == 0 {
            return None;
        }

        let mut in_silence = false;
        for i in (0..cur_idx).rev() {
            let peak = &self.peaks[i];
            let is_audible = peak.max_amplitude > threshold || peak.min_amplitude.abs() > threshold;
            if !is_audible {
                in_silence = true;
            } else if in_silence {
                return Some(FrameTime(i as i64 * self.us_per_peak as i64));
            }
        }
        None
    }

    /// 自动提取波形中的重音节拍与瞬态打击点 (Audio Transient & Beat Onsets)
    /// 基于短时能量导数与局部极大值检测
    pub fn detect_transients(&self, threshold: f32) -> Vec<FrameTime> {
        let mut transients = Vec::new();
        if self.peaks.len() < 3 || self.us_per_peak == 0 {
            return transients;
        }

        let mut prev_rms = self.peaks[0].rms;
        for i in 1..(self.peaks.len() - 1) {
            let cur_rms = self.peaks[i].rms;
            let next_rms = self.peaks[i + 1].rms;
            let delta = cur_rms - prev_rms;

            // 当能量快速攀升且当前点为局部极大值，且高于阈值时判定为瞬态打击点
            if delta > threshold && cur_rms >= next_rms && cur_rms > 0.15 {
                let time_us = i as i64 * self.us_per_peak as i64;
                // 避免 100ms 内重复触发
                if let Some(&last) = transients.last() {
                    if (time_us - last.0).abs() < 100_000 {
                        prev_rms = cur_rms;
                        continue;
                    }
                }
                transients.push(FrameTime(time_us));
            }
            prev_rms = cur_rms;
        }
        transients
    }
}

/// 多分辨率音频波形 LOD 金字塔缓存 (Level-of-Detail Pyramid)
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WaveformLodPyramid {
    /// L0: 精细层级 (~480 peaks/sec, 极大放大时查看)
    pub l0: AudioWaveform,
    /// L1: 常规层级 (~60 peaks/sec, 标准缩放查看)
    pub l1: AudioWaveform,
    /// L2: 概览层级 (~8 peaks/sec, 全景缩略查看)
    pub l2: AudioWaveform,
}

impl WaveformLodPyramid {
    /// 从原始 PCM 样本流生成全套多级波形金字塔缓存
    pub fn from_pcm_samples(samples: &[f32], sample_rate: u32) -> Self {
        let l0 = AudioWaveform::from_pcm_samples(samples, sample_rate, 480);
        let l1 = AudioWaveform::from_pcm_samples(samples, sample_rate, 60);
        let l2 = AudioWaveform::from_pcm_samples(samples, sample_rate, 8);
        Self { l0, l1, l2 }
    }

    /// 根据当前时间线缩放倍率 (像素/秒) 智能选取最优 LOD 层级
    pub fn select_lod(&self, pixels_per_sec: f32) -> &AudioWaveform {
        if pixels_per_sec > 200.0 {
            &self.l0
        } else if pixels_per_sec > 30.0 {
            &self.l1
        } else {
            &self.l2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waveform_generation_from_sine_wave() {
        // 生成 1 秒 48000Hz 纯正弦波
        let sample_rate = 48000;
        let mut samples = Vec::with_capacity(sample_rate as usize);
        for i in 0..sample_rate {
            let t = i as f32 / sample_rate as f32;
            let sample = (t * 440.0 * 2.0 * std::f32::consts::PI).sin() * 0.8;
            samples.push(sample);
        }

        let wf = AudioWaveform::from_pcm_samples(&samples, sample_rate, 100); // 100 peaks/sec
        assert_eq!(wf.peaks.len(), 100);
        assert!(wf.peaks[0].max_amplitude > 0.7);
        assert!(wf.peaks[0].min_amplitude < -0.7);
    }

    #[test]
    fn test_waveform_lod_pyramid_selection() {
        let sample_rate = 48000;
        let samples = vec![0.5f32; 48000]; // 1秒音频
        let pyramid = WaveformLodPyramid::from_pcm_samples(&samples, sample_rate);

        assert_eq!(pyramid.l0.peaks.len(), 480);
        assert_eq!(pyramid.l1.peaks.len(), 60);
        assert_eq!(pyramid.l2.peaks.len(), 8);

        // 高缩放率选择 L0
        assert_eq!(pyramid.select_lod(300.0).peaks.len(), 480);
        // 中缩放率选择 L1
        assert_eq!(pyramid.select_lod(80.0).peaks.len(), 60);
        // 低缩放率选择 L2
        assert_eq!(pyramid.select_lod(10.0).peaks.len(), 8);
    }

    #[test]
    fn test_find_audible_points() {
        // 构建一段 2s 静音 + 1s 声音 + 2s 静音的波形
        let sample_rate = 1000;
        let mut samples = vec![0.0f32; 2000]; // 2s 静音
        samples.extend(vec![0.8f32; 1000]); // 1s 声音
        samples.extend(vec![0.0f32; 2000]); // 2s 静音

        let wf = AudioWaveform::from_pcm_samples(&samples, sample_rate, 10);
        let next_point = wf.find_next_audible_point(FrameTime(0), 0.1);
        assert!(next_point.is_some());
        assert_eq!(next_point.unwrap(), FrameTime(2_000_000)); // 2.0s
    }

    #[test]
    fn test_detect_transients() {
        // 构建 1s 静音 + 突然爆发瞬态 (0.9) + 1s 平缓
        let sample_rate = 1000;
        let mut samples = vec![0.0f32; 1000]; // 1s 静音
        samples.extend(vec![0.9f32; 50]);     // 50ms 瞬态重音打击
        samples.extend(vec![0.1f32; 950]);    // 950ms 低音平缓

        let wf = AudioWaveform::from_pcm_samples(&samples, sample_rate, 50); // 50 peaks/sec (20ms/peak)
        let transients = wf.detect_transients(0.2);
        assert!(!transients.is_empty());
        assert!((transients[0].0 - 1_000_000).abs() < 100_000); // 应该在 1.0s 附近检测到打击点
    }
}
