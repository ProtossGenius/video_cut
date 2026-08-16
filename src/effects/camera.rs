use super::animation::CoordVal;
use crate::timeline::FrameTime;
use serde::{Deserialize, Serialize};

/// 摄像机聚焦矩形区域
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraRect {
    pub x: CoordVal,
    pub y: CoordVal,
    pub width: CoordVal,
    pub height: CoordVal,
}

impl CameraRect {
    pub fn full() -> Self {
        Self {
            x: CoordVal::Percent(0.0),
            y: CoordVal::Percent(0.0),
            width: CoordVal::Percent(1.0),
            height: CoordVal::Percent(1.0),
        }
    }

    pub fn to_pixel_rect(&self, canvas_w: f32, canvas_h: f32) -> (f32, f32, f32, f32) {
        let x = self.x.to_pixel(canvas_w);
        let y = self.y.to_pixel(canvas_h);
        let w = self.width.to_pixel(canvas_w);
        let h = self.height.to_pixel(canvas_h);
        (x, y, w, h)
    }
}

/// 摄像机关键帧
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraKeyframe {
    pub time_offset: FrameTime,
    pub focus_rect: CameraRect,
    pub smooth_interpolate: bool,
}

/// 摄像机序列
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CameraSequence {
    pub keyframes: Vec<CameraKeyframe>,
}

impl CameraSequence {
    pub fn new() -> Self {
        Self {
            keyframes: Vec::new(),
        }
    }

    pub fn add_keyframe(&mut self, keyframe: CameraKeyframe) {
        self.keyframes.push(keyframe);
    }

    /// 计算指定时间的摄像机视口矩形
    pub fn evaluate_at(
        &self,
        time: FrameTime,
        canvas_w: f32,
        canvas_h: f32,
    ) -> (f32, f32, f32, f32) {
        if self.keyframes.is_empty() {
            return (0.0, 0.0, canvas_w, canvas_h);
        }

        if time <= self.keyframes[0].time_offset {
            return self.keyframes[0]
                .focus_rect
                .to_pixel_rect(canvas_w, canvas_h);
        }

        let last = self.keyframes.last().unwrap();
        if time >= last.time_offset {
            return last.focus_rect.to_pixel_rect(canvas_w, canvas_h);
        }

        for i in 0..self.keyframes.len() - 1 {
            let kf0 = &self.keyframes[i];
            let kf1 = &self.keyframes[i + 1];

            if time >= kf0.time_offset && time <= kf1.time_offset {
                let r0 = kf0.focus_rect.to_pixel_rect(canvas_w, canvas_h);
                let r1 = kf1.focus_rect.to_pixel_rect(canvas_w, canvas_h);

                if !kf0.smooth_interpolate {
                    return r0;
                }

                let t0 = kf0.time_offset.0 as f32;
                let t1 = kf1.time_offset.0 as f32;
                let factor = if (t1 - t0).abs() < f32::EPSILON {
                    1.0
                } else {
                    ((time.0 as f32) - t0) / (t1 - t0)
                };

                return (
                    r0.0 + (r1.0 - r0.0) * factor,
                    r0.1 + (r1.1 - r0.1) * factor,
                    r0.2 + (r1.2 - r0.2) * factor,
                    r0.3 + (r1.3 - r0.3) * factor,
                );
            }
        }

        (0.0, 0.0, canvas_w, canvas_h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_rect_full() {
        let cam = CameraSequence::new();
        let (x, y, w, h) = cam.evaluate_at(FrameTime(0), 1920.0, 1080.0);
        assert_eq!((x, y, w, h), (0.0, 0.0, 1920.0, 1080.0));
    }

    #[test]
    fn test_camera_focus_zoom() {
        let mut cam = CameraSequence::new();
        cam.add_keyframe(CameraKeyframe {
            time_offset: FrameTime(0),
            focus_rect: CameraRect::full(),
            smooth_interpolate: true,
        });
        cam.add_keyframe(CameraKeyframe {
            time_offset: FrameTime(2_000_000), // 2s
            focus_rect: CameraRect {
                x: CoordVal::Percent(0.25),
                y: CoordVal::Percent(0.25),
                width: CoordVal::Percent(0.5),
                height: CoordVal::Percent(0.5),
            },
            smooth_interpolate: true,
        });

        // 1s 中点
        let (_x, _y, w, h) = cam.evaluate_at(FrameTime(1_000_000), 1920.0, 1080.0);
        assert_eq!(w, 1920.0 * 0.75);
        assert_eq!(h, 1080.0 * 0.75);
    }
}
