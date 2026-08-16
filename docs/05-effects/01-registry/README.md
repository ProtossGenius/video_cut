# 注册机制

保证引擎可扩展的核心机制。不允许对某个特定特效写死硬编码的 `if-else` 分支。

## 静态注册表
在 Rust 端，使用 `inventory` crate 提供无侵入式插件注册：
```rust
pub trait ClipProperty: Send + Sync {
    fn name(&self) -> &str;
    fn process(&self, time: RelativeTime, frame: &mut RenderContext);
}

// 通过宏搜集所有实现类，启动时放入全局 HashMap 表中。
inventory::collect!(&'static dyn ClipPropertyFactory);
```

## 动态添加
脚本系统内，Lua 可以直接实例化注册表中定义的特效工厂，或者利用 Lua 回调动态构建新的效果（如纯通过坐标变换方程生成震动属性）。所有的特效必须拥有对应的生命周期管理，同轨道不能有重名组件的冲突。
