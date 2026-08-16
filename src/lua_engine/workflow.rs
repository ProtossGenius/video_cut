use crate::project::ProjectState;
use crate::timeline::{AnchorPoint, AnchorScope, AssetId, Clip, ClipId, FrameTime, Track, TrackId};
use anyhow::{anyhow, Result};
use mlua::{FromLuaMulti, Lua, MultiValue, Result as LuaResult, Value};
use std::sync::{Arc, Mutex};

/// 从 MultiValue 中提取参数，若第一个参数是 Table (即通过 timeline:method 冒号调用传入的 self) 则自动剥离
fn extract_args<T: FromLuaMulti>(lua: &Lua, mut args: MultiValue) -> LuaResult<T> {
    if let Some(first) = args.front() {
        if matches!(first, Value::Table(_)) {
            let _ = args.pop_front();
        }
    }
    T::from_lua_multi(args, lua)
}

/// Lua 剪切流程脚本执行引擎
pub struct CutWorkflowEngine {
    lua: Lua,
}

impl Default for CutWorkflowEngine {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| panic!("Failed to init CutWorkflowEngine: {}", e))
    }
}

impl CutWorkflowEngine {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        // 强化沙箱安全
        let globals = lua.globals();
        let _ = globals.set("os", Value::Nil);
        let _ = globals.set("io", Value::Nil);
        let _ = globals.set("debug", Value::Nil);
        let _ = globals.set("package", Value::Nil);
        let _ = globals.set("dofile", Value::Nil);
        let _ = globals.set("loadfile", Value::Nil);

