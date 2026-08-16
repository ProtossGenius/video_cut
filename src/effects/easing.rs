use serde::{Deserialize, Serialize};

/// 缓动曲线类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EasingType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    BounceOut,
    ElasticOut,
    CubicBezier,
}

impl Default for EasingType {
    fn default() -> Self {
        Self::EaseInOut
    }
}

/// 缓动曲线与三次方贝塞尔求解器
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EasingCurve {
    pub easing_type: EasingType,
    /// 贝塞尔控制点 P1 [x1, y1] (0.0 ~ 1.0)
    pub p1: [f32; 2],
    /// 贝塞尔控制点 P2 [x2, y2] (0.0 ~ 1.0)
    pub p2: [f32; 2],
}

impl Default for EasingCurve {
    fn default() -> Self {
        Self::ease_in_out()
    }
}

impl EasingCurve {
    pub fn linear() -> Self {
        Self {
            easing_type: EasingType::Linear,
            p1: [0.0, 0.0],
            p2: [1.0, 1.0],
        }
    }

    pub fn ease_in() -> Self {
        Self {
            easing_type: EasingType::EaseIn,
            p1: [0.42, 0.0],
            p2: [1.0, 1.0],
        }
    }

    pub fn ease_out() -> Self {
        Self {
            easing_type: EasingType::EaseOut,
            p1: [0.0, 0.0],
            p2: [0.58, 1.0],
        }
    }

    pub fn ease_in_out() -> Self {
        Self {
            easing_type: EasingType::EaseInOut,
            p1: [0.42, 0.0],
            p2: [0.58, 1.0],
        }
    }

    pub fn bounce_out() -> Self {
        Self {
            easing_type: EasingType::BounceOut,
            p1: [0.0, 0.0],
            p2: [1.0, 1.0],
        }
    }

    pub fn elastic_out() -> Self {
        Self {
            easing_type: EasingType::ElasticOut,
            p1: [0.0, 0.0],
            p2: [1.0, 1.0],
        }
    }

    pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self {
            easing_type: EasingType::CubicBezier,
            p1: [x1.clamp(0.0, 1.0), y1],
            p2: [x2.clamp(0.0, 1.0), y2],
        }
    }

    pub fn name(&self) -> &'static str {
        match self.easing_type {
            EasingType::Linear => "匀速直线 (Linear)",
            EasingType::EaseIn => "平滑加速 (Ease In)",
            EasingType::EaseOut => "平滑减速 (Ease Out)",
            EasingType::EaseInOut => "平滑缓入缓出 (Ease In Out)",
            EasingType::BounceOut => "弹力弹跳 (Bounce Out)",
            EasingType::ElasticOut => "弹性阻尼 (Elastic Out)",
            EasingType::CubicBezier => "自定义贝塞尔 (Cubic Bezier)",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().replace(['_', '-'], "").as_str() {
            "linear" | "none" => Some(Self::linear()),
            "easein" | "in" => Some(Self::ease_in()),
            "easeout" | "out" => Some(Self::ease_out()),
            "easeinout" | "inout" | "smooth" => Some(Self::ease_in_out()),
            "bounce" | "bounceout" => Some(Self::bounce_out()),
            "elastic" | "elasticout" | "spring" => Some(Self::elastic_out()),
            _ => None,
        }
    }

    /// 根据当前时间进度 t (0.0 ~ 1.0) 计算缓动输出值 (通常在 0.0 ~ 1.0，但弹性/贝塞尔可能超调)
    pub fn evaluate(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);

        match self.easing_type {
            EasingType::Linear => t,
            EasingType::EaseIn => t * t,
            EasingType::EaseOut => t * (2.0 - t),
            EasingType::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            EasingType::BounceOut => {
                let n1 = 7.5625;
                let d1 = 2.75;
                let mut x = t;
                if x < 1.0 / d1 {
                    n1 * x * x
                } else if x < 2.0 / d1 {
                    x -= 1.5 / d1;
                    n1 * x * x + 0.75
                } else if x < 2.5 / d1 {
                    x -= 2.25 / d1;
                    n1 * x * x + 0.9375
                } else {
                    x -= 2.625 / d1;
                    n1 * x * x + 0.984375
                }
            }
            EasingType::ElasticOut => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    let c4 = (2.0 * std::f32::consts::PI) / 3.0;
                    (-10.0 * t).exp2() * ((t * 10.0 - 0.75) * c4).sin() + 1.0
                }
            }
            EasingType::CubicBezier => {
                // 三次方贝塞尔曲线牛顿迭代法求解 x(u) = t 对应的 u，并代入 y(u)
                let x1 = self.p1[0];
                let y1 = self.p1[1];
                let x2 = self.p2[0];
                let y2 = self.p2[1];

                let mut u = t;
                for _ in 0..8 {
                    let current_x = 3.0 * (1.0 - u) * (1.0 - u) * u * x1
                        + 3.0 * (1.0 - u) * u * u * x2
                        + u * u * u;
                    let dx = 3.0 * (1.0 - u) * (1.0 - u) * x1
                        + 6.0 * (1.0 - u) * u * (x2 - x1)
                        + 3.0 * u * u * (1.0 - x2);
                    let diff = current_x - t;
                    if diff.abs() < 1e-4 || dx.abs() < 1e-6 {
                        break;
                    }
                    u -= diff / dx;
                    u = u.clamp(0.0, 1.0);
                }

                3.0 * (1.0 - u) * (1.0 - u) * u * y1
                    + 3.0 * (1.0 - u) * u * u * y2
                    + u * u * u
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easing_curve_evaluations() {
        let linear = EasingCurve::linear();
        assert_eq!(linear.evaluate(0.0), 0.0);
        assert_eq!(linear.evaluate(0.5), 0.5);
        assert_eq!(linear.evaluate(1.0), 1.0);

        let ease_in_out = EasingCurve::ease_in_out();
        assert_eq!(ease_in_out.evaluate(0.0), 0.0);
        assert_eq!(ease_in_out.evaluate(0.5), 0.5);
        assert_eq!(ease_in_out.evaluate(1.0), 1.0);

        let bounce = EasingCurve::bounce_out();
        assert_eq!(bounce.evaluate(0.0), 0.0);
        assert!((bounce.evaluate(1.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_cubic_bezier_custom() {
        let bezier = EasingCurve::cubic_bezier(0.25, 0.1, 0.25, 1.0);
        assert_eq!(bezier.evaluate(0.0), 0.0);
        assert!((bezier.evaluate(1.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_easing_from_str_loose() {
        assert_eq!(EasingCurve::from_str_loose("ease_in_out"), Some(EasingCurve::ease_in_out()));
        assert_eq!(EasingCurve::from_str_loose("bounce"), Some(EasingCurve::bounce_out()));
        assert_eq!(EasingCurve::from_str_loose("spring"), Some(EasingCurve::elastic_out()));
        assert_eq!(EasingCurve::from_str_loose("unknown"), None);
    }
}
