# API 接口设计

通过实现 Rust 的 `UserData` trait 向 Lua 侧抛出控制钩子。

## 核心全局对象注册
```rust
// 伪代码：在 Rust 中将 main 对象和其方法注入
let globals = lua.globals();
let main_tbl = lua.create_table()?;

// 注册绑定快捷键 API
main_tbl.set("bind_key", lua.create_function(|_, (mode, key, cmd, desc): (String, String, String, String)| {
    app.bind_shortcut(mode, key, cmd, desc);
    Ok(())
})?)?;

// 注册分组描述
main_tbl.set("bind_group_desc", lua.create_function(|_, (prefix, desc): (String, String)| {
    app.bind_group(prefix, desc);
    Ok(())
})?)?;

globals.set("main", main_tbl)?;
```

通过此 API 设计，所有的内置命令也实质上都是底层调用了 Lua 的绑定层实现，保证架构的对称和优雅。