        Ok(Self { lua })
    }

    /// 执行剪辑流程脚本并作用到 ProjectState
    pub fn execute_workflow(&self, project: &mut ProjectState, script: &str) -> Result<()> {
        let proj_ref = Arc::new(Mutex::new(project.clone()));
        let globals = self.lua.globals();

        let timeline_tbl = self
            .lua
            .create_table()
            .map_err(|e| anyhow!("Lua table error: {}", e))?;

        // new_project(name)
        {
            let p = proj_ref.clone();
            let func = self.lua.create_function(move |lua, args: MultiValue| -> LuaResult<()> {
                let (name,): (String,) = extract_args(lua, args)?;
                let mut proj = p.lock().map_err(|_| mlua::Error::RuntimeError("Lock poisoned".into()))?;
                proj.name = name;
                proj.timeline.tracks.clear();
                Ok(())
            }).map_err(|e| anyhow!("Lua func error: {}", e))?;
            timeline_tbl.set("new_project", func).map_err(|e| anyhow!("Lua set error: {}", e))?;
        }

        // create_track(name) -> track_id
        {
            let p = proj_ref.clone();
            let func = self.lua.create_function(move |lua, args: MultiValue| -> LuaResult<u64> {
                let (name,): (String,) = extract_args(lua, args)?;
                let mut proj = p.lock().map_err(|_| mlua::Error::RuntimeError("Lock poisoned".into()))?;
                let new_id = TrackId((proj.timeline.tracks.len() + 1) as u64);
                let track = Track::new(new_id, name);
                proj.timeline.add_track(track);
                Ok(new_id.0)
            }).map_err(|e| anyhow!("Lua func error: {}", e))?;
            timeline_tbl.set("create_track", func).map_err(|e| anyhow!("Lua set error: {}", e))?;
        }

        // rename_track(track_id, new_name)
        {
            let p = proj_ref.clone();
            let func = self.lua.create_function(move |lua, args: MultiValue| -> LuaResult<bool> {
                let (track_id, new_name): (u64, String) = extract_args(lua, args)?;
                let mut proj = p.lock().map_err(|_| mlua::Error::RuntimeError("Lock poisoned".into()))?;
                if let Some(t) = proj.timeline.tracks.iter_mut().find(|t| t.id.0 == track_id) {
                    t.name = new_name;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }).map_err(|e| anyhow!("Lua func error: {}", e))?;
            timeline_tbl.set("rename_track", func).map_err(|e| anyhow!("Lua set error: {}", e))?;
        }

        // add_clip(track_id, name, start_us, duration_us) -> clip_id
        {
            let p = proj_ref.clone();
            let func = self.lua.create_function(
                move |lua, args: MultiValue| -> LuaResult<u64> {
                    let (track_id, name, start_us, duration_us): (u64, String, i64, i64) = extract_args(lua, args)?;
                    let mut proj = p.lock().map_err(|_| mlua::Error::RuntimeError("Lock poisoned".into()))?;
                    let total_clips: u64 =
                        proj.timeline.tracks.iter().map(|t| t.clips.len() as u64).sum();
                    let clip_id = ClipId(total_clips + 1);
                    let clip = Clip::new(
                        clip_id,
                        name,
                        AssetId(1),
                        FrameTime(start_us),
                        FrameTime(duration_us),
                    );
                    if let Some(track) =
                        proj.timeline.tracks.iter_mut().find(|t| t.id.0 == track_id)
                    {
                        track.add_clip(clip);
                        Ok(clip_id.0)
                    } else {
                        Err(mlua::Error::RuntimeError(format!(
                            "Track {} not found",
                            track_id
                        )))
                    }
                },
            ).map_err(|e| anyhow!("Lua func error: {}", e))?;
            timeline_tbl.set("add_clip", func).map_err(|e| anyhow!("Lua set error: {}", e))?;
        }

        // split(track_id, clip_id, split_time_us) -> (left_id, right_id)
        {
            let p = proj_ref.clone();
            let func = self.lua.create_function(
                move |lua, args: MultiValue| -> LuaResult<(u64, u64)> {
                    let (track_id, clip_id, split_us): (u64, u64, i64) = extract_args(lua, args)?;
                    let mut proj = p.lock().map_err(|_| mlua::Error::RuntimeError("Lock poisoned".into()))?;
                    let total_clips: u64 =
                        proj.timeline.tracks.iter().map(|t| t.clips.len() as u64).sum();
                    let new_id = ClipId(total_clips + 100);

                    if let Some(track) =
                        proj.timeline.tracks.iter_mut().find(|t| t.id.0 == track_id)
                    {
                        if let Some(pos) = track.clips.iter().position(|c| c.id.0 == clip_id) {
                            let clip = track.clips.remove(pos);
                            let split_pt = FrameTime(split_us);
                            if split_pt > clip.timeline_start && split_pt < clip.timeline_end() {
                                let left_dur = split_pt - clip.timeline_start;
                                let mut left = clip.clone();
                                left.source_out = clip.source_in + left_dur;

                                let right_dur = clip.timeline_end() - split_pt;
                                let mut right = clip;
                                right.id = new_id;
                                right.timeline_start = split_pt;
                                right.source_in = right.source_in + left_dur;
                                right.source_out = right.source_in + right_dur;

                                let left_id = left.id.0;
                                let right_id = right.id.0;

                                track.clips.insert(pos, left);
                                track.clips.insert(pos + 1, right);
                                return Ok((left_id, right_id));
                            }
                        }
                    }
                    Err(mlua::Error::RuntimeError("Split failed".into()))
                },
            ).map_err(|e| anyhow!("Lua func error: {}", e))?;
            timeline_tbl.set("split", func).map_err(|e| anyhow!("Lua set error: {}", e))?;
        }

        // merge(track_id, clip_ids_table) -> merged_id
        {
            let p = proj_ref.clone();
            let func = self.lua.create_function(
                move |lua, args: MultiValue| -> LuaResult<u64> {
                    let (track_id, clip_ids): (u64, Vec<u64>) = extract_args(lua, args)?;
                    let mut proj = p.lock().map_err(|_| mlua::Error::RuntimeError("Lock poisoned".into()))?;
                    let total_clips: u64 =
                        proj.timeline.tracks.iter().map(|t| t.clips.len() as u64).sum();
                    let merged_id = ClipId(total_clips + 200);

                    if let Some(track) =
                        proj.timeline.tracks.iter_mut().find(|t| t.id.0 == track_id)
                    {
                        let matching: Vec<Clip> = track
                            .clips
                            .iter()
                            .filter(|c| clip_ids.contains(&c.id.0))
                            .cloned()
                            .collect();
                        if matching.len() >= 2 {
                            let min_start = matching.first().unwrap().timeline_start;
                            let max_end = matching.last().unwrap().timeline_end();
                            let total_dur = max_end - min_start;
                            let first = &matching[0];

                            let merged_clip = Clip {
                                id: merged_id,
                                name: format!("{}_merged", first.name),
                                source: first.source,
                                timeline_start: min_start,
                                source_in: first.source_in,
                                source_out: first.source_in + total_dur,
                                anchors: std::collections::HashMap::new(),
                                locked: false,
                                z_index: first.z_index,
                                speed: 1.0,
                                audio_fade_in: FrameTime::ZERO,
                                audio_fade_out: FrameTime::ZERO,
                                transform_rotation_deg: 0.0,
                                transform_scale: [1.0, 1.0],
                                transform_flip_h: false,
                                transform_flip_v: false,
                                transform_offset: [0.0, 0.0],
                                color_grading: crate::effects::ColorGradingParams::default(),
                                transition_out: None,
                            };
                            track.clips.retain(|c| !clip_ids.contains(&c.id.0));
                            track.add_clip(merged_clip);
                            return Ok(merged_id.0);
                        }
                    }
                    Err(mlua::Error::RuntimeError("Merge failed".into()))
                },
            ).map_err(|e| anyhow!("Lua func error: {}", e))?;
            timeline_tbl.set("merge", func).map_err(|e| anyhow!("Lua set error: {}", e))?;
        }

        // delete_clip(track_id, clip_id)
        {
            let p = proj_ref.clone();
            let func = self.lua.create_function(
                move |lua, args: MultiValue| -> LuaResult<bool> {
                    let (track_id, clip_id): (u64, u64) = extract_args(lua, args)?;
                    let mut proj = p.lock().map_err(|_| mlua::Error::RuntimeError("Lock poisoned".into()))?;
                    if let Some(track) =
                        proj.timeline.tracks.iter_mut().find(|t| t.id.0 == track_id)
                    {
                        track.clips.retain(|c| c.id.0 != clip_id);
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                },
            ).map_err(|e| anyhow!("Lua func error: {}", e))?;
            timeline_tbl.set("delete_clip", func).map_err(|e| anyhow!("Lua set error: {}", e))?;
        }

        // set_speed(track_id, clip_id, speed)
        {
            let p = proj_ref.clone();
            let func = self.lua.create_function(
                move |lua, args: MultiValue| -> LuaResult<bool> {
                    let (track_id, clip_id, speed): (u64, u64, f32) = extract_args(lua, args)?;
                    let mut proj = p.lock().map_err(|_| mlua::Error::RuntimeError("Lock poisoned".into()))?;
                    if let Some(track) =
                        proj.timeline.tracks.iter_mut().find(|t| t.id.0 == track_id)
                    {
                        if let Some(c) = track.clips.iter_mut().find(|c| c.id.0 == clip_id) {
                            c.speed = speed.max(0.01);
                            return Ok(true);
                        }
                    }
                    Ok(false)
                },
            ).map_err(|e| anyhow!("Lua func error: {}", e))?;
            timeline_tbl.set("set_speed", func).map_err(|e| anyhow!("Lua set error: {}", e))?;
        }

        // set_lock(track_id, clip_id, locked)
        {
            let p = proj_ref.clone();
            let func = self.lua.create_function(
                move |lua, args: MultiValue| -> LuaResult<bool> {
                    let (track_id, clip_id, locked): (u64, u64, bool) = extract_args(lua, args)?;
                    let mut proj = p.lock().map_err(|_| mlua::Error::RuntimeError("Lock poisoned".into()))?;
                    if let Some(track) =
                        proj.timeline.tracks.iter_mut().find(|t| t.id.0 == track_id)
                    {
                        if let Some(c) = track.clips.iter_mut().find(|c| c.id.0 == clip_id) {
                            c.locked = locked;
                            return Ok(true);
                        }
                    }
                    Ok(false)
                },
            ).map_err(|e| anyhow!("Lua func error: {}", e))?;
            timeline_tbl.set("set_lock", func).map_err(|e| anyhow!("Lua set error: {}", e))?;
        }

        // add_anchor(track_id, clip_id, name, pos_us, desc)
        {
            let p = proj_ref.clone();
            let func = self.lua.create_function(
                move |lua, args: MultiValue| -> LuaResult<bool> {
                    let (track_id, clip_id, name, pos_us, desc): (u64, u64, String, i64, String) = extract_args(lua, args)?;
                    let mut proj = p.lock().map_err(|_| mlua::Error::RuntimeError("Lock poisoned".into()))?;
                    if let Some(track) =
                        proj.timeline.tracks.iter_mut().find(|t| t.id.0 == track_id)
                    {
                        if let Some(c) = track.clips.iter_mut().find(|c| c.id.0 == clip_id) {
                            c.anchors.insert(
                                name.clone(),
                                AnchorPoint {
                                    name,
                                    position: FrameTime(pos_us),
                                    description: desc,
                                    scope: AnchorScope::Local,
                                },
                            );
                            return Ok(true);
                        }
                    }
                    Ok(false)
                },
            ).map_err(|e| anyhow!("Lua func error: {}", e))?;
            timeline_tbl.set("add_anchor", func).map_err(|e| anyhow!("Lua set error: {}", e))?;
        }

        globals
            .set("timeline", timeline_tbl)
            .map_err(|e| anyhow!("Lua set timeline error: {}", e))?;

        self.lua
            .load(script)
            .exec()
            .map_err(|e| anyhow!("Lua workflow execution error: {}", e))?;

        let modified_proj = proj_ref
            .lock()
            .map_err(|_| anyhow!("Lock poisoned on completion"))?
            .clone();
        *project = modified_proj;

        Ok(())
    }
}

/// 将 ProjectState 完整序列化导出为规范的 Lua 脚本
pub fn export_project_to_lua(project: &ProjectState) -> String {
    let mut out = String::new();
    out.push_str("-- ===========================================\n");
    out.push_str(&format!("-- VideoCut Project: {}\n", project.name));
    out.push_str("-- Generated by VideoCut Lua Project Exporter\n");
    out.push_str("-- ===========================================\n\n");

    out.push_str(&format!("timeline:new_project({:?})\n\n", project.name));

    for (t_idx, track) in project.timeline.tracks.iter().enumerate() {
        let var_name = format!("t{}", t_idx + 1);
        out.push_str(&format!("-- 轨道: {}\n", track.name));
        out.push_str(&format!("local {} = timeline:create_track({:?})\n", var_name, track.name));

        for (c_idx, clip) in track.clips.iter().enumerate() {
            let clip_var = format!("c{}_{}", t_idx + 1, c_idx + 1);
            out.push_str(&format!(
                "local {} = timeline:add_clip({}, {:?}, {}, {})\n",
                clip_var, var_name, clip.name, clip.timeline_start.0, clip.duration().0
            ));

            if (clip.speed - 1.0).abs() > 0.01 {
                out.push_str(&format!("timeline:set_speed({}, {}, {})\n", var_name, clip_var, clip.speed));
            }
            if clip.locked {
                out.push_str(&format!("timeline:set_lock({}, {}, true)\n", var_name, clip_var));
            }
            for (aname, anchor) in &clip.anchors {
                out.push_str(&format!(
                    "timeline:add_anchor({}, {}, {:?}, {}, {:?})\n",
                    var_name, clip_var, aname, anchor.position.0, anchor.description
                ));
            }
        }
        out.push('\n');
    }

    out
}

/// 从 Lua 脚本加载重构 ProjectState
pub fn load_project_from_lua(script: &str) -> Result<ProjectState> {
    let mut project = ProjectState::new("Imported Lua Project");
    let engine = CutWorkflowEngine::new()?;
    engine.execute_workflow(&mut project, script)?;
    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cut_workflow_script_execution_and_e2e() {
        let engine = CutWorkflowEngine::new().unwrap();
        let mut project = ProjectState::new("Test Engine");

        let script = r#"
            timeline:new_project("My VideoCut Masterpiece")
            local v1 = timeline:create_track("Video 1")
            local a1 = timeline:create_track("Audio 1")

            local c1 = timeline:add_clip(v1, "intro.mp4", 0, 10000000) -- 10s
            local c2 = timeline:add_clip(v1, "broll.mp4", 10000000, 5000000) -- 5s
            local bgm = timeline:add_clip(a1, "bgm.mp3", 0, 15000000)

            -- 分割 c1 在 4s 处
            local c1_left, c1_right = timeline:split(v1, c1, 4000000)
            timeline:set_speed(v1, c1_right, 2.0)
            timeline:set_lock(v1, c1_left, true)
            timeline:add_anchor(v1, c1_left, "intro_mark", 1000000, "Intro anchor")
            timeline:delete_clip(v1, c2)
        "#;

        engine.execute_workflow(&mut project, script).unwrap();

        assert_eq!(project.name, "My VideoCut Masterpiece");
        assert_eq!(project.timeline.tracks.len(), 2);

        let v1 = &project.timeline.tracks[0];
        assert_eq!(v1.clips.len(), 2); // c1_left, c1_right (c2 deleted)
        assert!(v1.clips[0].locked);
        assert_eq!(v1.clips[0].anchors.len(), 1);
        assert_eq!(v1.clips[1].speed, 2.0);
    }

    #[test]
    fn test_export_and_import_lua_project_roundtrip() {
        let mut original = ProjectState::new("Roundtrip Vlog");
        let mut t1 = Track::new(TrackId(1), "V1 - Main");
        let mut c1 = Clip::new(ClipId(1), "hero.mp4".into(), AssetId(1), FrameTime(0), FrameTime(6_000_000));
        c1.locked = true;
        c1.speed = 1.5;
        c1.anchors.insert("start".into(), AnchorPoint { name: "start".into(), position: FrameTime(500_000), description: "Start point".into(), scope: AnchorScope::Local });
        t1.add_clip(c1);
        original.timeline.add_track(t1);

        // 导出为 Lua 脚本
        let lua_code = export_project_to_lua(&original);
        assert!(lua_code.contains("timeline:new_project(\"Roundtrip Vlog\")"));
        assert!(lua_code.contains("timeline:create_track(\"V1 - Main\")"));
        assert!(lua_code.contains("timeline:set_lock"));
        assert!(lua_code.contains("timeline:set_speed"));

        // 从 Lua 脚本重新加载
        let loaded = load_project_from_lua(&lua_code).unwrap();
        assert_eq!(loaded.name, "Roundtrip Vlog");
        assert_eq!(loaded.timeline.tracks.len(), 1);
        let clip = &loaded.timeline.tracks[0].clips[0];
        assert_eq!(clip.name, "hero.mp4");
        assert!(clip.locked);
        assert_eq!(clip.speed, 1.5);
        assert!(clip.anchors.contains_key("start"));
    }
}
