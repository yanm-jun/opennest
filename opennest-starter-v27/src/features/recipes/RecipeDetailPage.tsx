import { useEffect, useMemo, useState } from "react";
import { getRecipeSummary } from "./recipeRegistry";
import { RecipeStatusBadge } from "./RecipeStatusBadge";
import { translateInstallState, translateRecipeText, translateRunState, translateRuntimeKind, useI18n } from "../../i18n";
import type {
  OpenNestRecipeSummary,
  RecipeInstallPlan,
  RecipeProgressEvent,
  RecipeStatus,
  RuntimeActionResult,
} from "./types";
import {
  saveRecipeSecrets,
  acceptRecipeInstallPlan,
  checkRecipeHealth,
  clearRecipeInstallPlanAcceptance,
  getRecipeInstallPlan,
  getRecipeStatus,
  installRecipe,
  listenToRecipeProgress,
  openRecipeDashboard,
  readRecipeLogs,
  repairRecipe,
  rollbackFailedInstall,
  startRecipe,
  stopRecipe,
  uninstallRecipe,
} from "./recipeRuntimeApi";

export function RecipeDetailPage({ appId, onBack }: { appId: string; onBack?: () => void }) {
  const { lang, t } = useI18n();
  const fallbackRecipe = useMemo(() => getRecipeSummary(appId), [appId]);
  const [recipe, setRecipe] = useState<OpenNestRecipeSummary | undefined>(fallbackRecipe);
  const [status, setStatus] = useState<RecipeStatus | null>(null);
  const [installPlan, setInstallPlan] = useState<RecipeInstallPlan | null>(null);
  const [logs, setLogs] = useState<string[]>([]);
  const [busy, setBusy] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [progress, setProgress] = useState<RecipeProgressEvent[]>([]);
  const [providerKeys, setProviderKeys] = useState<Record<string, string>>({
    deepseek: "", openai: "", openrouter: "", anthropic: ""
  });
  const [showProviderPanel, setShowProviderPanel] = useState(false);
  const [secretsSaved, setSecretsSaved] = useState(false);
  const [confirmUninstall, setConfirmUninstall] = useState<"keep" | "wipe" | null>(null);

  const planAccepted = status?.planReviewed &&
    installPlan?.planDigest &&
    status.planDigest === installPlan.planDigest;

  const isInstalled = status?.installed;
  const isRunning = status?.runState === "running";
  const hasSecrets = (installPlan?.secrets?.length ?? 0) > 0;

  async function refresh() {
    try {
      const { listRecipes } = await import("./recipeRuntimeApi");
      const loaded = await listRecipes();
      const r = loaded.find((i) => i.id === appId);
      if (r) setRecipe(r);
    } catch {}
    const s = await getRecipeStatus(appId);
    const p = await getRecipeInstallPlan(appId);
    const l = await readRecipeLogs(appId);
    setStatus(s);
    setInstallPlan(p);
    setLogs(l);
  }

  async function action(label: string, fn: () => Promise<unknown>) {
    setBusy(label);
    setMessage(null);
    try {
      const result = await fn() as any;
      if (result?.ok === false && result?.error) {
        const e = result.error;
        setMessage([e.message, e.detail, e.nextAction].filter(Boolean).join(" | "));
      } else {
        setMessage(result?.message ?? `${label} done.`);
      }
      await refresh();
    } catch (e: any) {
      setMessage(e?.message ?? String(e));
    } finally {
      setBusy(null);
    }
  }

  useEffect(() => { refresh(); }, [appId]);

  useEffect(() => {
    let dead = false;
    let unlisten: (() => void) | undefined;
    listenToRecipeProgress(appId, (ev) => {
      if (!dead) setProgress((p) => [ev, ...p].slice(0, 20));
    }).then((u) => { if (dead) u(); else unlisten = u; });
    return () => { dead = true; unlisten?.(); };
  }, [appId]);

  if (!recipe) return <div className="p-6 text-red-600">Recipe not found: {appId}</div>;

  const recipeText = translateRecipeText(recipe, lang);

  return (
    <div className="grid gap-4">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div>
          {onBack && <button onClick={onBack} className="mb-1 text-[12px] text-[var(--fg-soft)] hover:text-[var(--fg-strong)]">← {t("back")}</button>}
          <h1 className="text-[22px] font-semibold">{recipeText.name}</h1>
          <p className="text-[13px] text-[var(--fg-soft)]">{recipeText.category} · {translateRuntimeKind(recipe.runtime, lang)}</p>
        </div>
        <RecipeStatusBadge state={status?.runState ?? "unknown"} />
      </div>

      {message && <div className="rounded-lg border border-[var(--border-soft)] bg-[var(--bg-strong)] px-4 py-2.5 text-[13px] text-[var(--fg-soft)]">{message}</div>}

      {/* Step 1: Install */}
      <StepCard
        step={1}
        title="安装"
        done={!!isInstalled}
        active={!isInstalled}
        detail={isInstalled ? `Installed · ${translateInstallState(status?.installState ?? "", lang)}` : planAccepted ? "计划已确认，可以安装" : "请查看计划后安装"}
      >
        {!isInstalled && (
          <>
            {installPlan && (
              <InstallPlanSummary plan={installPlan} planAccepted={!!planAccepted} />
            )}
            <div className="flex flex-wrap gap-2">
              <Btn onClick={() => action("plan", () => getRecipeInstallPlan(appId).then(p => { setInstallPlan(p); return { ok: true, appId, message: "Plan loaded." }; }))} busy={busy} label="加载计划" />
              <Btn onClick={() => action("accept", () => acceptRecipeInstallPlan(appId))} busy={busy} label="确认计划" disabled={!installPlan || planAccepted} />
              <Btn onClick={() => action("clear", () => clearRecipeInstallPlanAcceptance(appId))} busy={busy} label="清除" secondary disabled={!planAccepted} />
              <Btn onClick={() => action("install", () => installRecipe(appId))} busy={busy} label="安装" primary disabled={!planAccepted} />
            </div>
          </>
        )}
      </StepCard>

      {/* Provider Config (shown when app recipe declares secrets) */}
      {hasSecrets && (
        <StepCard
          step={showProviderPanel ? undefined : 0} title="模型提供商"
          done={false} active={true}
          detail={showProviderPanel ? "在下方配置一个或多个模型提供商" : "为你想要使用的模型提供商设置 API 密钥"}
        >
          <button
            onClick={() => setShowProviderPanel(!showProviderPanel)}
            className="rounded-lg border border-[var(--border-soft)] px-3 py-1.5 text-[12px] font-medium text-[var(--fg-soft)] hover:text-[var(--fg-strong)]"
          >
            {showProviderPanel ? "收起" : secretsSaved ? "API 密钥已保存 ✓" : "配置 API 密钥"}
          </button>
          {showProviderPanel && (
            <div className="mt-3 grid gap-2 sm:grid-cols-2">
              {(["deepseek", "openai", "openrouter", "anthropic"] as const).map((prov) => (
                <div key={prov} className="flex items-center gap-2">
                  <span className="w-20 shrink-0 text-[12px] font-medium capitalize">{prov}</span>
                  <input
                    type="password"
                    value={providerKeys[prov]}
                    onChange={(e) => setProviderKeys((prev) => ({ ...prev, [prov]: e.target.value }))}
                    placeholder="sk-..."
                    className="min-w-0 flex-1 rounded-lg border border-[var(--border-soft)] bg-[var(--bg-strong)] px-3 py-1.5 text-[12px] outline-none focus:border-[var(--fg-strong)]"
                  />
                </div>
              ))}
            </div>
          )}
          {showProviderPanel && (
            <div className="mt-3 flex gap-2">
              <Btn
                onClick={async () => {
                  const secrets = Object.entries(providerKeys)
                    .filter(([, v]) => v.trim())
                    .map(([k, v]) => ({ id: `${k}ApiToken`, value: v }));
                  if (secrets.length === 0) { setMessage("至少输入一个 API 密钥"); return; }
                  await action("save", () => saveRecipeSecrets(appId, secrets));
                  setProviderKeys({ deepseek: "", openai: "", openrouter: "", anthropic: "" });
                  setShowProviderPanel(false);
                }}
                busy={busy} label="保存全部密钥" primary
              />
              <Btn
                onClick={() => { setShowProviderPanel(false); }}
                busy={busy} label="取消" secondary
              />
            </div>
          )}
        </StepCard>
      )}

      {/* 步骤 2: Start / Stop */}
      <StepCard
        step={2}
        title={isRunning ? "运行中" : "启动"}
        done={isRunning}
        active={isInstalled}
        detail={isRunning ? `PID ${status?.pid ?? "?"} · Health: ${status?.healthState ?? "?"}` : isInstalled ? "已安装，点击启动" : "请先安装"}
      >
        {isInstalled && (
          <div className="flex flex-wrap gap-2">
            <Btn onClick={() => action("start", () => startRecipe(appId))} busy={busy} label="启动" primary disabled={isRunning} />
            <Btn onClick={() => action("stop", () => stopRecipe(appId))} busy={busy} label="停止" secondary disabled={!isRunning} />
            <Btn onClick={() => action("health", () => checkRecipeHealth(appId))} busy={busy} label="健康检查" secondary />
            {isRunning && (
              <Btn onClick={() => action("open", () => openRecipeDashboard(appId))} busy={busy} label="打开面板" primary disabled={!isRunning} />
            )}
          </div>
        )}
      </StepCard>

      {/* 步骤 3: Troubleshoot */}
      <StepCard
        step={3}
        title="维护"
        done={false}
        active={isInstalled}
        detail="修复、回滚、卸载或查看日志"
      >
        {isInstalled && (
          <div className="flex flex-wrap gap-2">
            <Btn onClick={() => action("repair", () => repairRecipe(appId))} busy={busy} label="修复" secondary />
            <Btn onClick={() => action("rollback", () => rollbackFailedInstall(appId))} busy={busy} label="回滚" secondary />
            <Btn onClick={() => { if (window.confirm("Uninstall? Data will be kept.")) action("uninstall", () => uninstallRecipe(appId, false)); }} busy={busy} label="卸载" secondary />
            <Btn onClick={() => { if (window.confirm("Remove all data too?")) action("wipe", () => uninstallRecipe(appId, true)); }} busy={busy} label="清除所有数据" danger />
          </div>
        )}
      </StepCard>

      {/* Progress (visible when busy) */}
      {(
        <div className="surface-card p-4">
          <h3 className="text-[14px] font-medium">进度</h3>
          {busy && <p className="text-[12px] text-[var(--fg-soft)]">Busy: {busy}...</p>}
          {progress.slice(0, 5).map((ev) => (
            <div key={ev.operationId + ev.phase} className="mt-2 flex items-center gap-2 text-[12px] text-[var(--fg-soft)]">
              <span className="w-16 truncate font-medium">{ev.operation}</span>
              <span>{ev.phase}</span>
              <span className="ml-auto">{ev.percent}%</span>
            </div>
          ))}
        </div>
      )}

      {/* Logs (collapsible) */}
      {logs.length > 0 && (
        <details className="surface-card p-4">
          <summary className="cursor-pointer text-[13px] font-medium">Logs ({logs.length} lines)</summary>
          <pre className="mt-2 max-h-60 overflow-auto rounded-lg bg-[var(--bg-strong)] p-3 text-[11px] leading-relaxed text-[var(--fg-soft)] font-mono">{logs.join("\n")}</pre>
        </details>
      )}
    </div>
  );
}

