use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

use super::animation::AnimationSequence;
use super::camera::CameraSequence;
use super::follow::FollowProperty;
use super::mask::MaskEffect;
use super::speed::SpeedProperty;

/// 切片拥有的所有特效与属性集合
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ClipProperties {
    /// 切片唯一名称（同一轨道内唯一）
    pub name: String,
    /// 是否锁定在当前时间点（禁止移动与切割碰撞）
    pub is_locked: bool,
    /// 渲染图层高度 (Z-Index)
    pub z_index: i32,
    /// 关键帧动画序列
    pub animation: Option<AnimationSequence>,
    /// 摄像机运镜序列
    pub camera: Option<CameraSequence>,
    /// 遮罩 / 颜色抠像
    pub mask: Option<MaskEffect>,
    /// 变速属性
    pub speed: SpeedProperty,
    /// 跟随属性
    pub follow: Option<FollowProperty>,
}

impl ClipProperties {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            is_locked: false,
            z_index: 0,
            animation: None,
            camera: None,
            mask: None,
            speed: SpeedProperty::default(),
            follow: None,
        }
    }
}

/// 特效元数据描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectDescriptor {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
}

/// 全局特效注册表
pub struct EffectRegistry {
    effects: RwLock<HashMap<&'static str, EffectDescriptor>>,
}

impl EffectRegistry {
    pub fn global() -> &'static Self {
        static REGISTRY: std::sync::OnceLock<EffectRegistry> = std::sync::OnceLock::new();
        REGISTRY.get_or_init(|| {
            let reg = EffectRegistry {
                effects: RwLock::new(HashMap::new()),
            };
            reg.register_builtins();
            reg
        })
    }

    fn register_builtins(&self) {
        self.register(EffectDescriptor {
            id: "animation",
            name: "关键帧动画",
            category: "Transform",
            description: "通过时间定义位移、缩放、旋转与透明度关键帧",
        });
        self.register(EffectDescriptor {
            id: "camera",
            name: "摄像机运镜",
            category: "View",
            description: "动态缩放与区域聚焦",
        });
        self.register(EffectDescriptor {
            id: "mask",
            name: "图形与颜色遮罩",
            category: "Compositing",
            description: "纯色绿幕抠像与 Alpha 蒙版",
        });
        self.register(EffectDescriptor {
            id: "speed",
            name: "播放速度调整",
            category: "Time",
            description: "加速或减速播放并自适应拉伸切片",
        });
        self.register(EffectDescriptor {
            id: "follow",
            name: "切片相对跟随",
            category: "Timeline",
            description: "跟随关联切片联动位移",
        });
    }

    pub fn register(&self, desc: EffectDescriptor) {
        if let Ok(mut lock) = self.effects.write() {
            lock.insert(desc.id, desc);
        }
    }

    pub fn get(&self, id: &str) -> Option<EffectDescriptor> {
        self.effects.read().ok()?.get(id).cloned()
    }

    pub fn all(&self) -> Vec<EffectDescriptor> {
        self.effects
            .read()
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_builtins() {
        let reg = EffectRegistry::global();
        assert!(reg.get("animation").is_some());
        assert!(reg.get("camera").is_some());
        assert!(reg.get("mask").is_some());
        assert!(reg.get("speed").is_some());
        assert_eq!(reg.all().len(), 5);
    }
}
