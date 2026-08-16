use mlua::{Lua, MultiValue, Result as LuaResult, Table, Value};
use std::sync::{Arc, Mutex};
use thiserror::Error;

use crate::effects::{
    AnimationAction, AnimationSequence, CameraKeyframe, CameraRect, CameraSequence, CoordVal,
    Keyframe,
};
use crate::keybinding::{HelpSystem, Mode};
use crate::timeline::FrameTime;

#[derive(Debug, Error)]
pub enum LuaEngineError {
    #[error("Lua 脚本执行错误: {0}")]
    ExecutionError(String),
    #[error("API 状态错误: {0}")]
    StateError(String),
}

/// Lua 脚本引擎运行时
pub struct LuaRuntime {
    lua: Lua,
    active_animation: Arc<Mutex<Option<AnimationSequence>>>,
    active_camera: Arc<Mutex<Option<CameraSequence>>>,
    help_system: Arc<Mutex<HelpSystem>>,
}

impl Default for LuaRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to initialize Lua runtime")
    }
}

impl LuaRuntime {
    pub fn new() -> Result<Self, LuaEngineError> {
        let lua = Lua::new();
        let active_animation = Arc::new(Mutex::new(None));
        let active_camera = Arc::new(Mutex::new(None));
        let help_system = Arc::new(Mutex::new(HelpSystem::new()));

        let runtime = Self {
            lua,
            active_animation,
            active_camera,
            help_system,
        };

        runtime
            .register_globals()
            .map_err(|e| LuaEngineError::ExecutionError(e.to_string()))?;

        Ok(runtime)
    }

