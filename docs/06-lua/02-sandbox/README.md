# 沙箱安全 (Sandbox)

避免用户脚本成为后门程序，破坏本地文件。

## 环境清洗
在启动 Lua Runtime 时，必须摘掉所有危险包引用。
Rust 中需执行：
```rust
globals.set("os", lua.null())?;
globals.set("io", lua.null())?;
globals.set("debug", lua.null())?;
globals.set("package", lua.null())?;
globals.set("dofile", lua.null())?;
globals.set("loadfile", lua.null())?;
```

仅保留 `math`, `string`, `table` 以及打印断言方法如 `print`, `type`, `tostring`, `pcall`。

## 运行时防护
利用 `lua.set_hook` 限制指令数：
```rust
// 设置最大执行指令阀门，每 10,000 条指令检查一次，防止脚本中出现 `while true do end` 把编辑主线程卡死
lua.set_hook(mlua::HookTriggers::every_nth_instruction(10_000), |_lua, _debug| {
    if timeout_exceeded() {
        return Err(mlua::Error::RuntimeError("Script execution time limit exceeded".into()));
    }
    Ok(())
});
```
