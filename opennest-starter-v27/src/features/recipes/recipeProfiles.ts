import type { OpenNestRecipeSummary, RuntimeKind } from "./types";
import type { Lang } from "../../i18n";

export type ValidationPriority = "first" | "second" | "third" | "last";

export interface RecipeProductProfile {
  accent: string;
  stage: string;
  difficulty: "Easy" | "Medium" | "Heavy";
  priority: ValidationPriority;
  productLine: string;
  hero: string;
  tags: string[];
  systemRequirements: Array<{ label: string; value: string; note: string }>;
  validationSteps: string[];
  failureHints: string[];
}

function fallbackForRuntime(runtime: RuntimeKind, lang: Lang): RecipeProductProfile {
  const docker = runtime === "docker-compose" || runtime === "external-compose";
  const zh = lang === "zh";
  return {
    accent: docker ? "Docker" : zh ? "本地运行时" : "Local runtime",
    stage: zh ? "待验证" : "Needs validation",
    difficulty: docker ? "Medium" : "Easy",
    priority: docker ? "third" : "first",
    productLine: docker ? (zh ? "容器化 AI 应用" : "Containerized AI app") : zh ? "本地 AI 应用" : "Local AI app",
    hero: zh ? "OpenNest 已经可以展示这个 recipe，但真实行为仍需在 Tauri 中验证。" : "OpenNest can display the recipe now. Real behavior should be validated in Tauri.",
    tags: docker ? ["Docker", zh ? "需要预检" : "Preflight required"] : [zh ? "本地" : "Local", zh ? "需要预检" : "Preflight required"],
    systemRequirements: [{ label: zh ? "运行时" : "Runtime", value: runtime, note: zh ? "由 recipe 清单定义。" : "Defined by the recipe manifest." }],
    validationSteps: zh ? ["查看安装计划", "接受计划", "检查环境", "运行资源预检"] : ["View Install Plan", "Accept Plan", "Check Environment", "Run Resource Preflight"],
    failureHints: zh ? ["缺少运行时依赖", "端口冲突", "网络或下载失败"] : ["Runtime dependency missing", "Port conflict", "Network/download failure"],
  };
}

export function getRecipeProductProfile(recipe: OpenNestRecipeSummary, lang: Lang = "en"): RecipeProductProfile {
  const fallback = fallbackForRuntime(recipe.runtime, lang);
  const docker = recipe.runtime === "docker-compose" || recipe.runtime === "external-compose";
  const nativeCli = recipe.runtime === "native-cli";
  const isNativeCli = recipe.runtime === "native-cli";
  const priority = (recipe.priority as ValidationPriority | undefined) ?? fallback.priority;
  const difficulty = (recipe.difficulty as RecipeProductProfile["difficulty"] | undefined) ?? fallback.difficulty;
  const zh = lang === "zh";

  return {
    accent: nativeCli ? (zh ? "原生 CLI" : "Native CLI") : docker ? "Docker" : zh ? "本地运行时" : "Local runtime",
    stage: priority === "first" ? (zh ? "第一验证目标" : "First validation target") : priority === "second" ? (zh ? "第二验证目标" : "Second validation target") : priority === "last" ? (zh ? "最后验证" : "Validate last") : zh ? "第三验证目标" : "Third validation target",
    difficulty,
    priority,
    productLine: recipe.category,
    hero: recipe.description ?? recipe.summary,
    tags: recipe.tags?.length ? recipe.tags : fallback.tags,
    systemRequirements: [
      { label: zh ? "运行时" : "Runtime", value: recipe.runtime, note: zh ? "从模板加载。" : "Loaded from template." },
      { label: zh ? "端口" : "Ports", value: isNativeCli ? (zh ? "桌面端内部管理" : "Desktop-managed") : recipe.ports?.length ? recipe.ports.join(", ") : zh ? "无" : "None", note: isNativeCli ? (zh ? recipe.name + " Desktop 会自动处理本地接入。" : recipe.name + " Desktop handles local access automatically.") : zh ? "由 OpenNest 在运行时解析。" : "Resolved by OpenNest at runtime." },
      ...(nativeCli ? [{ label: "Node", value: zh ? "必需" : "Required", note: zh ? "Windows 下可准备受管 Node。" : "Managed Node can be prepared on Windows." }] : []),
      ...(docker ? [{ label: "Docker", value: zh ? "必需" : "Required", note: zh ? "必须安装 Docker Desktop。" : "Docker Desktop must be available." }] : []),
    ],
    validationSteps: nativeCli
      ? (zh ? ["查看安装计划", "接受计划", "检查环境", "运行资源预检", "安装", "选择提供方", "粘贴 API Key", "打开聊天"] : ["View Install Plan", "Accept Plan", "Check Environment", "Run Resource Preflight", "Install", "Choose provider", "Paste API key", "Open chat"])
      : docker
        ? (zh ? ["接受计划", "检查环境", "运行资源预检", "解析端口", "安装", "启动", "检查就绪", "打开面板"] : ["Accept Plan", "Check Environment", "Run Resource Preflight", "Resolve Ports", "Install", "Start", "Check Readiness", "Open Dashboard"])
        : fallback.validationSteps,
    failureHints: nativeCli
      ? (zh ? ["Node 版本过旧", "CLI 安装失败", "提供方配置不完整", "网关或探测失败"] : ["Node too old", "CLI install failed", "Provider setup incomplete", "Gateway/probe failed"])
      : docker
        ? (zh ? ["Docker 不可用", "Compose 或镜像失败", "端口冲突", "就绪检查失败"] : ["Docker unavailable", "Compose/image failure", "Port conflict", "Readiness failed"])
        : fallback.failureHints,
  };
}

export const validationOrder: Record<ValidationPriority, number> = { first: 1, second: 2, third: 3, last: 4 };
