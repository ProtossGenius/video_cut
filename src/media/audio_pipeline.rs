use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

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

    /// 对单轨立体声采样应用该轨道的音量增益与常数能量等功率声相法则
    pub fn apply_track_pan_and_volume(sample: [f32; 2], volume: f32, pan: f32, is_muted: bool) -> [f32; 2] {
        if is_muted {
            return [0.0, 0.0];
        }
        let theta = (pan.clamp(-1.0, 1.0) + 1.0) * (std::f32::consts::PI / 4.0);
        let left_gain = theta.cos() * volume.clamp(0.0, 2.0);
        let right_gain = theta.sin() * volume.clamp(0.0, 2.0);
        [
            (sample[0] * left_gain).clamp(-1.0, 1.0),
            (sample[1] * right_gain).clamp(-1.0, 1.0),
        ]
    }

    /// 估算立体声缓冲的积分响度（简化版 EBU R128 LUFS）
    pub fn estimate_integrated_lufs(samples: &[[f32; 2]]) -> f32 {
        if samples.is_empty() {
            return -70.0;
        }

        let mut energy = 0.0f64;
        for [left, right] in samples {
            energy += (((left * left) + (right * right)) * 0.5) as f64;
        }
        energy /= samples.len() as f64;

        if energy <= 1e-12 {
            -70.0
        } else {
            (-0.691 + 10.0 * energy.log10()) as f32
        }
    }

    /// 根据测得 LUFS 与目标 LUFS 计算建议补偿增益
    pub fn calculate_loudness_gain_db(measured_lufs: f32, target: LoudnessTarget) -> f32 {
        target.target_lufs() - measured_lufs
    }

    /// 对整个立体声缓冲区应用 dB 增益
    pub fn apply_gain_db(samples: &mut [[f32; 2]], gain_db: f32) {
        let gain = 10f32.powf(gain_db / 20.0);
        for [left, right] in samples {
            *left = (*left * gain).clamp(-1.0, 1.0);
            *right = (*right * gain).clamp(-1.0, 1.0);
        }
    }
}

/// EQ 频段选择器
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EqBandSelector {
    Low,
    Mid,
    High,
}

impl EqBandSelector {
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "low" | "l" | "bass" => Some(Self::Low),
            "mid" | "m" | "middle" => Some(Self::Mid),
            "high" | "h" | "treble" => Some(Self::High),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Low => "低频",
            Self::Mid => "中频",
            Self::High => "高频",
        }
    }

    pub fn default_q(&self) -> f32 {
        match self {
            Self::Low | Self::High => 0.707,
            Self::Mid => 1.0,
        }
    }

    pub fn default_frequency_hz(&self) -> f32 {
        match self {
            Self::Low => 120.0,
            Self::Mid => 1_800.0,
            Self::High => 8_500.0,
        }
    }
}

/// 双二阶滤波器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EqBandKind {
    LowShelf,
    Peaking,
    HighShelf,
}

/// 单个参数均衡器频段
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EqBand {
    pub selector: EqBandSelector,
    pub kind: EqBandKind,
    pub frequency_hz: f32,
    pub gain_db: f32,
    pub q: f32,
    pub enabled: bool,
}

impl EqBand {
    pub fn new(selector: EqBandSelector, kind: EqBandKind, frequency_hz: f32, q: f32) -> Self {
        Self {
            selector,
            kind,
            frequency_hz,
            gain_db: 0.0,
            q,
            enabled: true,
        }
    }

    pub fn set(&mut self, frequency_hz: f32, gain_db: f32, q: f32) {
        self.frequency_hz = frequency_hz.clamp(20.0, 20_000.0);
        self.gain_db = gain_db.clamp(-18.0, 18.0);
        self.q = q.clamp(0.1, 10.0);
        self.enabled = true;
    }

    pub fn is_flat(&self) -> bool {
        !self.enabled || self.gain_db.abs() < 0.01
    }

