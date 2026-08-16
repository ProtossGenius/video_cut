# 切片管理 (Clip)

切片是轨道的原子构成要素，可以是一段视频、一份音频或一张图片。

## 结构定义
```rust
pub struct Clip {
    pub id: ClipId,
    pub name: String,              // 命名必须如标识符，同轨道不允许重名
    pub source: AssetId,           // 原始媒体文件引用
    pub source_in: FrameTime,      // 素材在原始文件中的起始时间
    pub source_out: FrameTime,     // 素材在原始文件中的结束时间
    pub timeline_start: FrameTime, // 素材在时间线上的摆放起始时间
    pub properties: Vec<Box<dyn EffectProperty>>, // 挂载的特效/属性
    pub anchors: HashMap<String, AnchorPoint>,    // 局部锚点
}
```
*注：切片长度等于 `(source_out - source_in) * 速度倍数`。*

## 核心操作
所有这些操作必须生成 `Command` 以存入 Undo/Redo 栈：
- **Split** (按 `s` 默认绑定): 将选定切片在当前时间点分为两段。
- **Merge**: 合并选中的处于相邻状态的切片组。
- **MergeCut**: 在选中的区域边界两端执行切断，并将中间区域合并成一个独立切片。
