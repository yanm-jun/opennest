# OpenNest — AI Agent 应用平台

OpenNest 是一个 AI Agent 应用商店 + 一键部署器 + 本地启动器 + 安全运行环境。
未来开发者可以发布不同类型的 AI Agent / AI App，OpenNest 负责展示、安装、环境隔离、版本管理、启动和密钥管理。

## 当前状态

**v27 平台化完成。前端 build 通过 (44 modules, 268KB JS)。**

- OpenClaw 已从平台本体降级为 `apps.ts` 中的第一个 app manifest
- 4 个应用注册：openclaw / open-webui / flowise / dify
- 安装、启动、停止、日志、状态全部通过 `appId` 驱动，不写死任何应用
- 31 个 Rust Tauri commands（22 核心 + 9 stub）
- 中英文 i18n、搜索/分类筛选、进度条、秘钥状态指示、卸载确认

## 目录结构

```
opennest-starter-v27/     # 主前端 (Vite + React + TypeScript)
  src/
    types/app.ts          # 通用类型定义 (OpenNestAppManifest 等 8 种)
    data/apps.ts          # 应用注册表（唯一数据源）
    lib/app-registry.ts   # 通用查询工具
    features/recipes/     # App Center / 详情页 / 我的库 / 验证面板 / 错误中心
    i18n.tsx              # 中英文翻译

drop-in/                  # Tauri 集成代码片段
  src-tauri/src/recipe_runtime/  # Rust 后端 (24 个 .rs 文件)
  frontend/src/features/recipes/ # 前端集成版 (与 v27 共享类型)

registry/                 # 应用注册表
  apps.json               # 已注册应用列表
  schema/                 # JSON Schema

recipes/                  # 应用安装配方
  openclaw/               # OpenClaw (native-cli)
  open-webui/             # Open WebUI (docker-compose)
  flowise/                # Flowise (docker-compose)
  dify/                   # Dify (external-compose)

scripts/                  # 集成脚本
  copy-into-opennest.ps1  # 一键复制到 Tauri 项目

snippets/                 # 集成代码段
  register-recipe-runtime-main.rs
  cargo-dependencies.toml
```

## 新增应用只需三步

1. 在 `opennest-starter-v27/src/data/apps.ts` 的 `apps` 数组中加入一条 `OpenNestAppManifest`
2. 在 `recipes/<appId>/` 下创建 `recipe.opennest.json`
3. 在 `registry/apps.json` 中注册

无需修改任何页面或按钮逻辑。

## 快速开始

```powershell
cd opennest-starter-v27
npm install
npm run build     # TypeScript + Vite 构建
npm run dev       # 开发模式
```

## Tauri 集成

```powershell
powershell -ExecutionPolicy Bypass -File scripts/copy-into-opennest.ps1 -TauriRoot "path/to/your-tauri-project"
```

然后手动：
- `src-tauri/src/main.rs` 注册 `recipe_runtime` 模块和全部 commands
- `src-tauri/Cargo.toml` 补充依赖
- `cargo build` / `cargo tauri dev`

## 验收清单

- [x] TypeScript 编译通过
- [x] Vite 生产构建通过
- [x] appId 通用架构（安装/启动/停止/日志/状态）
- [x] 无 OpenClaw 硬编码
- [x] apps.ts 为唯一数据源
- [x] 31 个 Rust Tauri commands
- [x] 搜索 + 分类筛选
- [x] 进度条可视化
- [x] 秘钥状态指示
- [x] 卸载确认对话框
- [ ] Windows Tauri 真机 E2E (OpenClaw install→start→dashboard)
- [ ] Docker app E2E (Open WebUI / Flowise compose 启动)