    fn coefficients(&self, sample_rate: u32) -> BiquadCoefficients {
        if self.is_flat() {
            return BiquadCoefficients::identity();
        }

        let sample_rate = sample_rate.max(1) as f32;
        let f0 = self.frequency_hz.clamp(20.0, sample_rate * 0.45);
        let q = self.q.clamp(0.1, 10.0);
        let a = 10f32.powf(self.gain_db / 40.0);
        let w0 = 2.0 * std::f32::consts::PI * f0 / sample_rate;
        let cos_w0 = w0.cos();
        let alpha = w0.sin() / (2.0 * q);

        let (b0, b1, b2, a0, a1, a2) = match self.kind {
            EqBandKind::Peaking => (
                1.0 + alpha * a,
                -2.0 * cos_w0,
                1.0 - alpha * a,
                1.0 + alpha / a,
                -2.0 * cos_w0,
                1.0 - alpha / a,
            ),
            EqBandKind::LowShelf => {
                let sqrt_a = a.sqrt();
                (
                    a * ((a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * sqrt_a * alpha),
                    2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0),
                    a * ((a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * sqrt_a * alpha),
                    (a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * sqrt_a * alpha,
                    -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0),
                    (a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * sqrt_a * alpha,
                )
            }
            EqBandKind::HighShelf => {
                let sqrt_a = a.sqrt();
                (
                    a * ((a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * sqrt_a * alpha),
                    -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0),
                    a * ((a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * sqrt_a * alpha),
                    (a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * sqrt_a * alpha,
                    2.0 * ((a - 1.0) - (a + 1.0) * cos_w0),
                    (a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * sqrt_a * alpha,
                )
            }
        };

        BiquadCoefficients::new(b0, b1, b2, a0, a1, a2)
    }
}

/// 3 段参数均衡器预设
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EqPreset {
    Flat,
    Podcast,
    Vocal,
    BassCut,
    Brightness,
}

impl EqPreset {
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "flat" | "reset" => Some(Self::Flat),
            "podcast" | "speech" => Some(Self::Podcast),
            "vocal" | "voice" => Some(Self::Vocal),
            "bass_cut" | "basscut" | "cut_low" => Some(Self::BassCut),
            "brightness" | "bright" | "air" => Some(Self::Brightness),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Flat => "Flat",
            Self::Podcast => "Podcast",
            Self::Vocal => "Vocal",
            Self::BassCut => "Bass Cut",
            Self::Brightness => "Brightness",
        }
    }
}

/// 三段参数均衡器
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreeBandEq {
    pub low: EqBand,
    pub mid: EqBand,
    pub high: EqBand,
}

impl Default for ThreeBandEq {
    fn default() -> Self {
        Self {
            low: EqBand::new(EqBandSelector::Low, EqBandKind::LowShelf, 120.0, 0.707),
            mid: EqBand::new(EqBandSelector::Mid, EqBandKind::Peaking, 1_800.0, 1.0),
            high: EqBand::new(EqBandSelector::High, EqBandKind::HighShelf, 8_500.0, 0.707),
        }
    }
}

impl ThreeBandEq {
    pub fn set_band(&mut self, band: EqBandSelector, frequency_hz: f32, gain_db: f32, q: f32) {
        match band {
            EqBandSelector::Low => self.low.set(frequency_hz, gain_db, q),
            EqBandSelector::Mid => self.mid.set(frequency_hz, gain_db, q),
            EqBandSelector::High => self.high.set(frequency_hz, gain_db, q),
        }
    }

    pub fn apply_preset(&mut self, preset: EqPreset) {
        *self = Self::default();
        match preset {
            EqPreset::Flat => {}
            EqPreset::Podcast => {
                self.low.set(90.0, -3.5, 0.8);
                self.mid.set(2_200.0, 2.5, 1.2);
                self.high.set(9_500.0, 1.8, 0.75);
            }
            EqPreset::Vocal => {
                self.low.set(110.0, -2.0, 0.9);
                self.mid.set(3_000.0, 3.0, 1.0);
                self.high.set(10_500.0, 2.0, 0.7);
            }
            EqPreset::BassCut => {
                self.low.set(80.0, -6.0, 0.7);
                self.mid.set(1_600.0, 1.2, 1.1);
                self.high.set(8_000.0, 0.8, 0.7);
            }
            EqPreset::Brightness => {
                self.low.set(140.0, -1.5, 0.8);
                self.mid.set(2_400.0, 1.5, 1.0);
                self.high.set(11_000.0, 4.0, 0.65);
            }
        }
    }

    pub fn is_active(&self) -> bool {
        !self.low.is_flat() || !self.mid.is_flat() || !self.high.is_flat()
    }

    pub fn summary_line(&self) -> String {
        format!(
            "Low {:.0}Hz {:+.1}dB Q{:.2} | Mid {:.0}Hz {:+.1}dB Q{:.2} | High {:.0}Hz {:+.1}dB Q{:.2}",
            self.low.frequency_hz,
            self.low.gain_db,
            self.low.q,
            self.mid.frequency_hz,
            self.mid.gain_db,
            self.mid.q,
            self.high.frequency_hz,
            self.high.gain_db,
            self.high.q,
        )
    }

    pub fn preview_gain_multiplier(&self) -> f32 {
        let avg_gain_db = (self.low.gain_db + self.mid.gain_db + self.high.gain_db) / 6.0;
        10f32.powf(avg_gain_db / 20.0)
    }

    pub fn process_stereo_buffer(&self, samples: &mut [[f32; 2]], sample_rate: u32) {
        let low = self.low.coefficients(sample_rate);
        let mid = self.mid.coefficients(sample_rate);
        let high = self.high.coefficients(sample_rate);

        let mut low_l = BiquadState::default();
        let mut low_r = BiquadState::default();
        let mut mid_l = BiquadState::default();
        let mut mid_r = BiquadState::default();
        let mut high_l = BiquadState::default();
        let mut high_r = BiquadState::default();

        for frame in samples {
            let mut left = frame[0];
            let mut right = frame[1];
            left = low_l.process(left, low);
            right = low_r.process(right, low);
            left = mid_l.process(left, mid);
            right = mid_r.process(right, mid);
            left = high_l.process(left, high);
            right = high_r.process(right, high);
            frame[0] = left.clamp(-1.0, 1.0);
            frame[1] = right.clamp(-1.0, 1.0);
        }
    }
}

/// EBU R128 目标响度预设
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoudnessTarget {
    StreamingMinus14,
    BroadcastMinus23,
}

impl LoudnessTarget {
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "stream" | "streaming" | "-14" | "-14lufs" | "-14_lufs" => {
                Some(Self::StreamingMinus14)
            }
            "broadcast" | "ebu" | "r128" | "-23" | "-23lufs" | "-23_lufs" => {
                Some(Self::BroadcastMinus23)
            }
            _ => None,
        }
    }

    pub fn target_lufs(&self) -> f32 {
        match self {
            Self::StreamingMinus14 => -14.0,
            Self::BroadcastMinus23 => -23.0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::StreamingMinus14 => "Streaming -14 LUFS",
            Self::BroadcastMinus23 => "Broadcast -23 LUFS",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            Self::StreamingMinus14 => "-14 LUFS",
            Self::BroadcastMinus23 => "-23 LUFS",
        }
    }
}

/// 响度标准化状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoudnessNormalization {
    pub enabled: bool,
    pub target: LoudnessTarget,
    pub measured_lufs: f32,
    pub gain_db: f32,
}

