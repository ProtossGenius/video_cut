use serde::{Deserialize, Serialize};

/// 颜色分级与滤镜预设
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LutPreset {
    None,
    /// 鲜艳增强
    Vibrant,
    /// 暖调电影感
    CinematicWarm,
    /// 青橙电影调色 (Teal & Orange)
    TealOrange,
    /// 经典黑白单色 (Monochrome / B&W)
    Monochrome,
    /// 复古胶片 (Vintage Film)
    Vintage,
    /// 冷冽科幻 (Cool Sci-Fi)
    CoolSciFi,
}

impl LutPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::None => "原色 (None)",
            Self::Vibrant => "鲜艳 (Vibrant)",
            Self::CinematicWarm => "暖调电影 (Cinematic Warm)",
            Self::TealOrange => "青橙色调 (Teal & Orange)",
            Self::Monochrome => "经典黑白 (Monochrome B&W)",
            Self::Vintage => "复古胶片 (Vintage Film)",
            Self::CoolSciFi => "冷色科幻 (Cool Sci-Fi)",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().replace(['_', '-'], "").as_str() {
            "none" | "default" | "off" | "0" => Some(Self::None),
            "vibrant" | "vivid" => Some(Self::Vibrant),
            "cinematic" | "cinematicwarm" | "warm" => Some(Self::CinematicWarm),
            "tealorange" | "teal" | "orange" => Some(Self::TealOrange),
            "monochrome" | "mono" | "bw" | "blackwhite" => Some(Self::Monochrome),
            "vintage" | "film" | "retro" => Some(Self::Vintage),
            "cool" | "scifi" | "coolscifi" => Some(Self::CoolSciFi),
            _ => None,
        }
    }
}

/// 颜色分级参数
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColorGradingParams {
    /// 亮度增益: -1.0 ~ 1.0 (默认 0.0)
    pub brightness: f32,
    /// 对比度乘数: 0.0 ~ 3.0 (默认 1.0)
    pub contrast: f32,
    /// 饱和度乘数: 0.0 ~ 3.0 (默认 1.0, 0.0 为灰度)
    pub saturation: f32,
    /// 色温调节: -1.0 (偏冷蓝) ~ 1.0 (偏暖橙) (默认 0.0)
    pub temperature: f32,
    /// LUT 滤镜预设
    pub lut_preset: LutPreset,
}

impl Default for ColorGradingParams {
    fn default() -> Self {
        Self {
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            temperature: 0.0,
            lut_preset: LutPreset::None,
        }
    }
}

