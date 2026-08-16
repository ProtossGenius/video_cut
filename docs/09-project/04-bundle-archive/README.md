# 全局项目打包归档与资产自包含导出 (Project Packaging & Self-Contained Archive Exporter)

解决多设备协作与工程迁移时“媒体文件丢失、路径断裂、脚本不一致”的痛点，将工程文件、引用的视频/音频素材、自定义 Lua 脚本及预计算波形打包为自包含的独立项目包。

## 归档包目录结构 (`.vcutpkg` / 独立工程包)
```text
my_video.vcutpkg/
├── manifest.json       # 项目归档元数据 (版本号、创建时间戳、资产清单与校验哈希)
├── project.vcut        # 核心工程状态 (包含轨道、切片、特效、关键帧与时间线)
├── assets/             # 自包含媒体素材仓库 (视频、音频、图片)
│   ├── video1.mp4
│   └── background.mp3
└── scripts/            # 关联的自定义 Lua 自动化剪辑脚本
    └── custom_intro.lua
```

## 核心接口 (`ProjectBundle`)
- `pack_bundle(bundle_dir, project, asset_paths)`：收集所有素材与工程，将资产自动拷贝至 `assets/` 并重映射路径，生成完整的自包含包与 `manifest.json`。
- `unpack_bundle(bundle_dir)`：读取 `manifest.json`，校验完整性并安全还原 `ProjectState`，无缝加载工程。

## 交互命令
- `:pack_project [dir]` / `:bundle [dir]`：将当前工程与其所有媒体资产打包归档为自包含目录。
- `:unpack_project <dir>` / `:open_bundle <dir>`：从自包含归档包中导入并打开工程。