impl Default for LoudnessNormalization {
    fn default() -> Self {
        Self {
            enabled: false,
            target: LoudnessTarget::StreamingMinus14,
            measured_lufs: -18.0,
            gain_db: 0.0,
        }
    }
}

impl LoudnessNormalization {
    pub fn from_measured(target: LoudnessTarget, measured_lufs: f32) -> Self {
        Self {
            enabled: true,
            target,
            measured_lufs,
            gain_db: AudioMixer::calculate_loudness_gain_db(measured_lufs, target),
        }
    }

    pub fn from_samples(target: LoudnessTarget, samples: &[[f32; 2]]) -> Self {
        let measured_lufs = AudioMixer::estimate_integrated_lufs(samples);
        Self::from_measured(target, measured_lufs)
    }

    pub fn gain_multiplier(&self) -> f32 {
        if self.enabled {
            10f32.powf(self.gain_db / 20.0)
        } else {
            1.0
        }
    }
}

/// 单条轨道的音频处理器链
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TrackAudioProcessor {
    pub eq: ThreeBandEq,
    pub loudness: LoudnessNormalization,
}

impl TrackAudioProcessor {
    pub fn apply_eq_preset(&mut self, preset: EqPreset) {
        self.eq.apply_preset(preset);
    }

    pub fn set_eq_band(&mut self, band: EqBandSelector, frequency_hz: f32, gain_db: f32, q: f32) {
        self.eq.set_band(band, frequency_hz, gain_db, q);
    }

    pub fn disable_loudness(&mut self) {
        self.loudness.enabled = false;
        self.loudness.gain_db = 0.0;
    }

    pub fn apply_loudness_target(
        &mut self,
        target: LoudnessTarget,
        measured_lufs: f32,
    ) -> LoudnessNormalization {
        let state = LoudnessNormalization::from_measured(target, measured_lufs);
        self.loudness = state.clone();
        state
    }

    pub fn preview_gain_multiplier(&self) -> f32 {
        self.eq.preview_gain_multiplier() * self.loudness.gain_multiplier()
    }

