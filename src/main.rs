pub mod app;
pub mod editor;
pub mod effects;
pub mod gui;
pub mod keybinding;
pub mod lua_engine;
pub mod media;
pub mod project;
pub mod rendering;
pub mod search;
pub mod timeline;

use eframe::egui;
use std::env;
use std::path::Path;

fn main() -> eframe::Result<()> {
    // 初始化日志记录
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    // 1. 命令行帮助支持 (--help / -h)
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("===============================================================");
        println!(" VideoCut - 无鼠标键盘优先的极客视频剪辑软件");
        println!("===============================================================");
        println!("用法 (Usage):");
        println!("  video_cut                    启动并进入主导航网格界面");
        println!("  video_cut <project.vcut>     直接打开指定的 .vcut 工程");
        println!("  video_cut <project.lua>      直接加载并运行 .lua 工程脚本");
        println!("  video_cut run <workflow.lua> 命令行无头模式执行剪切流程脚本并落盘");
        println!("  video_cut --help / -h        显示本帮助信息");
        println!();
        println!("快捷键概述 (Keybindings Overview):");
        println!("  f              导航页 Vimium 极速字母跳跃打开项目");
        println!("  /              导航页拼音/模糊实时过滤搜索");
        println!("  h / j / k / l  光标移动 (左右微调 / 上下轨道选择)");
        println!("  s              当前播放头分割切片 (Split)");
        println!("  d              删除当前切片至垃圾回收轨道");
        println!("  u / Ctrl+r     撤销 / 重做 (Undo / Redo)");
        println!("  m / M          设置局部 / 全局锚点");
        println!("  '              跳转到锚点面板");
        println!("  v / V          Visual 时间区间选择 / Line Visual 整块多选");
        println!("  Ctrl+o / Ctrl+i 页面历史前进 / 后退");
        println!("  :              进入 Vim 命令行模式 (:split, :name, :export_lua, etc.)");
        println!("  ?              显示/隐藏半透明快捷键帮助面板");
        println!("===============================================================");
        return Ok(());
    }

    // 2. 命令行脚本无头执行 (Headless Cut Workflow Execution)
    if args.len() >= 3 && args[1] == "run" {
        let script_path = &args[2];
        match std::fs::read_to_string(script_path) {
            Ok(script) => {
                println!("正在执行 Lua 剪辑流程脚本: {} ...", script_path);
                match lua_engine::load_project_from_lua(&script) {
                    Ok(proj) => {
                        println!("✅ 剪切流程脚本执行成功!");
                        println!("   项目名称: {}", proj.name);
                        println!("   轨道数量: {}", proj.timeline.tracks.len());
                        let total_clips: usize =
                            proj.timeline.tracks.iter().map(|t| t.clips.len()).sum();
                        println!("   切片总数: {}", total_clips);
                        let vcut_path = Path::new(script_path).with_extension("vcut");
                        if project::ProjectStorage::save_project_atomic(&vcut_path, &proj).is_ok() {
                            println!("   已自动落盘保存工程: {}", vcut_path.display());
                        }
                        return Ok(());
                    }
                    Err(e) => {
                        eprintln!("❌ 执行剪辑脚本失败: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ 无法读取脚本文件 {}: {}", script_path, e);
                std::process::exit(1);
            }
        }
    }

    // 3. 打开外部指定工程
    let initial_project = if args.len() >= 2 && !args[1].starts_with('-') {
        let file_path = &args[1];
        if file_path.ends_with(".lua") {
            if let Ok(code) = std::fs::read_to_string(file_path) {
                lua_engine::load_project_from_lua(&code).ok()
            } else {
                None
            }
        } else if file_path.ends_with(".vcut") {
            project::ProjectStorage::load_project(Path::new(file_path)).ok()
        } else {
            None
        }
    } else {
        None
    };

    // 设置 eframe 窗口选项
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_title("VideoCut"),
        ..Default::default()
    };

    // 运行原生应用
    eframe::run_native(
        "VideoCut",
        options,
        Box::new(move |cc| {
            if let Some(proj) = initial_project {
                Ok(Box::new(app::VideoCutApp::new_with_project(cc, proj)))
            } else {
                Ok(Box::new(app::VideoCutApp::new(cc)))
            }
        }),
    )
}
