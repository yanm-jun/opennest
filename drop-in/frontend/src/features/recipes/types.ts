export type RuntimeKind =
  | "native-cli"
  | "docker-compose"
  | "external-compose"
  | "webview"
  | "mcp-server"
  | "agent-container";

export type RecipeInstallState = "not_installed" | "installed" | "installing" | "error";
export type RecipeRunState = "stopped" | "starting" | "running" | "stopping" | "error" | "unknown";

export interface OpenNestRecipeSummary {
  id: string;
  name: string;
  summary: string;
  category: string;
  runtime: RuntimeKind;
  ports?: number[];
  featured?: boolean;
}

export type AppRiskLevel = "low" | "medium" | "high";
export type AppAvailability = "ready" | "planned";
export type AppCenterStatus = "not installed" | "ready to install" | "installed" | "running" | "planned";
export type PreflightGateState = "accepted" | "review_required";

export interface RuntimeGuardItem {
  label: string;
  value: string;
  healthy: boolean;
}

export interface AppCenterInstallStep {
  id: string;
  label: string;
  description: string;
}

export interface AppCenterProgressStage {
  id: "install-plan" | "resource-preflight" | "port-resolution" | "runtime" | "readiness";
  label: string;
  description: string;
  state: "complete" | "active" | "pending";
}

export interface AppCenterCapabilityCard {
  id: string;
  title: string;
  description: string;
}

export interface AppCenterSummaryMetric {
  id: string;
  title: string;
  value: string;
  description: string;
}

export interface PreviewRecipeApp extends OpenNestRecipeSummary {
  description: string;
  runtimeLabel: string;
  tagline: string;
  riskLevel: AppRiskLevel;
  status: AppCenterStatus;
  availability: AppAvailability;
  health: string;
  port: string;
  badge?: string;
  installPlanPreview: AppCenterInstallStep[];
  progressStages: AppCenterProgressStage[];
}


export interface InstallPlanItem {
  label: string;
  value?: string;
  description?: string;
  required: boolean;
}


export interface ResourcePreflightCheck {
  id: string;
  label: string;
  status: "pass" | "warning" | "error" | string;
  required: boolean;
  message: string;
  details?: string;
}

export interface ResourcePreflightReport {
  appId: string;
  checkedAt: string;
  ok: boolean;
  blockingCount: number;
  warningCount: number;
  summary: string;
  checks: ResourcePreflightCheck[];
}


export interface RecipePortMapping {
  host: string;
  requestedPort: number;
  resolvedPort: number;
  changed: boolean;
}

export interface PortResolutionResult {
  appId: string;
  checkedAt: string;
  ok: boolean;
  state: string;
  message: string;
  mappings: RecipePortMapping[];
  dashboardUrl?: string;
  readinessUrl?: string;
  warnings: string[];
}

export interface RecipeInstallPlan {
  appId: string;
  name: string;
  planVersion: string;
  planDigest: string;
  runtime: RuntimeKind | string;
  summary: string;
  installStrategy?: string;
  startStrategy?: string;
  dashboardUrl?: string;
  riskLevel: string;
  estimatedTime: string;
  estimatedDisk: string;
  requiresNetwork: boolean;
  requiresDocker: boolean;
  requiresNode: boolean;
  requiresGit: boolean;
  recommendedMemoryGb?: number;
  recommendedCpu?: number;
  ports: number[];
  downloads: InstallPlanItem[];
  directories: InstallPlanItem[];
  commands: InstallPlanItem[];
  secrets: InstallPlanItem[];
  permissions: InstallPlanItem[];
  checks: InstallPlanItem[];
  rollback: InstallPlanItem[];
  warnings: string[];
  notes: string[];
}


export interface RecipeProgressEvent {
  appId: string;
  operationId: string;
  operation: string;
  phase: string;
  state: "running" | "succeeded" | "failed" | string;
  message: string;
  step: number;
  totalSteps: number;
  percent: number;
  timestamp: string;
  error?: string;
}

export interface RecipeStatus {
  appId: string;
  installed: boolean;
  installState: RecipeInstallState;
  runState: RecipeRunState;
  dashboardUrl?: string;
  lastStartedAt?: string;
  lastStoppedAt?: string;
  lastError?: string;
  pid?: number;
  healthState?: "healthy" | "unhealthy" | "unknown" | string;
  healthCheckedAt?: string;
  readinessState?: "ready" | "not_ready" | "checking" | "unknown" | string;
  readinessCheckedAt?: string;
  readinessUrl?: string;
  readinessStatusCode?: number;
  readinessLatencyMs?: number;
  nodeRuntimeSource?: string;
  nodeRuntimeVersion?: string;
  nodeRuntimePath?: string;
  npmPath?: string;
  planReviewed: boolean;
  planAcceptedAt?: string;
  planVersion?: string;
  planDigest?: string;
  planRiskLevel?: string;
  services?: string[];
  resourcePreflightState?: string;
  resourcePreflightCheckedAt?: string;
  resourcePreflightBlockingCount?: number;
  resourcePreflightWarningCount?: number;
  portResolutionState?: string;
  portResolutionCheckedAt?: string;
  portResolutionMessage?: string;
  portMappings?: RecipePortMapping[];
  effectiveDashboardUrl?: string;
  effectiveReadinessUrl?: string;
  progressState?: string;
  progressOperation?: string;
  progressOperationId?: string;
  progressPhase?: string;
  progressMessage?: string;
  progressPercent?: number;
  progressStep?: number;
  progressTotalSteps?: number;
  progressStartedAt?: string;
  progressUpdatedAt?: string;
  progressFinishedAt?: string;
  progressError?: string;
}

export interface RuntimeActionResult {
  ok: boolean;
  appId: string;
  status?: RecipeStatus;
  message?: string;
  logs?: string[];
  error?: string;
}

export interface RecipeSecretInput {
  id: string;
  value: string;
}