    fn register_globals(&self) -> LuaResult<()> {
        let globals = self.lua.globals();

        // 沙箱安全：环境清洗 (禁止 os, io, debug, package, dofile, loadfile 等危险库)
        globals.set("os", Value::Nil)?;
        globals.set("io", Value::Nil)?;
        globals.set("debug", Value::Nil)?;
        globals.set("package", Value::Nil)?;
        globals.set("dofile", Value::Nil)?;
        globals.set("loadfile", Value::Nil)?;

        // 1. begin_animation()
        let anim_arc = Arc::clone(&self.active_animation);
        let begin_animation = self.lua.create_function(move |_, ()| {
            let mut lock = anim_arc.lock().unwrap();
            *lock = Some(AnimationSequence::new());
            Ok(())
        })?;
        globals.set("begin_animation", begin_animation)?;

        // 2. end_animation()
        let end_animation = self.lua.create_function(|_, ()| Ok(()))?;
        globals.set("end_animation", end_animation)?;

        // 3. begin_camera()
        let cam_arc = Arc::clone(&self.active_camera);
        let begin_camera = self.lua.create_function(move |_, ()| {
            let mut lock = cam_arc.lock().unwrap();
            *lock = Some(CameraSequence::new());
            Ok(())
        })?;
        globals.set("begin_camera", begin_camera)?;

        // 4. end_camera()
        let end_camera = self.lua.create_function(|_, ()| Ok(()))?;
        globals.set("end_camera", end_camera)?;

        // 5. actions 全局表
        let actions_table = self.lua.create_table()?;
        let actions_vec = Arc::new(Mutex::new(Vec::<AnimationAction>::new()));

        let vec_clone = Arc::clone(&actions_vec);
        let add_fn = self.lua.create_function(move |_, val: Table| {
            if let Ok(action_type) = val.get::<String>("type") {
                let mut vec = vec_clone.lock().unwrap();
                match action_type.as_str() {
                    "moveto" => {
                        let x_str: String = val.get("x").unwrap_or_default();
                        let y_str: String = val.get("y").unwrap_or_default();
                        if let (Some(x), Some(y)) =
                            (CoordVal::parse(&x_str), CoordVal::parse(&y_str))
                        {
                            vec.push(AnimationAction::MoveTo { x, y });
                        }
                    }
                    "resize" => {
                        let scale: f32 = val.get("scale").unwrap_or(1.0);
                        let interp: bool = val.get("interpolate").unwrap_or(true);
                        vec.push(AnimationAction::Resize {
                            scale_x: scale,
                            scale_y: scale,
                            interpolate: interp,
                        });
                    }
                    _ => {}
                }
            }
            Ok(())
        })?;
        actions_table.set("add", add_fn)?;
        globals.set("actions", actions_table)?;

        // 6. moveto(x, y)
        let moveto = self.lua.create_function(|lua, (x, y): (Value, Value)| {
            let t = lua.create_table()?;
            t.set("type", "moveto")?;
            t.set("x", value_to_coord_str(x))?;
            t.set("y", value_to_coord_str(y))?;
            Ok(t)
        })?;
        globals.set("moveto", moveto)?;

        // 7. resize(scale, [pct], [interpolate])
        let resize = self.lua.create_function(
            |lua, (scale, _pct, interp): (f32, Option<Value>, Option<bool>)| {
                let t = lua.create_table()?;
                t.set("type", "resize")?;
                t.set("scale", scale)?;
                t.set("interpolate", interp.unwrap_or(true))?;
                Ok(t)
            },
        )?;
        globals.set("resize", resize)?;

        // 8. frame(time_str, [actions_or_table])
        let anim_arc_for_frame = Arc::clone(&self.active_animation);
        let cam_arc_for_frame = Arc::clone(&self.active_camera);
        let actions_vec_for_frame = Arc::clone(&actions_vec);

        let frame_fn =
            self.lua
                .create_function(move |_, (time_str, _param): (String, Option<Value>)| {
                    let time_offset = parse_time_str(&time_str);

                    // 处理动画关键帧
                    let mut anim_lock = anim_arc_for_frame.lock().unwrap();
                    if let Some(ref mut anim) = *anim_lock {
                        let mut current_actions = actions_vec_for_frame.lock().unwrap();
                        let kf = Keyframe {
                            time_offset,
                            actions: current_actions.clone(),
                        };
                        current_actions.clear();
                        let _ = anim.add_keyframe(kf);
                    }

                    // 处理摄像机关键帧
                    let mut cam_lock = cam_arc_for_frame.lock().unwrap();
                    if let Some(ref mut cam) = *cam_lock {
                        cam.add_keyframe(CameraKeyframe {
                            time_offset,
                            focus_rect: CameraRect::full(),
                            smooth_interpolate: true,
                        });
                    }

                    Ok(())
                })?;
        globals.set("frame", frame_fn)?;

        // 9. main 全局对象（提供快捷键与组描述绑定）
        let main_table = self.lua.create_table()?;

        let help_arc_key = Arc::clone(&self.help_system);
        let bind_key_fn = self.lua.create_function(move |_, args: MultiValue| {
            let mut strings: Vec<String> = Vec::new();
            for val in args {
                if let Value::String(s) = val {
                    strings.push(s.to_string_lossy());
                }
            }

            // 支持 main:bind_key(mode, key, cmd, desc) 或 main.bind_key(mode, key, cmd, desc)
            let (mode_str, key, cmd, desc) = match strings.len() {
                4 => (&strings[0], &strings[1], &strings[2], &strings[3]),
                5 => (&strings[1], &strings[2], &strings[3], &strings[4]),
                _ => return Ok(()),
            };

            let mode = parse_mode_str(mode_str);
            let mut help = help_arc_key.lock().unwrap();
            help.register_binding(mode, key, cmd, desc);
            Ok(())
        })?;
        main_table.set("bind_key", bind_key_fn)?;

        let help_arc_group = Arc::clone(&self.help_system);
        let bind_group_desc_fn = self.lua.create_function(move |_, args: MultiValue| {
            let mut strings: Vec<String> = Vec::new();
            for val in args {
                if let Value::String(s) = val {
                    strings.push(s.to_string_lossy());
                }
            }

            let (prefix, desc) = match strings.len() {
                2 => (&strings[0], &strings[1]),
                3 => (&strings[1], &strings[2]),
                _ => return Ok(()),
            };

            let mut help = help_arc_group.lock().unwrap();
            help.register_group_desc(prefix, desc);
            Ok(())
        })?;
        main_table.set("bind_group_desc", bind_group_desc_fn)?;

        globals.set("main", main_table)?;

        Ok(())
    }

    /// 执行 Lua 脚本文本
    pub fn execute(&self, script: &str) -> Result<(), LuaEngineError> {
        self.lua
            .load(script)
            .exec()
            .map_err(|e| LuaEngineError::ExecutionError(e.to_string()))
    }

