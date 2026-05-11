# OpenNest Recipe Standard v1

## 文件命名

每个应用一个目录：

```text
recipes/{appId}/recipe.opennest.json
```

例如：

```text
recipes/openclaw/recipe.opennest.json
recipes/open-webui/recipe.opennest.json
recipes/flowise/recipe.opennest.json
```

## 最小字段

```json
{
  "schemaVersion": "1.0.0",
  "id": "openclaw",
  "name": "OpenClaw",
  "summary": "Run OpenClaw locally with one-click setup.",
  "runtime": "native-cli",
  "category": "Local AI Assistant",
  "versionSource": "latest",
  "homepage": "https://github.com/openclaw/openclaw",
  "license": "upstream",
  "ports": [18789],
  "requirements": {},
  "install": {},
  "start": {},
  "stop": {},
  "dashboard": {},
  "logs": {},
  "secrets": [],
  "permissions": []
}
```

## 变量

Recipe 中可以使用变量：

```text
${appDir}       当前应用目录
${runtimeDir}   runtime 目录
${dataDir}      数据目录
${logsDir}      日志目录
${secrets.*}    密钥值，只能由后端注入，不能进入前端
${env.*}        环境变量
```

## Runtime 行为统一

每个 runtime adapter 至少支持：

```text
check
install
start
stop
restart
open
logs
repair
uninstall（后续）
update（后续）
```

## 密钥标准

密钥字段只描述“需要什么”，不保存真实值。

```json
{
  "id": "modelApiToken",
  "label": "Model API Token",
  "required": true,
  "store": "system-credential-store",
  "redact": true
}
```

真实 Token 必须：

- 不进 recipe
- 不进 localStorage
- 不进 app config JSON
- 不进日志
- 不被前端读取
- 只由 Tauri/Rust 后端在启动进程时注入

## 权限标准

第一版只做描述，不强制 sandbox：

```json
{
  "type": "network",
  "level": "local",
  "description": "Starts a local dashboard on 127.0.0.1:18789"
}
```

后续可以升级成安装前权限弹窗。
