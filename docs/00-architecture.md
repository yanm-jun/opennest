# OpenNest Resource Integration Architecture v1

## 一句话定位

OpenNest 不应该只是一个“OpenClaw 安装器”，而应该是：

```text
开源 AI 应用 / Agent / MCP / Docker 项目的桌面级统一入口。
```

## 四层架构

```text
┌───────────────────────────────────────────┐
│ Desktop Host                              │
│ App Center / My Library / Logs / Settings │
└───────────────────────────────────────────┘
                    ↓
┌───────────────────────────────────────────┐
│ Runtime Router                            │
│ native-cli / docker-compose / webview     │
└───────────────────────────────────────────┘
                    ↓
┌───────────────────────────────────────────┐
│ Recipe System                             │
│ install/start/stop/open/logs/secrets      │
└───────────────────────────────────────────┘
                    ↓
┌───────────────────────────────────────────┐
│ Registry                                  │
│ apps.json + recipes/*                     │
└───────────────────────────────────────────┘
```

## 为什么要做 Recipe

如果没有 Recipe，接每个开源项目都要单独写：

- 安装按钮
- 启动按钮
- 停止按钮
- 日志按钮
- 端口检查
- Token 配置
- Docker Compose 管理
- Dashboard 打开

这样一定会乱。

Recipe 的价值就是把这些统一成“标准描述”，OpenNest 只负责执行。

## Runtime 类型

### native-cli

适合：

- OpenClaw
- Agent-S
- 各类 CLI 启动型 Agent
- 本地 MCP server

特点：

```text
下载/安装 CLI → 注入密钥 → 启动进程 → 打开本地 Dashboard
```

### docker-compose

适合：

- Open WebUI
- Flowise
- AnythingLLM
- Langfuse
- ComfyUI 变体

特点：

```text
写入 compose.yml → docker compose up -d → 打开本地端口
```

### external-compose

适合：

- Dify
- 大型多服务系统
- 官方 compose 文件经常变化的项目

特点：

```text
不手抄巨大 compose。
从官方仓库同步 docker 目录，再按官方方式启动。
```

### webview

适合：

- 已有本地 Web UI 的应用
- 闭源 Agent 自己的工作台
- 嵌入式应用窗口

特点：

```text
OpenNest 管启动和权限，应用自己的网页负责使用体验。
```

## OpenNest 不是做什么

OpenNest 不应该：

- 重写开源项目源码
- 复制别人仓库全部源码
- 强行把所有应用 UI 塞进 OpenNest
- 每个应用都写一套特殊逻辑

OpenNest 应该：

- 识别应用需要什么环境
- 安装它
- 启动它
- 停止它
- 打开它的 Dashboard
- 管理它的密钥和日志
- 给用户一个统一体验
