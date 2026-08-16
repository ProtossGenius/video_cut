use serde::{Deserialize, Serialize};

/// 视频色彩空间标准
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ColorStandard {
    #[default]
    Bt709, // 高清 (HD / 1080p / 4K)
    Bt601, // 标清 (SD / DVD)
}

/// YUV 到 RGBA 转换器（提供 CPU 转换算法与 GPU WGSL 着色器）
pub struct YuvConverter;

impl YuvConverter {
    /// CPU 像素转换：单像素 YUV -> RGBA (根据 BT.709 或 BT.601 标准)
    pub fn yuv_to_rgb(y: u8, u: u8, v: u8, standard: ColorStandard) -> (u8, u8, u8) {
        let y_f = y as f32;
        let u_f = u as f32 - 128.0;
        let v_f = v as f32 - 128.0;

        let (r, g, b) = match standard {
            ColorStandard::Bt709 => {
                let r = y_f + 1.5748 * v_f;
                let g = y_f - 0.1873 * u_f - 0.4681 * v_f;
                let b = y_f + 1.8556 * u_f;
                (r, g, b)
            }
            ColorStandard::Bt601 => {
                let r = y_f + 1.402 * v_f;
                let g = y_f - 0.344136 * u_f - 0.714136 * v_f;
                let b = y_f + 1.772 * u_f;
                (r, g, b)
            }
        };

        (
            r.clamp(0.0, 255.0).round() as u8,
            g.clamp(0.0, 255.0).round() as u8,
            b.clamp(0.0, 255.0).round() as u8,
        )
    }

    /// 将整个 I420 (YUV420P) 图像帧转换为 RGBA 像素数组
    pub fn i420_to_rgba(
        y_plane: &[u8],
        u_plane: &[u8],
        v_plane: &[u8],
        width: usize,
        height: usize,
        standard: ColorStandard,
    ) -> Vec<u8> {
        let mut rgba = vec![255u8; width * height * 4];
        let uv_width = width / 2;

        for row in 0..height {
            for col in 0..width {
                let y_idx = row * width + col;
                let uv_idx = (row / 2) * uv_width + (col / 2);

                let y = y_plane.get(y_idx).copied().unwrap_or(0);
                let u = u_plane.get(uv_idx).copied().unwrap_or(128);
                let v = v_plane.get(uv_idx).copied().unwrap_or(128);

                let (r, g, b) = Self::yuv_to_rgb(y, u, v, standard);

                let out_idx = y_idx * 4;
                rgba[out_idx] = r;
                rgba[out_idx + 1] = g;
                rgba[out_idx + 2] = b;
                rgba[out_idx + 3] = 255;
            }
        }

        rgba
    }

    /// 生成用于 wgpu GPU 硬件加速转换的 WGSL 片元着色器代码
    pub fn wgsl_shader() -> &'static str {
        r#"
        struct VertexOutput {
            @builtin(position) clip_position: vec4<f32>,
            @location(0) uv: vec2<f32>,
        };

        @group(0) @binding(0) var t_y: texture_2d<f32>;
        @group(0) @binding(1) var s_y: sampler;
        @group(0) @binding(2) var t_uv: texture_2d<f32>;
        @group(0) @binding(3) var s_uv: sampler;

        @fragment
        fn fs_main_bt709(in: VertexOutput) -> @location(0) vec4<f32> {
            let y = textureSample(t_y, s_y, in.uv).r;
            let uv = textureSample(t_uv, s_uv, in.uv).rg - vec2<f32>(0.5, 0.5);
            let u = uv.r;
            let v = uv.g;

            let r = y + 1.5748 * v;
            let g = y - 0.1873 * u - 0.4681 * v;
            let b = y + 1.8556 * u;

            return vec4<f32>(clamp(r, 0.0, 1.0), clamp(g, 0.0, 1.0), clamp(b, 0.0, 1.0), 1.0);
        }
        "#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yuv_pure_black_and_white() {
        // 纯黑 (Y=0, U=128, V=128)
        let (r, g, b) = YuvConverter::yuv_to_rgb(0, 128, 128, ColorStandard::Bt709);
        assert_eq!((r, g, b), (0, 0, 0));

        // 纯白 (Y=255, U=128, V=128)
        let (r, g, b) = YuvConverter::yuv_to_rgb(255, 128, 128, ColorStandard::Bt709);
        assert_eq!((r, g, b), (255, 255, 255));
    }

    #[test]
    fn test_i420_to_rgba_buffer() {
        let width = 2;
        let height = 2;
        let y_plane = vec![255, 255, 0, 0];
        let u_plane = vec![128];
        let v_plane = vec![128];

        let rgba = YuvConverter::i420_to_rgba(
            &y_plane,
            &u_plane,
            &v_plane,
            width,
            height,
            ColorStandard::Bt709,
        );
        assert_eq!(rgba.len(), 16); // 2x2 * 4
                                    // 前两个像素为白色 (255, 255, 255, 255)
        assert_eq!(&rgba[0..4], &[255, 255, 255, 255]);
        assert_eq!(&rgba[4..8], &[255, 255, 255, 255]);
        // 后两个像素为黑色 (0, 0, 0, 255)
        assert_eq!(&rgba[8..12], &[0, 0, 0, 255]);
        assert_eq!(&rgba[12..16], &[0, 0, 0, 255]);
    }

    #[test]
    fn test_wgsl_shader_not_empty() {
        assert!(YuvConverter::wgsl_shader().contains("fs_main_bt709"));
    }
}