    pub fn estimate_track_input_lufs(track_volume: f32, clip_count: usize) -> f32 {
        let amplitude = (0.08 * track_volume.clamp(0.05, 2.0) * (clip_count.max(1) as f32).sqrt())
            .clamp(0.02, 0.95);
        let frames = 4_096usize;
        let mut probe = Vec::with_capacity(frames);
        for idx in 0..frames {
            let t = idx as f32 / 48_000.0;
            let left = (2.0 * std::f32::consts::PI * 330.0 * t).sin() * amplitude;
            let right = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * amplitude * 0.95;
            probe.push([left, right]);
        }
        AudioMixer::estimate_integrated_lufs(&probe)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
struct BiquadCoefficients {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

impl BiquadCoefficients {
    fn new(b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) -> Self {
        let a0 = if a0.abs() < f32::EPSILON { 1.0 } else { a0 };
        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    fn identity() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
struct BiquadState {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadState {
    fn process(&mut self, input: f32, coeffs: BiquadCoefficients) -> f32 {
        let output = coeffs.b0 * input
            + coeffs.b1 * self.x1
            + coeffs.b2 * self.x2
            - coeffs.a1 * self.y1
            - coeffs.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        output
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

    fn stereo_rms(samples: &[[f32; 2]]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let energy: f32 = samples
            .iter()
            .map(|[left, right]| ((left * left) + (right * right)) * 0.5)
            .sum();
        (energy / samples.len() as f32).sqrt()
    }

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

    #[test]
    fn test_track_pan_and_volume_math() {
        let sample = [1.0, 1.0];

        // 1. 居中 (Pan = 0.0), 等功率各为 0.7071
        let center = AudioMixer::apply_track_pan_and_volume(sample, 1.0, 0.0, false);
        assert!((center[0] - 0.7071).abs() < 0.01);
        assert!((center[1] - 0.7071).abs() < 0.01);

        // 2. 全左 (Pan = -1.0)
        let left_only = AudioMixer::apply_track_pan_and_volume(sample, 1.0, -1.0, false);
        assert!((left_only[0] - 1.0).abs() < 0.01);
        assert!(left_only[1].abs() < 0.01);

        // 3. 全右 (Pan = 1.0)
        let right_only = AudioMixer::apply_track_pan_and_volume(sample, 1.0, 1.0, false);
        assert!(right_only[0].abs() < 0.01);
        assert!((right_only[1] - 1.0).abs() < 0.01);

        // 4. 静音
        let muted = AudioMixer::apply_track_pan_and_volume(sample, 1.0, 0.0, true);
        assert_eq!(muted, [0.0, 0.0]);
    }

    #[test]
    fn test_three_band_eq_preset_and_band_update() {
        let mut eq = ThreeBandEq::default();
        assert!(!eq.is_active());

        eq.apply_preset(EqPreset::Podcast);
        assert!(eq.is_active());
        assert!(eq.low.gain_db < 0.0);
        assert!(eq.mid.gain_db > 0.0);

        eq.set_band(EqBandSelector::High, 9_200.0, 3.5, 0.8);
        assert_eq!(eq.high.frequency_hz, 9_200.0);
        assert_eq!(eq.high.gain_db, 3.5);
        assert!((eq.high.q - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_three_band_eq_process_stereo_buffer() {
        let sample_rate = 48_000u32;
        let mut samples = Vec::with_capacity(4_096);
        for i in 0..4_096 {
            let t = i as f32 / sample_rate as f32;
            let v = (2.0 * std::f32::consts::PI * 1_800.0 * t).sin() * 0.2;
            samples.push([v, v]);
        }

        let before = stereo_rms(&samples);
        let mut eq = ThreeBandEq::default();
        eq.set_band(EqBandSelector::Mid, 1_800.0, 6.0, 1.0);
        eq.process_stereo_buffer(&mut samples, sample_rate);
        let after = stereo_rms(&samples);

        assert!(after > before * 1.2);
    }

    #[test]
    fn test_loudness_normalization_gain_and_apply() {
        let mut samples = vec![[0.1, 0.1]; 4_800];
        let measured = AudioMixer::estimate_integrated_lufs(&samples);
        assert!((measured + 20.691).abs() < 0.05);

        let state = LoudnessNormalization::from_samples(LoudnessTarget::StreamingMinus14, &samples);
        assert!(state.enabled);
        assert!((state.gain_db - 6.691).abs() < 0.1);

        AudioMixer::apply_gain_db(&mut samples, state.gain_db);
        let normalized = AudioMixer::estimate_integrated_lufs(&samples);
        assert!((normalized - LoudnessTarget::StreamingMinus14.target_lufs()).abs() < 0.2);
    }

    #[test]
    fn test_track_audio_processor_preview_gain_multiplier() {
        let mut processor = TrackAudioProcessor::default();
        processor.apply_eq_preset(EqPreset::Brightness);
        let state = processor.apply_loudness_target(LoudnessTarget::BroadcastMinus23, -18.0);
        assert!(state.gain_db < 0.0);
        assert!(processor.preview_gain_multiplier() < 1.0);
        assert!(TrackAudioProcessor::estimate_track_input_lufs(1.0, 1).is_finite());
    }
}