    /// 获取最后生成的动画序列
    pub fn get_animation_sequence(&self) -> Option<AnimationSequence> {
        self.active_animation.lock().unwrap().clone()
    }

    /// 获取最后生成的摄像机序列
    pub fn get_camera_sequence(&self) -> Option<CameraSequence> {
        self.active_camera.lock().unwrap().clone()
    }

    /// 获取快捷键注册/帮助面板管理器
    pub fn get_help_system(&self) -> HelpSystem {
        self.help_system.lock().unwrap().clone()
    }
}

fn parse_mode_str(s: &str) -> Mode {
    Mode::from_str_loose(s)
}

fn value_to_coord_str(val: Value) -> String {
    match val {
        Value::String(s) => s.to_string_lossy(),
        Value::Integer(i) => i.to_string(),
        Value::Number(n) => n.to_string(),
        _ => "0".to_string(),
    }
}

fn parse_time_str(s: &str) -> FrameTime {
    let s = s.trim().trim_start_matches('+');
    if s == "begin" {
        return FrameTime::ZERO;
    }
    if s == "end" {
        return FrameTime(10_000_000); // 假定 end 为默认 10s
    }

    if let Some((m, rest)) = s.split_once(':') {
        let mins: i64 = m.parse().unwrap_or(0);
        let secs: f64 = rest.parse().unwrap_or(0.0);
        let total_us = (mins * 60 * 1_000_000) + (secs * 1_000_000.0) as i64;
        FrameTime(total_us)
    } else {
        let secs: f64 = s.parse().unwrap_or(0.0);
        FrameTime((secs * 1_000_000.0) as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_animation_script() {
        let runtime = LuaRuntime::new().unwrap();
        let script = r#"
            begin_animation()
            actions.add( moveto("30%", 400) )
            frame('1:00')

            actions.add( resize(0.5, '50%', true) )
            frame('+1:30')
            end_animation()
        "#;

        assert!(runtime.execute(script).is_ok());
        let anim = runtime.get_animation_sequence();
        assert!(anim.is_some());
        let seq = anim.unwrap();
        assert_eq!(seq.keyframes.len(), 2);
        assert_eq!(seq.keyframes[0].time_offset, FrameTime(60_000_000)); // 1 min
        assert_eq!(seq.keyframes[1].time_offset, FrameTime(90_000_000)); // 1.5 min
    }

    #[test]
    fn test_execute_camera_script() {
        let runtime = LuaRuntime::new().unwrap();
        let script = r#"
            begin_camera()
            frame('0:00')
            frame('0:05')
            end_camera()
        "#;

        assert!(runtime.execute(script).is_ok());
        let cam = runtime.get_camera_sequence();
        assert!(cam.is_some());
        let seq = cam.unwrap();
        assert_eq!(seq.keyframes.len(), 2);
    }

    #[test]
    fn test_lua_bind_key_and_group_desc() {
        let runtime = LuaRuntime::new().unwrap();
        let script = r#"
            main:bind_key('normal', 's', ':split<CR>', '分割轨道')
            main:bind_group_desc('ad', '添加元素组')
            main:bind_key('normal', 'ad1', ':add_text', '添加文字')
            main:bind_key('normal', 'ad2', ':add_sticker', '添加贴纸')
        "#;

        assert!(runtime.execute(script).is_ok());
        let help = runtime.get_help_system();
        let items_ad = help.query_help_items(Mode::Normal, "ad");
        assert_eq!(items_ad.len(), 2);
        assert_eq!(items_ad[0].description, "添加文字");
        assert_eq!(items_ad[1].description, "添加贴纸");

        let items_a = help.query_help_items(Mode::Normal, "a");
        let ad_group = items_a.iter().find(|e| e.key_label == "ad").unwrap();
        assert_eq!(ad_group.description, "添加元素组");
    }

    #[test]
    fn test_lua_sandbox_security() {
        let runtime = LuaRuntime::new().unwrap();

        // 尝试执行危险的 os/io 调用，沙箱中应直接报错
        let script_os = "os.execute('ls')";
        assert!(runtime.execute(script_os).is_err());

        let script_io = "io.open('/tmp/hacked.txt', 'w')";
        assert!(runtime.execute(script_io).is_err());
    }
}
