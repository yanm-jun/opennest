# OpenNest Marketplace

社区 recipe 索引。提交 PR 即可让你的应用出现在 OpenNest 桌面的应用市场中。

## 如何贡献

1. Fork `opennest/app-registry`
2. 在 `index.json` 中添加你的 recipe
3. 确保 JSON 格式正确（可以用 `jq` 或在线工具验证）
4. 提交 PR

## Recipe 字段说明

| 字段 | 必填 | 说明 |
|------|------|------|
| id | ✅ | 唯一标识符，只能用小写字母、数字和连字符 |
| name | ✅ | 应用名称 |
| summary | ✅ | 一句话描述（40字以内） |
| runtime | ✅ | 运行时类型：docker-compose, webview, mcp-server, agent-container |
| category | ✅ | 分类标签 |
| ports | ✅ | 暴露的端口列表 |
| tags | - | 搜索标签 |
| requirements | ✅ | 依赖（docker, memoryGbRecommended 等） |
| start | ✅ | 启动配置（strategy + args + healthcheck） |
| stop | ✅ | 停止配置 |
| dashboard | ✅ | 面板 URL |

## 当前收录

| 应用 | 类型 | 描述 |
|------|------|------|
| n8n | 自动化 | 可视化工作流引擎 |
| Langflow | AI 构建器 | 拖拽式 LLM 应用设计 |
| Qdrant | 向量数据库 | 高性能向量搜索 |
| AnythingLLM | AI 工具箱 | 多模型文档 AI |
| Ollama | 模型运行时 | 本地大模型运行 |