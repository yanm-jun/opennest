import { createContext, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import type { OpenNestRecipeSummary, RecipeInstallState, RecipeRunState, RuntimeKind } from "./features/recipes/types";

export type Lang = "en" | "zh";

const STORAGE_KEY = "opennest.language";

const dictionary: Record<Lang, Record<string, string>> = {
  en: {
    language: "Language",
    english: "English",
    chinese: "中文",
    app_center: "App Center",
    my_library: "My Library",
    validation: "Validation",
    error_center: "Error Center",
    settings: "Settings",
    browse_template_apps: "Browse template-based apps",
    manage_installed_runtimes: "Manage installed runtimes",
    review_acceptance_evidence: "Review acceptance evidence",
    inspect_failures_and_repair: "Inspect failures and repair",
    control_local_runtime_basics: "Control local runtime basics",
    catalog: "Catalog",
    operations: "Operations",
    gates: "Gates",
    recovery: "Recovery",
    system: "System",
    recipe_detail: "Recipe Detail",
    recipe_detail_desc: "Review install plan, runtime status, logs, and health checks before changing the app state.",
    back_to_dashboard: "Back to dashboard",
    local_application_catalog: "Local application catalog",
    local_application_catalog_desc: "Install open-source tools through a recipe flow with predictable files, ports, and runtime actions.",
    browse_recipes: "Browse recipes",
    local_app_management: "Local app management",
    local_app_management_desc: "Track installed apps, current ports, app access state, health checks, and repair actions from one place.",
    review_library: "Review library",
    acceptance_evidence: "Acceptance evidence",
    acceptance_evidence_desc: "Check runtime progress against the current validation order and keep the first passing chain visible.",
    review_gates: "Review gates",
    failure_explanation: "Failure explanation",
    failure_explanation_desc: "Read the original runtime output, understand likely causes, and apply safe repair actions without leaving the app.",
    inspect_errors: "Inspect errors",
    runtime_settings: "Runtime settings",
    runtime_settings_desc: "Keep the desktop runtime predictable: prerequisites, local paths, ports, and operating boundaries stay explicit.",
    review_settings: "Review settings",
    back: "Back",
    runtime: "Runtime",
    local_first: "Local-first",
    runtime_card_detail: "All app execution stays in Tauri Rust commands and local runtime adapters.",
    runtime_card_note: "No cloud control plane is introduced.",
    layout: "Layout",
    grid_8pt: "8pt grid",
    layout_card_detail: "Cards use consistent spacing, compact controls, and a documentation-like shell.",
    layout_card_note: "The shell stays light and operational.",
    state: "State",
    restart_safe: "Restart-safe",
    state_card_detail: "My Library mirrors local status into persisted JSON for runtime recovery.",
    state_card_note: "Failure flows jump directly into Error Center.",
    desktop_starter: "Desktop Starter v30",
    light_shell: "Light shell",
    open_app_center: "Open App Center",
    workspace_notes: "Workspace notes",
    expand: "Expand",
    current_scope: "Current scope",
    current_scope_primary: "Native CLI + Docker Compose apps",
    current_scope_secondary: "No accounts, billing, or marketplace flows.",
    current_scope_tertiary: "Shell updates stay outside runtime-critical actions.",
    visual_direction: "Visual direction",
    visual_direction_primary: "Light, restrained surfaces",
    visual_direction_secondary: "One neutral brand accent and weak borders.",
    visual_direction_tertiary: "Density stays close to a settings console.",
    local_ai_app_platform: "Local AI App Platform",
    shell_note: "The shell uses one brand color, weak borders, shallow shadows, and compact information blocks.",
    prerequisite_checklist: "Prerequisite checklist",
    prerequisite_desc: "Use this page as a controlled review surface. The goal is not breadth. The first accepted chain should remain explainable and repeatable.",
    required_for_desktop_validation: "Required for desktop validation",
    required_for_desktop_validation_desc: "Local prerequisites checked before runtime actions.",
    operating_boundaries: "Operating boundaries",
    operating_boundaries_desc: "Scope controls that keep the prototype focused.",
    empty_state: "Empty state",
    empty_state_desc: "Use a quiet placeholder instead of noisy illustration when no configuration is active.",
    no_active_configuration: "No active configuration",
    no_active_configuration_desc: "Start from a recipe card, accept the install plan, and let runtime status populate this workspace.",
    application_catalog: "Application catalog",
    recipe_based_local_apps: "Recipe-based local apps",
    recipe_based_local_apps_desc: "Browse app templates, compare runtime type and port usage, then open a detail page for plan review, install, and health checks.",
    bundled_metadata_error: "Using bundled recipe metadata because the runtime loader returned an error: {error}",
    review_featured_apps: "Review featured apps",
    featured: "Featured",
    open_detail: "Open detail",
    view_detail: "View detail",
    no_fixed_port: "No fixed port",
    port_list: "Port {ports}",
    local_library: "Local library",
    installed_runtimes_and_local_state: "Installed runtimes and local state",
    installed_runtimes_and_local_state_desc: "Review install path, current port, app access state, health timestamps, logs, and repair actions without leaving the library view.",
    refresh_library: "Refresh library",
    running_apps: "Running apps",
    running_apps_desc: "Apps with an active runtime or reachable local service.",
    no_running_apps: "No running apps.",
    failed_apps: "Failed apps",
    failed_apps_desc: "Apps that need inspection, retry, or repair.",
    no_failed_apps: "No failed apps.",
    incomplete_installs: "Incomplete installs",
    incomplete_installs_desc: "Apps that still need acceptance, install, or first-run steps.",
    no_incomplete_installs: "No incomplete installs.",
    installed_apps: "Installed apps",
    installed_apps_desc: "Stable apps available for restart, app open, or logs review.",
    no_installed_apps: "No installed apps.",
    no_local_apps_managed_yet: "No local apps managed yet.",
    app_count: "{count} apps",
    install_path: "Install path",
    current_port: "Current port",
    dashboard_url: "App Access",
    last_start: "Last start",
    last_health_check: "Last health check",
    logs_path: "Logs path",
    open_dashboard: "Open App",
    start: "Start",
    stop: "Stop",
    restart: "Restart",
    check_health: "Check Health",
    view_logs: "View Logs",
    repair: "Repair",
    uninstall: "Uninstall",
    no_logs_yet: "No logs yet.",
    failure_center: "Failure center",
    structured_runtime_failures: "Structured runtime failures",
    structured_runtime_failures_desc: "Keep the raw error visible, explain the likely cause in plain language, and expose only safe retry or repair actions.",
    refresh: "Refresh",
    no_active_runtime_errors: "No active runtime errors.",
    retry: "Retry",
    re_detect: "Re-detect",
    likely_cause: "Likely cause",
    next_action: "Next action",
    repair_action: "Repair action",
    exit_code: "Exit code",
    original_error: "Original error",
    stdout: "stdout",
    stderr: "stderr",
    logs: "Logs",
    selected_app_runtime_output: "Raw runtime output stays available for the selected app.",
    refresh_logs: "Refresh logs",
    not_provided: "Not provided",
    review_logs_first: "Review logs first",
    none: "None",
    acceptance_board: "Acceptance board",
    validation_order_and_evidence: "Validation order and evidence",
    validation_order_and_evidence_desc: "Keep the sequence explicit: prove the native-cli chain first, then move to Docker-backed apps with clear readiness and logs.",
    export_evidence: "Export evidence",
    reset_preview: "Reset preview",
    validation_evidence_exported: "Validation evidence exported.",
    reset_browser_preview_confirm: "Reset browser preview state?",
    browser_preview_state_reset: "Browser preview state reset.",
    pass: "Pass",
    pending: "Pending",
    gate_installed_detail: "{app} should have completed its install recipe.",
    gate_running_detail: "{app} should have a healthy running process.",
    gate_healthy_detail: "{app} health check should respond successfully.",
    gate_docker_staged: "{docker} staged after {native}",
    gate_docker_staged_detail: "{docker} install gates are looser until the first native-cli app passes.",
    







    install_plan: "Install plan",
    install_plan_desc: "Review downloads, commands, ports, secrets, rollback, and risk before installing.",
    risk: "Risk",
    time: "Time",
    disk: "Disk",
    plan: "Plan",
    accepted: "Accepted",
    yes: "Yes",
    no: "No",
    network: "Network",
    docker: "Docker",
    node: "Node",
    git: "Git",
    preflight_checks: "Preflight checks",
    downloads_sources: "Downloads / sources",
    commands: "Commands",
    directories: "Directories",
    secrets: "Secrets",
    permissions: "Permissions",
    rollback: "Rollback",
    notes: "Notes",
    no_required_secrets: "No required secrets.",
    no_special_permissions: "No special permissions declared.",
    warnings: "Warnings",
    install_confirmation_gate: "Install confirmation gate",
    accepted_at: "Accepted at {time}. Install is unlocked for this exact plan digest.",
    accept_plan_required: "You must accept this install plan before OpenNest will run Install.",
    accept_plan: "Accept Plan",
    clear_acceptance: "Clear Acceptance",
    runtime_controls: "Runtime controls",
    runtime_controls_desc: "Install, start, stop, repair, and open the app without touching the terminal.",
    view_install_plan: "View Install Plan",
    check_environment: "Check Environment",
    run_resource_preflight: "Run Resource Preflight",
    resolve_ports: "Resolve Ports",
    install: "Install",
    gateway_status: "Gateway Status",
    container_status: "Container Status",
    check_readiness: "Check Readiness",
    run_doctor: "Run Doctor",
    run_official_onboarding: "Run Official Onboarding",
    rollback_failed_install: "Rollback Failed Install",
    uninstall_keep_data: "Uninstall Keep Data",
    uninstall_remove_data: "Uninstall + Remove Data",
    status: "Status",
    installed: "Installed",
    install_state: "Install State",
    run_state: "Run State",
    dashboard: "App Access",
    managed_pid: "Managed PID",
    health: "Health",
    readiness: "Readiness",
    node_runtime: "Node Runtime",
    plan_accepted: "Plan Accepted",
    docker_services: "Docker Services",
    resource_preflight: "Resource Preflight",
    port_resolution: "Port Resolution",
    effective_dashboard: "Resolved Access",
    effective_readiness: "Effective Readiness",
    port_mappings: "Port Mappings",
    progress: "Progress",
    progress_phase: "Progress Phase",
    last_error: "Last Error",
    not_ready: "Not ready",
    unknown: "unknown",
    idle: "Idle",


    model_token: "Model token",

    paste_selected_provider_token: "Paste selected provider token",
    save_token: "Save Token",
    log_section: "Logs",
    product_profile_runtime: "Runtime",
    product_profile_ports: "Ports",
    no_items: "No items.",
    required: "required",
    optional: "optional",
    state_label: "State",
    operation: "Operation",
    phase: "Phase",
    no_active_progress: "No active progress.",
    no_live_events_yet: "No live events yet.",
    blocking: "Blocking",
    checked: "Checked",
    preview_status_initialized: "Preview status initialized.",
    installed_preview: "Install plan accepted in preview.",
  },
  zh: {
    language: "语言",
    english: "English",
    chinese: "中文",
    app_center: "应用中心",
    my_library: "我的库",
    validation: "验证面板",
    error_center: "错误中心",
    settings: "设置",
    browse_template_apps: "浏览模板化应用",
    manage_installed_runtimes: "管理已安装运行时",
    review_acceptance_evidence: "查看验收证据",
    inspect_failures_and_repair: "查看失败并修复",
    control_local_runtime_basics: "管理本地运行时基础设置",
    catalog: "目录",
    operations: "操作",
    gates: "门禁",
    recovery: "恢复",
    system: "系统",
    recipe_detail: "应用详情",
    recipe_detail_desc: "在变更应用状态前，查看安装计划、运行状态、日志和健康检查。",
    back_to_dashboard: "返回总览",
    local_application_catalog: "本地应用目录",
    local_application_catalog_desc: "通过可预测的文件、端口和运行时动作，用 recipe 流程安装开源工具。",
    browse_recipes: "浏览应用",
    local_app_management: "本地应用管理",
    local_app_management_desc: "在一个页面里跟踪已安装应用、当前端口、应用入口状态、健康检查和修复操作。",
    review_library: "查看库",
    acceptance_evidence: "验收证据",
    acceptance_evidence_desc: "对照当前验证顺序检查运行进度，并持续展示第一条跑通链路。",
    review_gates: "查看门禁",
    failure_explanation: "失败说明",
    failure_explanation_desc: "查看原始运行输出，理解可能原因，并在应用内执行安全修复操作。",
    inspect_errors: "查看错误",
    runtime_settings: "运行时设置",
    runtime_settings_desc: "让桌面运行时保持可预测：前置条件、本地路径、端口和运行边界都明确可见。",
    review_settings: "查看设置",
    back: "返回",
    runtime: "运行时",
    local_first: "本地优先",
    runtime_card_detail: "所有应用执行都停留在 Tauri Rust 命令和本地 runtime adapter 内。",
    runtime_card_note: "不引入云端控制平面。",
    layout: "布局",
    grid_8pt: "8pt 网格",
    layout_card_detail: "卡片使用统一间距、紧凑控件和接近文档控制台的外壳。",
    layout_card_note: "整体风格偏轻量、偏操作台。",
    state: "状态",
    restart_safe: "重启可恢复",
    state_card_detail: "我的库会把本地状态同步到持久化 JSON，便于运行时恢复。",
    state_card_note: "失败流程会直接跳到错误中心。",
    desktop_starter: "桌面启动版 v30",
    light_shell: "浅色外壳",
    open_app_center: "打开应用中心",
    workspace_notes: "工作区备注",
    expand: "展开",
    current_scope: "当前范围",
    current_scope_primary: "Native CLI + Docker Compose apps",
    current_scope_secondary: "暂不包含账号、计费或市场流程。",
    current_scope_tertiary: "外壳调整不干扰 runtime 关键路径。",
    visual_direction: "视觉方向",
    visual_direction_primary: "浅色、克制的表面",
    visual_direction_secondary: "单一中性品牌色和弱边框。",
    visual_direction_tertiary: "密度接近设置控制台，而不是营销页。",
    local_ai_app_platform: "本地 AI 应用平台",
    shell_note: "界面采用单一品牌色、弱边框、浅阴影和紧凑信息块。",
    prerequisite_checklist: "前置条件清单",
    prerequisite_desc: "把这一页当作受控检查面板来用。目标不是铺广，而是让第一条跑通链路可解释、可重复。",
    required_for_desktop_validation: "桌面验证必需项",
    required_for_desktop_validation_desc: "运行操作前需要确认的本地依赖。",
    operating_boundaries: "运行边界",
    operating_boundaries_desc: "保证原型聚焦范围的限制条件。",
    empty_state: "空状态",
    empty_state_desc: "当没有激活配置时，使用安静的占位，而不是喧闹插图。",
    no_active_configuration: "当前没有激活配置",
    no_active_configuration_desc: "从某个应用卡片开始，接受安装计划，然后让 runtime 状态填充这个工作区。",
    application_catalog: "应用目录",
    recipe_based_local_apps: "基于 Recipe 的本地应用",
    recipe_based_local_apps_desc: "浏览应用模板，比较运行时类型和端口占用，然后进入详情页查看计划、安装和健康检查。",
    bundled_metadata_error: "运行时加载器报错，当前改用内置 recipe 元数据：{error}",
    review_featured_apps: "查看精选应用",
    featured: "精选",
    open_detail: "打开详情",
    view_detail: "查看详情",
    no_fixed_port: "无固定端口",
    port_list: "端口 {ports}",
    local_library: "本地库",
    installed_runtimes_and_local_state: "已安装运行时与本地状态",
    installed_runtimes_and_local_state_desc: "在库视图内直接查看安装路径、当前端口、应用入口状态、健康时间戳、日志和修复操作。",
    refresh_library: "刷新库",
    running_apps: "运行中的应用",
    running_apps_desc: "已经启动 runtime 或本地服务可访问的应用。",
    no_running_apps: "没有正在运行的应用。",
    failed_apps: "失败应用",
    failed_apps_desc: "需要排查、重试或修复的应用。",
    no_failed_apps: "没有失败应用。",
    incomplete_installs: "未完成安装",
    incomplete_installs_desc: "还需要接受计划、安装或完成首次启动步骤的应用。",
    no_incomplete_installs: "没有未完成安装。",
    installed_apps: "已安装应用",
    installed_apps_desc: "可重启、打开应用或查看日志的稳定应用。",
    no_installed_apps: "没有已安装应用。",
    no_local_apps_managed_yet: "还没有已管理的本地应用。",
    app_count: "{count} 个应用",
    install_path: "安装路径",
    current_port: "当前端口",
    dashboard_url: "应用入口",
    last_start: "上次启动",
    last_health_check: "上次健康检查",
    logs_path: "日志路径",
    open_dashboard: "打开应用",
    start: "启动",
    stop: "停止",
    restart: "重启",
    check_health: "检查健康",
    view_logs: "查看日志",
    repair: "修复",
    uninstall: "卸载",
    no_logs_yet: "还没有日志。",
    failure_center: "失败中心",
    structured_runtime_failures: "结构化运行时失败",
    structured_runtime_failures_desc: "保留原始错误，同时用通俗语言解释可能原因，只暴露安全的重试或修复操作。",
    refresh: "刷新",
    no_active_runtime_errors: "当前没有活跃运行时错误。",
    retry: "重试",
    re_detect: "重新检测",
    likely_cause: "可能原因",
    next_action: "下一步",
    repair_action: "修复动作",
    exit_code: "退出码",
    original_error: "原始错误",
    stdout: "stdout",
    stderr: "stderr",
    logs: "日志",
    selected_app_runtime_output: "保留当前所选应用的原始 runtime 输出。",
    refresh_logs: "刷新日志",
    not_provided: "未提供",
    review_logs_first: "先查看日志",
    none: "无",
    acceptance_board: "验收面板",
    validation_order_and_evidence: "验证顺序与证据",
    validation_order_and_evidence_desc: "顺序必须明确：先证明原生 CLI 链路，再推进 Docker 应用，并保留清晰的 readiness 与日志证据。",
    export_evidence: "导出证据",
    reset_preview: "重置预览",
    validation_evidence_exported: "验证证据已导出。",
    reset_browser_preview_confirm: "要重置浏览器预览状态吗？",
    browser_preview_state_reset: "浏览器预览状态已重置。",
    pass: "通过",
    pending: "待完成",
    gate_installed_detail: "{app} 应已完成安装配方。",
    gate_running_detail: "{app} 应有健康运行的进程。",
    gate_healthy_detail: "{app} 健康检查应有成功响应。",
    gate_docker_staged: "{docker} 在 {native} 之后验证",
    gate_docker_staged_detail: "{docker} 安装门禁较宽松，直到首个 native-cli 应用通过。",








    install_plan: "安装计划",
    install_plan_desc: "安装前先检查下载、命令、端口、密钥、回滚和风险。",
    risk: "风险",
    time: "时间",
    disk: "磁盘",
    plan: "计划",
    accepted: "已接受",
    yes: "是",
    no: "否",
    network: "网络",
    docker: "Docker",
    node: "Node",
    git: "Git",
    preflight_checks: "预检项",
    downloads_sources: "下载 / 来源",
    commands: "命令",
    directories: "目录",
    secrets: "密钥",
    permissions: "权限",
    rollback: "回滚",
    notes: "备注",
    no_required_secrets: "没有必填密钥。",
    no_special_permissions: "没有声明特殊权限。",
    warnings: "警告",
    install_confirmation_gate: "安装确认门禁",
    accepted_at: "已于 {time} 接受。当前安装只对这个 plan digest 解锁。",
    accept_plan_required: "在 OpenNest 执行安装前，你必须先接受这个安装计划。",
    accept_plan: "接受计划",
    clear_acceptance: "清除接受状态",
    runtime_controls: "运行控制",
    runtime_controls_desc: "无需切到终端，就能安装、启动、停止、修复并打开应用。",
    view_install_plan: "查看安装计划",
    check_environment: "检查环境",
    run_resource_preflight: "运行资源预检",
    resolve_ports: "解析端口",
    install: "安装",
    gateway_status: "网关状态",
    container_status: "容器状态",
    check_readiness: "检查就绪",
    run_doctor: "运行 Doctor",
    run_official_onboarding: "运行官方引导",
    rollback_failed_install: "回滚失败安装",
    uninstall_keep_data: "卸载并保留数据",
    uninstall_remove_data: "卸载并删除数据",
    status: "状态",
    installed: "已安装",
    install_state: "安装状态",
    run_state: "运行状态",
    dashboard: "应用入口",
    managed_pid: "受管 PID",
    health: "健康",
    readiness: "就绪",
    node_runtime: "Node 运行时",
    plan_accepted: "计划已接受",
    docker_services: "Docker 服务",
    resource_preflight: "资源预检",
    port_resolution: "端口解析",
    effective_dashboard: "实际入口",
    effective_readiness: "最终就绪地址",
    port_mappings: "端口映射",
    progress: "进度",
    progress_phase: "进度阶段",
    last_error: "最近错误",
    not_ready: "未就绪",
    unknown: "未知",
    idle: "空闲",


    model_token: "模型 Token",

    paste_selected_provider_token: "粘贴所选提供方 Token",
    save_token: "保存 Token",
    log_section: "日志",
    product_profile_runtime: "运行时",
    product_profile_ports: "端口",
    no_items: "暂无内容。",
    required: "必需",
    optional: "可选",
    state_label: "状态",
    operation: "操作",
    phase: "阶段",
    no_active_progress: "当前没有活动进度。",
    no_live_events_yet: "还没有实时事件。",
    blocking: "阻塞项",
    checked: "检查时间",
    preview_status_initialized: "预览状态已初始化。",
    installed_preview: "预览模式下已接受安装计划。",
  },
};

type I18nValue = {
  lang: Lang;
  setLang: (lang: Lang) => void;
  t: (key: string, vars?: Record<string, string | number>) => string;
};

const I18nContext = createContext<I18nValue | null>(null);

function interpolate(template: string, vars?: Record<string, string | number>) {
  if (!vars) return template;
  return template.replace(/\{(\w+)\}/g, (_, key: string) => String(vars[key] ?? ""));
}

function getInitialLang(): Lang {
  if (typeof window === "undefined") return "en";
  const stored = window.localStorage.getItem(STORAGE_KEY);
  return stored === "zh" || stored === "en" ? stored : "en";
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [lang, setLang] = useState<Lang>(getInitialLang);

  useEffect(() => {
    if (typeof window !== "undefined") {
      window.localStorage.setItem(STORAGE_KEY, lang);
    }
  }, [lang]);

  const value = useMemo<I18nValue>(() => ({
    lang,
    setLang,
    t: (key, vars) => interpolate(dictionary[lang][key] ?? dictionary.en[key] ?? key, vars),
  }), [lang]);

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n() {
  const value = useContext(I18nContext);
  if (!value) {
    throw new Error("useI18n must be used within I18nProvider.");
  }
  return value;
}

export function translateRuntimeKind(runtime: RuntimeKind, lang: Lang) {
  if (lang === "zh") {
    switch (runtime) {
      case "native-cli":
        return "原生 CLI";
      case "docker-compose":
        return "Docker Compose";
      case "external-compose":
        return "外部 Compose";
      case "local-web":
        return "本地网页";
      case "webview":
        return "WebView";
      case "mcp-server":
        return "MCP 服务";
      case "agent-container":
        return "Agent 容器";
    }
  }
  return runtime;
}

export function translateInstallState(state: RecipeInstallState, lang: Lang) {
  if (lang === "zh") {
    switch (state) {
      case "not_installed":
        return "未安装";
      case "installed":
        return "已安装";
      case "installing":
        return "安装中";
      case "error":
        return "错误";
    }
  }
  return state;
}

export function translateRunState(state: RecipeRunState, lang: Lang) {
  if (lang === "zh") {
    switch (state) {
      case "stopped":
        return "已停止";
      case "starting":
        return "启动中";
      case "running":
        return "运行中";
      case "stopping":
        return "停止中";
      case "error":
        return "错误";
      case "unknown":
        return "未知";
    }
  }
  return state;
}

export function translateRecipeText(recipe: OpenNestRecipeSummary, lang: Lang) {
  if (lang !== "zh") {
    return {
      name: recipe.name,
      summary: recipe.summary,
      description: recipe.description ?? recipe.summary,
      category: recipe.category,
    };
  }

  const map: Record<string, { name: string; summary: string; description: string; category: string }> = {
    openclaw: {
      name: "OpenClaw",
      summary: "一键在本地运行 OpenClaw。",
      description: "OpenClaw 是本地 AI 助手网关。OpenNest 负责安装、密钥、网关生命周期、日志与面板打开。",
      category: "本地 AI 助手",
    },
    "open-webui": {
      name: "Open WebUI",
      summary: "通过 Docker Compose 在本地运行 Open WebUI。",
      description: "本地 AI 模型 Web UI。OpenNest 负责 Docker Compose 生命周期和控制面板打开。",
      category: "AI 对话界面",
    },
    flowise: {
      name: "Flowise",
      summary: "通过 Docker Compose 在本地运行 Flowise。",
      description: "可视化 AI 工作流工具。OpenNest 负责本地 compose 生命周期。",
      category: "AI 工作流",
    },
    dify: {
      name: "Dify",
      summary: "通过官方 Docker Compose 方式自托管 Dify。",
      description: "Dify 是更复杂的多服务栈。OpenNest 通过 external-compose 策略接入，而不是手写大 compose。",
      category: "Agent 工作流平台",
    },
    "demo-local-web": {
      name: "本地 Web Demo",
      summary: "用于验证本地 Web 应用接入路径的演示应用。",
      description: "用于验证本地 Web 集成链路的演示 recipe。",
      category: "演示",
    },
  };

  return map[recipe.id] ?? {
    name: recipe.name,
    summary: recipe.summary,
    description: recipe.description ?? recipe.summary,
    category: recipe.category,
  };
}
