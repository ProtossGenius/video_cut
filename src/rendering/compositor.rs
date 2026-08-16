use crate::timeline::{ClipId, TrackId};
use serde::{Deserialize, Serialize};

/// 单个切片在某一时刻参与画面合成的图层状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RenderLayer {
    pub track_id: TrackId,
    pub clip_id: ClipId,
    /// Z 轴图层高度（数值越大越在最上层）
    pub z_index: i32,
    /// 画面中心或锚点位置 (x, y)，单位为像素或百分比坐标
    pub position: (f32, f32),
    /// 水平与垂直缩放比例 (scale_x, scale_y)
    pub scale: (f32, f32),
    /// 旋转角度（度）
    pub rotation_deg: f32,
    /// 图层不透明度 (0.0 ~ 1.0)
    pub opacity: f32,
    /// 摄像机/蒙版裁剪矩形 [min_x, min_y, max_x, max_y]
    pub crop_rect: Option<[f32; 4]>,
    /// 是否可见
    pub visible: bool,
}

impl Default for RenderLayer {
    fn default() -> Self {
        Self {
            track_id: TrackId(0),
            clip_id: ClipId(0),
            z_index: 0,
            position: (0.0, 0.0),
            scale: (1.0, 1.0),
            rotation_deg: 0.0,
            opacity: 1.0,
            crop_rect: None,
            visible: true,
        }
    }
}

/// 摄像机视口配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CameraViewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub zoom: f32,
}

impl Default for CameraViewport {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
            zoom: 1.0,
        }
    }
}

/// 多轨道 GPU 画面合成器
#[derive(Debug, Clone, Default)]
pub struct VideoCompositor {
    pub viewport: CameraViewport,
    pub canvas_width: u32,
    pub canvas_height: u32,
}

impl VideoCompositor {
    pub fn new(canvas_width: u32, canvas_height: u32) -> Self {
        Self {
            viewport: CameraViewport {
                x: 0.0,
                y: 0.0,
                width: canvas_width as f32,
                height: canvas_height as f32,
                zoom: 1.0,
            },
            canvas_width,
            canvas_height,
        }
    }

    /// 按照画家算法（Painter's Algorithm 从后向前）对激活图层进行排序
    /// 排序规则：先按 z_index 从小到大，若相同则按 track_id 从低到高
    pub fn sort_layers_for_render(&self, mut layers: Vec<RenderLayer>) -> Vec<RenderLayer> {
        layers.retain(|l| l.visible && l.opacity > 0.0);
        layers.sort_by(|a, b| {
            a.z_index
                .cmp(&b.z_index)
                .then_with(|| a.track_id.0.cmp(&b.track_id.0))
        });
        layers
    }

    /// 计算在当前摄像机视口变换下的图层最终渲染变换矩阵/几何包围盒
    /// 返回渲染目标矩形 (left, top, width, height)
    pub fn compute_screen_bounds(
        &self,
        layer: &RenderLayer,
        original_w: f32,
        original_h: f32,
    ) -> [f32; 4] {
        let scaled_w = original_w * layer.scale.0 * self.viewport.zoom;
        let scaled_h = original_h * layer.scale.1 * self.viewport.zoom;

        // 视口相对偏移计算
        let center_x = (layer.position.0 - self.viewport.x) * self.viewport.zoom
            + (self.canvas_width as f32 / 2.0);
        let center_y = (layer.position.1 - self.viewport.y) * self.viewport.zoom
            + (self.canvas_height as f32 / 2.0);

        let left = center_x - scaled_w / 2.0;
        let top = center_y - scaled_h / 2.0;

        [left, top, scaled_w, scaled_h]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compositor_layer_sorting_by_z_index_and_track() {
        let compositor = VideoCompositor::new(1920, 1080);

        let layer1 = RenderLayer {
            track_id: TrackId(1),
            clip_id: ClipId(10),
            z_index: 0,
            ..Default::default()
        };
        let layer2 = RenderLayer {
            track_id: TrackId(2),
            clip_id: ClipId(20),
            z_index: 10, // 更高，应该画在上方
            ..Default::default()
        };
        let layer3 = RenderLayer {
            track_id: TrackId(3),
            clip_id: ClipId(30),
            z_index: -5, // 处于底层背景
            ..Default::default()
        };
        let hidden_layer = RenderLayer {
            track_id: TrackId(1),
            clip_id: ClipId(40),
            visible: false,
            ..Default::default()
        };

        let sorted = compositor.sort_layers_for_render(vec![layer1, layer2, layer3, hidden_layer]);
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].clip_id, ClipId(30)); // z-index: -5
        assert_eq!(sorted[1].clip_id, ClipId(10)); // z-index: 0
        assert_eq!(sorted[2].clip_id, ClipId(20)); // z-index: 10
    }

    #[test]
    fn test_compositor_screen_bounds_with_zoom() {
        let mut compositor = VideoCompositor::new(1920, 1080);
        compositor.viewport.zoom = 2.0; // 放大 2 倍

        let layer = RenderLayer {
            position: (0.0, 0.0),
            scale: (1.0, 1.0),
            ..Default::default()
        };

        let bounds = compositor.compute_screen_bounds(&layer, 400.0, 300.0);
        // 放大后宽 800，高 600，居中于 (960, 540)
        assert_eq!(bounds[2], 800.0);
        assert_eq!(bounds[3], 600.0);
        assert_eq!(bounds[0], 960.0 - 400.0);
        assert_eq!(bounds[1], 540.0 - 300.0);
    }
}
