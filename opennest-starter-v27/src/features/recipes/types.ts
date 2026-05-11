export type RuntimeKind =
  | "native-cli"
  | "docker-compose"
  | "external-compose"
  | "local-web"
  | "webview"
  | "mcp-server"
  | "agent-container";

export type RecipeInstallState = "not_installed" | "installed" | "installing" | "error";
export type RecipeRunState = "stopped" | "starting" | "running" | "stopping" | "error" | "unknown";

export type SourceType = "open-source" | "closed-source" | "docker" | "local-web" | "script" | "template";

export interface OpenNestRecipeSummary {
  id: string;
  name: string;
  summary: string;
  description?: string;
  category: string;
  runtime: RuntimeKind;
  ports?: number[];
  featured?: boolean;
  icon?: string;
  screenshots?: string[];
  tags?: string[];
  difficulty?: string;
  sourceType?: SourceType;
  priority?: string;
  homepage?: string;
  sourceUrl?: string;
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
  installDir?: string;
  dashboardUrl?: string;
  effectivePort?: number;
  lastStartedAt?: string;
  lastStoppedAt?: string;
  lastError?: string;
  logsPath?: string;
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
  runtimeError?: RuntimeActionError;
}

export interface RuntimeActionResult {
  ok: boolean;
  appId: string;
  status?: RecipeStatus;
  message?: string;
  logs?: string[];
  error?: RuntimeActionError;
}

export interface RecipeSecretInput {
  id: string;
  value: string;
}

export type AppProvider = "deepseek" | "openai" | "openrouter" | "anthropic";

export interface AppSecretsSetupInput {
  provider: AppProvider;
  apiKey: string;
  openChat?: boolean;
}

export interface RuntimeActionError {
  code: string;
  title: string;
  message: string;
  detail?: string;
  likelyCause?: string;
  nextAction?: string;
  repairable: boolean;
  repairAction?: string;
  stdout?: string;
  stderr?: string;
  exitCode?: number;
}