function StepCard({ step, title, done, active, detail, children }: {
  step?: number; title: string; done: boolean | undefined; active: boolean | undefined; detail: string; children?: React.ReactNode;
}) {
  return (
    <div className={"surface-card p-4 transition-opacity " + (active ? "" : "opacity-50")}>
      <div className="flex items-start gap-3">
        <span className={"mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-full text-[11px] font-bold " +
          (done ? "bg-green-100 text-green-700" : active ? "bg-[var(--bg-strong)] text-[var(--fg-strong)]" : "bg-gray-100 text-gray-400")}>
          {done ? "✓" : step}
        </span>
        <div className="min-w-0 flex-1">
          <h3 className="text-[15px] font-medium">{title}</h3>
          <p className="text-[12px] text-[var(--fg-soft)]">{detail}</p>
          {children && <div className="mt-3">{children}</div>}
        </div>
      </div>
    </div>
  );
}

function Btn({ onClick, busy, label, disabled, primary, secondary, danger }: {
  onClick: () => void; busy: string | null; label: string;
  disabled?: boolean | ""; primary?: boolean; secondary?: boolean; danger?: boolean | "";
}) {
  let cls = "rounded-lg px-3 py-1.5 text-[12px] font-medium transition disabled:opacity-60 ";
  if (primary) cls += "bg-emerald-600 text-white hover:bg-emerald-700 hover:opacity-90";
  else if (danger) cls += "border border-red-200 text-red-700 hover:bg-red-50";
  else cls += "border border-[var(--border-soft)] text-[var(--fg-soft)] hover:text-[var(--fg-strong)] hover:border-[var(--fg-strong)]";
  return <button onClick={onClick} disabled={!!busy || !!disabled} className={cls}>{busy === label ? `${label}...` : label}</button>;
}

function InstallPlanSummary({ plan, planAccepted }: { plan: RecipeInstallPlan; planAccepted: boolean }) {
  const { t } = useI18n();
  return (
    <div className="mb-3 grid grid-cols-2 gap-2 text-[11px] sm:grid-cols-4">
      <Pill label={t("risk")} value={plan.riskLevel} />
      <Pill label={t("time")} value={plan.estimatedTime} />
      <Pill label={t("disk")} value={plan.estimatedDisk} />
      <Pill label={t("ports")} value={plan.ports?.join(", ") || "-"} />
      <Pill label="Docker" value={plan.requiresDocker ? "Yes" : "No"} />
      <Pill label="Node" value={plan.requiresNode ? "Yes" : "No"} />
      <Pill label="Git" value={plan.requiresGit ? "Yes" : "No"} />
      <Pill label={t("network")} value={plan.requiresNetwork ? "Yes" : "No"} />
    </div>
  );
}

function Pill({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-md border border-[var(--border-soft)] px-2 py-1">
      <span className="text-[var(--fg-muted)]">{label}:</span> <span className="font-medium">{value}</span>
    </div>
  );
}