impl ColorGradingParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_active(&self) -> bool {
        self.brightness.abs() > 0.001
            || (self.contrast - 1.0).abs() > 0.001
            || (self.saturation - 1.0).abs() > 0.001
            || self.temperature.abs() > 0.001
            || self.lut_preset != LutPreset::None
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// 应用颜色分级算法到单个 RGBA 像素 (r, g, b in 0.0 ~ 1.0)
    pub fn apply_pixel(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        let mut cr = r;
        let mut cg = g;
        let mut cb = b;

        // 1. 亮度 Brightness
        if self.brightness != 0.0 {
            cr += self.brightness;
            cg += self.brightness;
            cb += self.brightness;
        }

        // 2. 对比度 Contrast (以 0.5 为基准点)
        if self.contrast != 1.0 {
            cr = (cr - 0.5) * self.contrast + 0.5;
            cg = (cg - 0.5) * self.contrast + 0.5;
            cb = (cb - 0.5) * self.contrast + 0.5;
        }

        // 3. 饱和度 Saturation (Rec.709 亮度权重)
        if self.saturation != 1.0 {
            let luma = 0.2126 * cr + 0.7152 * cg + 0.0722 * cb;
            cr = luma + (cr - luma) * self.saturation;
            cg = luma + (cg - luma) * self.saturation;
            cb = luma + (cb - luma) * self.saturation;
        }

        // 4. 色温 Temperature
        if self.temperature != 0.0 {
            if self.temperature > 0.0 {
                // 暖色：增加红/微增绿，减少蓝
                cr += self.temperature * 0.15;
                cg += self.temperature * 0.05;
                cb -= self.temperature * 0.15;
            } else {
                // 冷色：增加蓝，减少红
                let t = -self.temperature;
                cr -= t * 0.15;
                cb += t * 0.15;
            }
        }

        // 5. LUT 预设滤镜映射
        match self.lut_preset {
            LutPreset::None => {}
            LutPreset::Vibrant => {
                let luma = 0.2126 * cr + 0.7152 * cg + 0.0722 * cb;
                cr = luma + (cr - luma) * 1.35;
                cg = luma + (cg - luma) * 1.35;
                cb = luma + (cb - luma) * 1.35;
                cr = (cr - 0.5) * 1.1 + 0.5;
                cg = (cg - 0.5) * 1.1 + 0.5;
                cb = (cb - 0.5) * 1.1 + 0.5;
            }
            LutPreset::CinematicWarm => {
                cr = cr * 1.1 + 0.03;
                cg = cg * 1.02 + 0.01;
                cb *= 0.88;
                cr = (cr - 0.5) * 1.15 + 0.5;
                cg = (cg - 0.5) * 1.15 + 0.5;
                cb = (cb - 0.5) * 1.15 + 0.5;
            }
            LutPreset::TealOrange => {
                // 阴影偏青，高光偏橙
                let luma = 0.2126 * cr + 0.7152 * cg + 0.0722 * cb;
                if luma > 0.5 {
                    cr += 0.08 * (luma - 0.5) * 2.0;
                    cg += 0.03 * (luma - 0.5) * 2.0;
                } else {
                    cg += 0.05 * (0.5 - luma) * 2.0;
                    cb += 0.09 * (0.5 - luma) * 2.0;
                }
            }
            LutPreset::Monochrome => {
                let luma = 0.2126 * cr + 0.7152 * cg + 0.0722 * cb;
                cr = luma;
                cg = luma;
                cb = luma;
            }
            LutPreset::Vintage => {
                cr = cr * 1.05 + 0.04;
                cg = cg * 0.95 + 0.02;
                cb = cb * 0.85 + 0.02;
                let luma = 0.2126 * cr + 0.7152 * cg + 0.0722 * cb;
                cr = luma + (cr - luma) * 0.8;
                cg = luma + (cg - luma) * 0.8;
                cb = luma + (cb - luma) * 0.8;
            }
            LutPreset::CoolSciFi => {
                cr *= 0.85;
                cg = cg * 0.98 + 0.02;
                cb = cb * 1.15 + 0.05;
                cr = (cr - 0.5) * 1.2 + 0.5;
                cg = (cg - 0.5) * 1.2 + 0.5;
                cb = (cb - 0.5) * 1.2 + 0.5;
            }
        }

        (cr.clamp(0.0, 1.0), cg.clamp(0.0, 1.0), cb.clamp(0.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_grading_brightness_and_contrast() {
        let mut params = ColorGradingParams::new();
        params.brightness = 0.1;
        params.contrast = 1.2;

        let (r, g, b) = params.apply_pixel(0.5, 0.5, 0.5);
        // 0.5 + 0.1 = 0.6; (0.6 - 0.5) * 1.2 + 0.5 = 0.62
        assert!((r - 0.62).abs() < 0.001);
        assert!((g - 0.62).abs() < 0.001);
        assert!((b - 0.62).abs() < 0.001);
    }

    #[test]
    fn test_color_grading_monochrome_lut() {
        let mut params = ColorGradingParams::new();
        params.lut_preset = LutPreset::Monochrome;

        let (r, g, b) = params.apply_pixel(1.0, 0.0, 0.0);
        // Rec.709 红色亮度: 0.2126
        assert!((r - 0.2126).abs() < 0.001);
        assert_eq!(r, g);
        assert_eq!(g, b);
    }

    #[test]
    fn test_lut_preset_from_str_loose() {
        assert_eq!(LutPreset::from_str_loose("teal_orange"), Some(LutPreset::TealOrange));
        assert_eq!(LutPreset::from_str_loose("BW"), Some(LutPreset::Monochrome));
        assert_eq!(LutPreset::from_str_loose("cinematic"), Some(LutPreset::CinematicWarm));
        assert_eq!(LutPreset::from_str_loose("none"), Some(LutPreset::None));
        assert_eq!(LutPreset::from_str_loose("unknown"), None);
    }
}
