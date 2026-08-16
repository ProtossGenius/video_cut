use anyhow::{Context, Result};
use crossbeam_channel::{unbounded, Receiver, Sender};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};

/// 脚本热重载文件系统监听器
pub struct ScriptWatcher {
    _watcher: RecommendedWatcher,
    pub reload_rx: Receiver<PathBuf>,
}

impl ScriptWatcher {
    /// 创建一个新的脚本监听器，监听指定目录下的所有 .lua 变更
    pub fn new(watch_path: &Path) -> Result<Self> {
        let (tx, rx): (Sender<PathBuf>, Receiver<PathBuf>) = unbounded();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        for path in event.paths {
                            if path.extension().and_then(|ext| ext.to_str()) == Some("lua") {
                                let _ = tx.send(path);
                            }
                        }
                    }
                }
            },
            Config::default(),
        )
        .context("Failed to create file watcher")?;

        if watch_path.exists() {
            watcher
                .watch(watch_path, RecursiveMode::Recursive)
                .with_context(|| format!("Failed to watch directory: {:?}", watch_path))?;
        }

        Ok(Self {
            _watcher: watcher,
            reload_rx: rx,
        })
    }

    /// 尝试以非阻塞方式获取最新被修改的脚本文件
    pub fn try_recv(&self) -> Option<PathBuf> {
        self.reload_rx.try_recv().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_script_watcher_detects_lua_file_change() {
        let tmp_dir = std::env::temp_dir().join(format!("vcut_watcher_{}", std::process::id()));
        let _ = fs::create_dir_all(&tmp_dir);

        let watcher = ScriptWatcher::new(&tmp_dir);
        assert!(watcher.is_ok(), "Watcher should initialize successfully");
        let watcher = watcher.unwrap();

        let lua_file = tmp_dir.join("test_script.lua");
        let txt_file = tmp_dir.join("ignored.txt");

        // 写入被忽略的 txt 文件
        {
            let mut f = File::create(&txt_file).unwrap();
            f.write_all(b"ignored").unwrap();
        }

        // 写入 lua 文件
        {
            let mut f = File::create(&lua_file).unwrap();
            f.write_all(b"begin_animation()").unwrap();
        }

        // 等待操作系统文件系统事件触发
        sleep(Duration::from_millis(150));

        let mut received = false;
        while let Some(path) = watcher.try_recv() {
            if path.file_name() == lua_file.file_name() {
                received = true;
            }
        }

        assert!(
            received,
            "Should receive notification for modified .lua file"
        );

        let _ = fs::remove_dir_all(&tmp_dir);
    }
}
