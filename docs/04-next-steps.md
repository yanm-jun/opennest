# Next Steps

## 第一阶段：Recipe System 跑起来

目标：OpenNest 能读取 registry 和 recipes，App Center 出现多个应用卡片。

验收：

- App Center 出现 OpenClaw、Open WebUI、Flowise、Dify。
- 点击卡片进入详情页。
- 详情页能显示 install/start/open/logs 按钮。

## 第二阶段：native-cli 跑通 OpenClaw

目标：OpenClaw 可以安装、填 Token、启动、打开 Dashboard。

验收：

- Check Environment 成功。
- Install OpenClaw 成功。
- Save Token 不泄露。
- Start Gateway 成功或能打开 Official Onboarding。
- Logs 能看到真实错误。

## 第三阶段：docker-compose 跑通 Open WebUI

目标：Docker Compose runtime 能用。

验收：

- 检查 Docker Desktop。
- 写入 compose.yml。
- docker compose up -d。
- 打开 http://127.0.0.1:3000。
- Stop 能 down 或 stop。

## 第四阶段：Flowise

目标：验证第二个 docker-compose app，证明 runtime 可复用。

验收：

- 不新增特殊后端代码。
- 只靠 recipe 启动 Flowise。

## 第五阶段：Dify external-compose

目标：验证大型复杂项目的整合方式。

验收：

- OpenNest 不手抄 Dify compose。
- 能引导用户 clone 官方 repo / docker 目录。
- 能从 OpenNest 控制 docker compose。

## 第六阶段：闭源 Agent 接入

目标：Agent 作者上传一个 recipe + package，OpenNest 管运行。

关键：

- 权限声明
- 密钥注入
- 本地资料箱
- 运行日志
- 网页工作台
- 收费/分成后续再说
