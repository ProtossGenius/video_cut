use serde::{Deserialize, Serialize};

/// 遮罩模式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MaskMode {
    /// 颜色抠像 / 纯色变透明 (如绿幕/蓝幕抠像)
    ChromaKey {
        /// 目标颜色 RGB (0~255)
        target_color: [u8; 3],
        /// 容差范围 (0.0 ~ 1.0)
        tolerance: f32,
        /// 边缘平滑过渡因子 (0.0 ~ 1.0)
        softness: f32,
    },
    /// 矩形窗口遮罩
    RectAlpha {
        /// 规范化矩形坐标 [x_min, y_min, x_max, y_max] (0.0 ~ 1.0)
        bounds: [f32; 4],
        /// 边缘羽化半径 (像素)
        feather: f32,
        /// 是否反转遮罩
        invert: bool,
    },
    /// 全局不透明度乘数 (0.0 ~ 1.0)
    GlobalOpacity(f32),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaskEffect {
    pub enabled: bool,
    pub mode: MaskMode,
}

impl Default for MaskEffect {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: MaskMode::GlobalOpacity(1.0),
        }
    }
}

impl MaskEffect {
    pub fn new_chroma_key(target_color: [u8; 3], tolerance: f32, softness: f32) -> Self {
        Self {
            enabled: true,
            mode: MaskMode::ChromaKey {
                target_color,
                tolerance: tolerance.clamp(0.0, 1.0),
                softness: softness.clamp(0.0, 1.0),
            },
        }
    }

    pub fn new_rect_mask(bounds: [f32; 4], feather: f32, invert: bool) -> Self {
        Self {
            enabled: true,
            mode: MaskMode::RectAlpha {
                bounds,
                feather: feather.max(0.0),
                invert,
            },
        }
    }

    pub fn new_global_opacity(opacity: f32) -> Self {
        Self {
            enabled: true,
            mode: MaskMode::GlobalOpacity(opacity.clamp(0.0, 1.0)),
        }
    }

    /// 在 CPU 上对 RGBA 图像缓冲区应用遮罩透明度运算
    pub fn apply_to_rgba(&self, rgba_buffer: &mut [u8], width: usize, height: usize) {
        if !self.enabled {
            return;
        }

        match &self.mode {
            MaskMode::GlobalOpacity(opacity) => {
                let factor = opacity.clamp(0.0, 1.0);
                for chunk in rgba_buffer.chunks_exact_mut(4) {
                    chunk[3] = ((chunk[3] as f32) * factor).round() as u8;
                }
            }
            MaskMode::ChromaKey {
                target_color,
                tolerance,
                softness,
            } => {
                let [tr, tg, tb] = *target_color;
                let max_dist = 441.67295_f32; // sqrt(255^2 * 3)
                let threshold = tolerance * max_dist;
                let soft_range = (softness * max_dist).max(1.0);

                for chunk in rgba_buffer.chunks_exact_mut(4) {
                    let r_diff = chunk[0] as f32 - tr as f32;
                    let g_diff = chunk[1] as f32 - tg as f32;
                    let b_diff = chunk[2] as f32 - tb as f32;
                    let dist = (r_diff * r_diff + g_diff * g_diff + b_diff * b_diff).sqrt();

                    if dist <= threshold {
                        // 完全在抠像容差内 -> 全透明
                        chunk[3] = 0;
                    } else if dist < threshold + soft_range {
                        // 处于边缘平滑过渡区
                        let alpha_factor = (dist - threshold) / soft_range;
                        chunk[3] = ((chunk[3] as f32) * alpha_factor).round() as u8;
                    }
                }
            }
            MaskMode::RectAlpha {
                bounds,
                feather,
                invert,
            } => {
                let [x_min, y_min, x_max, y_max] = *bounds;

                for y in 0..height {
                    for x in 0..width {
                        let idx = (y * width + x) * 4;
                        let norm_x = (x as f32 + 0.5) / width as f32;
                        let norm_y = (y as f32 + 0.5) / height as f32;

                        // 计算点到矩形边界的最短距离
                        let inside = norm_x >= x_min
                            && norm_x <= x_max
                            && norm_y >= y_min
                            && norm_y <= y_max;

                        let mut alpha_factor = if inside {
                            if *feather > 0.0 {
                                let dist_x = (norm_x - x_min).min(x_max - norm_x) * width as f32;
                                let dist_y = (norm_y - y_min).min(y_max - norm_y) * height as f32;
                                (dist_x.min(dist_y) / *feather).min(1.0)
                            } else {
                                1.0
                            }
                        } else {
                            0.0
                        };

                        if *invert {
                            alpha_factor = 1.0 - alpha_factor;
                        }

                        rgba_buffer[idx + 3] =
                            ((rgba_buffer[idx + 3] as f32) * alpha_factor).round() as u8;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_opacity_mask() {
        let mask = MaskEffect::new_global_opacity(0.5);
        let mut buffer = vec![255, 128, 64, 200];
        mask.apply_to_rgba(&mut buffer, 1, 1);
        assert_eq!(buffer[3], 100);
    }

    #[test]
    fn test_chroma_key_green_screen() {
        // 绿幕色 RGB(0, 255, 0)
        let mask = MaskEffect::new_chroma_key([0, 255, 0], 0.2, 0.1);

        // 纯绿像素 -> 应该变全透明 (Alpha = 0)
        let mut pure_green = vec![0, 255, 0, 255];
        mask.apply_to_rgba(&mut pure_green, 1, 1);
        assert_eq!(pure_green[3], 0);

        // 纯红像素 -> 不受绿幕影响 (Alpha = 255)
        let mut pure_red = vec![255, 0, 0, 255];
        mask.apply_to_rgba(&mut pure_red, 1, 1);
        assert_eq!(pure_red[3], 255);
    }

    #[test]
    fn test_rect_alpha_mask_and_inversion() {
        // 矩形占据左半边 [0.0, 0.0, 0.5, 1.0]
        let mask = MaskEffect::new_rect_mask([0.0, 0.0, 0.5, 1.0], 0.0, false);
        let mut buffer = vec![
            255, 255, 255, 255, // 左侧像素 (0, 0)
            255, 255, 255, 255, // 右侧像素 (1, 0)
        ];

        mask.apply_to_rgba(&mut buffer, 2, 1);
        assert_eq!(buffer[3], 255); // 左侧保留
        assert_eq!(buffer[7], 0); // 右侧裁剪为透明

        // 反转遮罩
        let mask_invert = MaskEffect::new_rect_mask([0.0, 0.0, 0.5, 1.0], 0.0, true);
        let mut buffer_inv = vec![
            255, 255, 255, 255, // 左侧像素 (0, 0)
            255, 255, 255, 255, // 右侧像素 (1, 0)
        ];
        mask_invert.apply_to_rgba(&mut buffer_inv, 2, 1);
        assert_eq!(buffer_inv[3], 0); // 左侧变透明
        assert_eq!(buffer_inv[7], 255); // 右侧保留
    }
}
