# Merge Guide

## 1. 复制文件

运行：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\copy-into-opennest.ps1 -OpenNestRoot "C:\path\to\opennest"
```

复制后会新增：

```text
src/features/recipes/
src-tauri/src/recipe_runtime/
registry/
recipes/
```

## 2. 合并 Rust 命令

打开：

```text
src-tauri/src/main.rs
```

按 `snippets/register-recipe-runtime-main.rs` 注册：

```rust
mod recipe_runtime;
```

以及：

```rust
recipe_runtime::commands::recipe_check_environment,
recipe_runtime::commands::recipe_install,
recipe_runtime::commands::recipe_start,
recipe_runtime::commands::recipe_stop,
recipe_runtime::commands::recipe_restart,
recipe_runtime::commands::recipe_open_dashboard,
recipe_runtime::commands::recipe_read_logs,
recipe_runtime::commands::recipe_run_doctor,
recipe_runtime::commands::recipe_repair,
```

## 3. 合并 Cargo 依赖

打开：

```text
src-tauri/Cargo.toml
```

补上 `snippets/cargo-dependencies.toml` 里的依赖。

## 4. 接入 App Center

把 `RecipeAppCenter` 放进现有 App Center 页面。

如果你现在已有自己的 App Center，不要替换整个页面，只新增一个 section：

```tsx
<RecipeAppCenter onOpenApp={(appId) => navigate(`/apps/${appId}`)} />
```

## 5. 接入详情页

路由示例：

```text
/apps/:appId
```

对应组件：

```tsx
<RecipeDetailPage appId={appId} />
```

## 6. 接入 My Library

把已安装状态读出来，显示在 My Library。

第一版可以先只根据 recipe status 显示。

## 7. 验收顺序

不要一次测所有 app。

先测：

```text
OpenClaw → Open WebUI → Flowise → Dify
```

Dify 最后测，因为它最重。
