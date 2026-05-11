# Runtime Adapters v1

## native-cli runtime

用于 CLI 应用。

典型流程：

```text
check node/npm/binary
install package
save secrets
start child process
stream logs
open dashboard
```

OpenClaw 就属于这个类型。

## docker-compose runtime

用于容器应用。

典型流程：

```text
check Docker Desktop
generate compose.yml
docker compose pull
docker compose up -d
docker compose logs --tail 200
open dashboard
```

Open WebUI、Flowise 属于这个类型。

## external-compose runtime

用于大型官方 compose 项目。

不要把 Dify 这种 1000+ 行 compose 文件手动维护在 OpenNest 里。更好的方式是：

```text
clone official repo
进入官方 docker 目录
复制 .env.example
引导用户配置
运行 docker compose up -d
```

## webview runtime

用于已有 Web UI 的应用。

典型流程：

```text
start backend
wait healthcheck
open isolated WebviewWindow
```

这以后适合闭源 Agent：OpenNest 不看它内部逻辑，只负责启动、密钥、日志、权限和窗口。
