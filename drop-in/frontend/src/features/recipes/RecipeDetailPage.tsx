import { useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";
import { getRecipeSummary } from "./recipeRegistry";
import { RecipeStatusBadge } from "./RecipeStatusBadge";
import type {
  InstallPlanItem,
  OpenNestRecipeSummary,
  RecipeInstallPlan,
  RecipeProgressEvent,
  RecipeStatus,
  RuntimeActionResult,
  ResourcePreflightReport,
} from "./types";
import {
  acceptRecipeInstallPlan,
  checkRecipeEnvironment,
  checkRecipeHealth,
  checkRecipeReadiness,
  clearRecipeInstallPlanAcceptance,
  getRecipeInstallPlan,
  getRecipeStatus,
  installRecipe,
  listenToRecipeProgress,
  openRecipeDashboard,
  readRecipeLogs,
  repairRecipe,
  resolveRecipePorts,
  rollbackFailedInstall,
  runRecipeDoctor,
  runRecipeOnboarding,
  runResourcePreflight,
  saveRecipeSecrets,
  startRecipe,
  stopRecipe,
  restartRecipe,
  uninstallRecipe,
} from "./recipeRuntimeApi";

export function RecipeDetailPage({ appId, onBack }: { appId: string; onBack?: () => void }) {
  const fallbackRecipe = useMemo(() => getRecipeSummary(appId), [appId]);
  const [recipe, setRecipe] = useState<OpenNestRecipeSummary | undefined>(fallbackRecipe);
  const [status, setStatus] = useState<RecipeStatus | null>(null);
  const [installPlan, setInstallPlan] = useState<RecipeInstallPlan | null>(null);
  const [logs, setLogs] = useState<string[]>([]);
  const [resourcePreflight, setResourcePreflight] = useState<ResourcePreflightReport | null>(null);
  const [progressEvents, setProgressEvents] = useState<RecipeProgressEvent[]>([]);
  const [busy, setBusy] = useState<string | null>(null);
  const [token, setToken] = useState("");
  const [secretId, setSecretId] = useState("openrouterApiToken");
  const [message, setMessage] = useState<string | null>(null);

  const currentPlanAccepted = Boolean(
    status?.planReviewed &&
      installPlan?.planDigest &&
      status.planDigest === installPlan.planDigest &&
      status.planVersion === installPlan.planVersion,
  );

  async function refresh() {
    try {
      const { listRecipes } = await import("./recipeRuntimeApi");
      const loadedRecipes = await listRecipes();
      const loadedRecipe = loadedRecipes.find((item) => item.id === appId);
      if (loadedRecipe) setRecipe(loadedRecipe);
    } catch {
      setRecipe(fallbackRecipe);
    }

    const nextStatus = await getRecipeStatus(appId);
    const nextPlan = await getRecipeInstallPlan(appId);
    const nextLogs = await readRecipeLogs(appId);
    setStatus(nextStatus);
    setInstallPlan(nextPlan);
    setLogs(nextLogs);
  }

  function isRuntimeActionResult(value: unknown): value is RuntimeActionResult {
    return Boolean(value && typeof value === "object" && "ok" in value && "appId" in value);
  }

  async function run(label: string, fn: () => Promise<unknown>) {
    setBusy(label);
    setMessage(null);
    try {
      const result = await fn();
      if (isRuntimeActionResult(result) && result.status) {
        setStatus(result.status);
      }
      if (isRuntimeActionResult(result) && !result.ok) {
        setMessage(result.error ?? `${label} failed.`);
      } else {
        setMessage(isRuntimeActionResult(result) ? result.message ?? `${label} completed.` : `${label} completed.`);
      }
      await refresh();
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
      await refresh().catch(() => undefined);
    } finally {
      setBusy(null);
    }
  }

  useEffect(() => {
    setRecipe(fallbackRecipe);
    setResourcePreflight(null);
    setProgressEvents([]);
    refresh().catch((error) => setMessage(String(error)));
  }, [appId, fallbackRecipe]);

  useEffect(() => {
    let disposed = false;
    let cleanup: (() => void) | undefined;

    listenToRecipeProgress(appId, (event) => {
      setProgressEvents((items) => [event, ...items].slice(0, 20));
      setStatus((current) => current ? {
        ...current,
        progressState: event.state,
        progressOperation: event.operation,
        progressOperationId: event.operationId,
        progressPhase: event.phase,
        progressMessage: event.message,
        progressPercent: event.percent,
        progressStep: event.step,
        progressTotalSteps: event.totalSteps,
        progressUpdatedAt: event.timestamp,
        progressFinishedAt: event.state === "succeeded" || event.state === "failed" ? event.timestamp : current.progressFinishedAt,
        progressError: event.error ?? current.progressError,
      } : current);
    }).then((unlisten) => {
      if (disposed) {
        unlisten();
      } else {
        cleanup = unlisten;
      }
    }).catch(() => undefined);

    return () => {
      disposed = true;
      cleanup?.();
    };
  }, [appId]);

  if (!recipe) {
    return <div className="rounded-2xl border border-red-200 bg-red-50 p-6 text-red-800">Recipe not found: {appId}</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between gap-4">
        <div>
          {onBack && (
            <button onClick={onBack} className="mb-3 text-sm font-medium text-slate-500 hover:text-slate-950">
              ← Back
            </button>
          )}
          <p className="text-sm font-medium text-slate-500">{recipe.category}</p>
          <h1 className="text-3xl font-semibold text-slate-950">{recipe.name}</h1>
          <p className="mt-2 max-w-2xl text-sm leading-6 text-slate-600">{recipe.summary}</p>
        </div>
        <RecipeStatusBadge state={status?.runState ?? "unknown"} />
      </div>

      {message && <div className="rounded-xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-700">{message}</div>}

      {installPlan && (
        <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
          <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
            <div>
              <h2 className="text-lg font-semibold text-slate-950">Install plan</h2>
              <p className="mt-1 text-sm text-slate-600">Review downloads, commands, ports, secrets, rollback, and risk before installing.</p>
            </div>
            <div className="rounded-xl bg-slate-50 px-4 py-3 text-sm text-slate-700">
              <div><span className="font-medium">Risk:</span> {installPlan.riskLevel}</div>
              <div><span className="font-medium">Time:</span> {installPlan.estimatedTime}</div>
              <div><span className="font-medium">Disk:</span> {installPlan.estimatedDisk}</div>
              <div><span className="font-medium">Plan:</span> {installPlan.planDigest}</div>
              <div><span className="font-medium">Accepted:</span> {currentPlanAccepted ? "Yes" : "No"}</div>
            </div>
          </div>

          <div className="mt-5 grid gap-4 lg:grid-cols-4">
            <PlanFlag label="Network" active={installPlan.requiresNetwork} />
            <PlanFlag label="Docker" active={installPlan.requiresDocker} />
            <PlanFlag label="Node" active={installPlan.requiresNode} />
            <PlanFlag label="Git" active={installPlan.requiresGit} />
          </div>

          <div className="mt-5 grid gap-4 lg:grid-cols-2">
            <PlanList title="Preflight checks" items={installPlan.checks} />
            <PlanList title="Downloads / sources" items={installPlan.downloads} />
            <PlanList title="Commands" items={installPlan.commands} />
            <PlanList title="Directories" items={installPlan.directories} />
            <PlanList title="Secrets" items={installPlan.secrets} empty="No required secrets." />
            <PlanList title="Rollback" items={installPlan.rollback} />
          </div>

          {installPlan.warnings.length > 0 && (
            <div className="mt-5 rounded-xl border border-amber-200 bg-amber-50 p-4 text-sm text-amber-900">
              <div className="font-semibold">Warnings</div>
              <ul className="mt-2 list-disc space-y-1 pl-5">
                {installPlan.warnings.map((warning) => <li key={warning}>{warning}</li>)}
              </ul>
            </div>
          )}

          <div className={`mt-5 rounded-xl border p-4 text-sm ${currentPlanAccepted ? "border-emerald-200 bg-emerald-50 text-emerald-900" : "border-amber-200 bg-amber-50 text-amber-900"}`}>
            <div className="font-semibold">Install confirmation gate</div>
            <p className="mt-1">
              {currentPlanAccepted
                ? `Accepted at ${status?.planAcceptedAt ?? "unknown time"}. Install is unlocked for this exact plan digest.`
                : "You must accept this install plan before OpenNest will run Install."}
            </p>
            <div className="mt-3 flex flex-wrap gap-3">
              <button
                disabled={!!busy || currentPlanAccepted}
                onClick={() => run("Accept install plan", () => acceptRecipeInstallPlan(appId))}
                className="rounded-xl bg-slate-950 px-4 py-2 text-sm font-medium text-white disabled:opacity-50"
              >
                Accept Plan
              </button>
              <button
                disabled={!!busy || !status?.planReviewed}
                onClick={() => run("Clear plan acceptance", () => clearRecipeInstallPlanAcceptance(appId))}
                className="rounded-xl border border-slate-200 bg-white px-4 py-2 text-sm font-medium text-slate-800 disabled:opacity-50"
              >
                Clear Acceptance
              </button>
            </div>
          </div>
        </section>
      )}

      <div className="grid gap-4 lg:grid-cols-3">
        <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm lg:col-span-2">
          <h2 className="text-lg font-semibold text-slate-950">Runtime controls</h2>
          <p className="mt-1 text-sm text-slate-600">Install, start, stop, repair, and open the app without touching the terminal.</p>
          <div className="mt-5 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            <ActionButton disabled={!!busy} onClick={() => run("Refresh install plan", () => getRecipeInstallPlan(appId).then((plan) => { setInstallPlan(plan); return { ok: true, appId, message: "Install plan refreshed." }; }))}>View Install Plan</ActionButton>
            <ActionButton disabled={!!busy} onClick={() => run("Check environment", () => checkRecipeEnvironment(appId))}>Check Environment</ActionButton>
            <ActionButton disabled={!!busy} onClick={() => run("Resource preflight", () => runResourcePreflight(appId).then((report) => { setResourcePreflight(report); return { ok: report.ok, appId, message: report.summary, error: report.ok ? undefined : report.summary }; }))}>Run Resource Preflight</ActionButton>
            <ActionButton disabled={!!busy} onClick={() => run("Resolve ports", () => resolveRecipePorts(appId).then((result) => ({ ok: result.ok, appId, message: result.message, error: result.ok ? undefined : result.message })))}>Resolve Ports</ActionButton>
            <ActionButton disabled={!!busy || !currentPlanAccepted} onClick={() => run("Install", () => installRecipe(appId))}>Install</ActionButton>
            <ActionButton disabled={!!busy} onClick={() => run("Start", () => startRecipe(appId))}>Start</ActionButton>
            <ActionButton disabled={!!busy} onClick={() => run("Stop", () => stopRecipe(appId))}>Stop</ActionButton>
            <ActionButton disabled={!!busy} onClick={() => run("Restart", () => restartRecipe(appId))}>Restart</ActionButton>
            <ActionButton disabled={!!busy} onClick={() => run("Check health", () => checkRecipeHealth(appId))}>Check Health</ActionButton>
            <ActionButton disabled={!!busy} onClick={() => run("Check readiness", () => checkRecipeReadiness(appId))}>Check Readiness</ActionButton>
            <ActionButton disabled={!!busy} onClick={() => run("Open dashboard", () => openRecipeDashboard(appId))}>Open Dashboard</ActionButton>
            <ActionButton disabled={!!busy} onClick={() => run("Doctor", () => runRecipeDoctor(appId))}>Run Doctor</ActionButton>
            {recipe.id === "openclaw" /* FIXME: generalize to any recipe with secrets */ && (
              <ActionButton disabled={!!busy} onClick={() => run("Official onboarding", () => runRecipeOnboarding(appId))}>Run Official Onboarding</ActionButton>
            )}
            <ActionButton disabled={!!busy} onClick={() => run("Repair", () => repairRecipe(appId))}>Repair</ActionButton>
            <ActionButton disabled={!!busy} onClick={() => run("Rollback failed install", () => rollbackFailedInstall(appId))}>Rollback Failed Install</ActionButton>
            <ActionButton disabled={!!busy} onClick={() => {
              if (window.confirm("Uninstall this app but keep app data, logs, compose files, source checkouts, and saved secrets?")) {
                run("Uninstall keep data", () => uninstallRecipe(appId, false));
              }
            }}>Uninstall Keep Data</ActionButton>
            <ActionButton disabled={!!busy} onClick={() => {
              if (window.confirm("This removes OpenNest-managed app data and saved secrets for this app. Docker volumes will also be removed where supported. Continue?")) {
                run("Uninstall remove data", () => uninstallRecipe(appId, true));
              }
            }}>Uninstall + Remove Data</ActionButton>
            <ActionButton disabled={!!busy} onClick={() => refresh()}>Refresh</ActionButton>
          </div>
        </section>

        <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
          <h2 className="text-lg font-semibold text-slate-950">Status</h2>
          <dl className="mt-4 space-y-3 text-sm">
            <Row label="Installed" value={status?.installed ? "Yes" : "No"} />
            <Row label="Install State" value={status?.installState ?? "unknown"} />
            <Row label="Run State" value={status?.runState ?? "unknown"} />
            <Row label="Dashboard" value={status?.dashboardUrl ?? "Not ready"} />
            <Row label="Managed PID" value={status?.pid ? String(status.pid) : "None"} />
            <Row label="Health" value={status?.healthState ?? "Unknown"} />
            <Row label="Readiness" value={status?.readinessState ?? "Unknown"} />
            <Row label="Node Runtime" value={status?.nodeRuntimeSource ? `${status.nodeRuntimeSource} ${status.nodeRuntimeVersion ?? ""}` : "Unknown"} />
            <Row label="Plan Accepted" value={currentPlanAccepted ? "Yes" : "No"} />
            <Row label="Docker Services" value={status?.services?.length ? status.services.join(", ") : "None"} />
            <Row label="Resource Preflight" value={status?.resourcePreflightState ?? "Not checked"} />
            <Row label="Port Resolution" value={status?.portResolutionState ?? "Not checked"} />
            <Row label="Effective Dashboard" value={status?.effectiveDashboardUrl ?? status?.dashboardUrl ?? "Not ready"} />
            <Row label="Effective Readiness" value={status?.effectiveReadinessUrl ?? status?.readinessUrl ?? "Not ready"} />
            <Row label="Port Mappings" value={status?.portMappings?.length ? status.portMappings.map((mapping) => `${mapping.requestedPort}→${mapping.resolvedPort}${mapping.changed ? "*" : ""}`).join(", ") : "None"} />
            <Row label="Progress" value={status?.progressState ? `${status.progressState} ${status.progressPercent ?? 0}%` : "Idle"} />
            <Row label="Progress Phase" value={status?.progressPhase ?? "None"} />
            <Row label="Last Error" value={status?.lastError ?? "None"} />
          </dl>
        </section>
      </div>

      <ProgressPanel status={status} events={progressEvents} />

      {resourcePreflight && <ResourcePreflightPanel report={resourcePreflight} />}

      {recipe.id === "openclaw" /* FIXME: generalize to any recipe with secrets */ && (
        <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
          <h2 className="text-lg font-semibold text-slate-950">Model token</h2>
          <p className="mt-1 text-sm text-slate-600">For OpenClaw, save at least one model provider token. The token is sent to Rust and stored outside localStorage.</p>
          <div className="mt-4 grid gap-3 md:grid-cols-[220px_1fr_auto]">
            <select
              value={secretId}
              onChange={(event) => setSecretId(event.target.value)}
              className="rounded-xl border border-slate-200 px-4 py-2 text-sm outline-none focus:border-slate-400"
            >
              <option value="openrouterApiToken">OpenRouter</option>
              <option value="openaiApiToken">OpenAI</option>
              <option value="anthropicApiToken">Anthropic</option>
              <option value="geminiApiToken">Google Gemini</option>
            </select>
            <input
              value={token}
              onChange={(event) => setToken(event.target.value)}
              placeholder="Paste selected provider token"
              className="min-w-0 rounded-xl border border-slate-200 px-4 py-2 text-sm outline-none focus:border-slate-400"
              type="password"
            />
            <button
              disabled={!token || !!busy}
              onClick={() => run("Save token", () => saveRecipeSecrets(appId, [{ id: secretId, value: token }]).then((result) => { if (result.ok) setToken(""); return result; }))}
              className="rounded-xl bg-slate-950 px-4 py-2 text-sm font-medium text-white disabled:opacity-50"
            >
              Save Token
            </button>
          </div>
        </section>
      )}

      <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold text-slate-950">Logs</h2>
          <button onClick={() => refresh()} className="text-sm font-medium text-slate-600 hover:text-slate-950">Refresh</button>
        </div>
        <pre className="mt-4 max-h-80 overflow-auto rounded-xl bg-slate-950 p-4 text-xs leading-6 text-slate-100">
          {logs.length ? logs.join("\n") : "No logs yet."}
        </pre>
      </section>
    </div>
  );
}

function ActionButton({ children, disabled, onClick }: { children: ReactNode; disabled?: boolean; onClick: () => void }) {
  return (
    <button
      disabled={disabled}
      onClick={onClick}
      className="rounded-xl border border-slate-200 bg-white px-4 py-2 text-sm font-medium text-slate-800 shadow-sm transition hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-50"
    >
      {children}
    </button>
  );
}

function PlanFlag({ label, active }: { label: string; active: boolean }) {
  return (
    <div className={`rounded-xl border px-4 py-3 text-sm ${active ? "border-slate-300 bg-slate-50 text-slate-900" : "border-slate-200 bg-white text-slate-400"}`}>
      <span className="font-medium">{label}</span>: {active ? "Required" : "Not required"}
    </div>
  );
}

function PlanList({ title, items, empty = "No items." }: { title: string; items: InstallPlanItem[]; empty?: string }) {
  return (
    <div className="rounded-xl border border-slate-200 p-4">
      <h3 className="text-sm font-semibold text-slate-950">{title}</h3>
      {items.length === 0 ? (
        <p className="mt-2 text-sm text-slate-500">{empty}</p>
      ) : (
        <ul className="mt-3 space-y-3">
          {items.map((item, index) => (
            <li key={`${item.label}-${index}`} className="text-sm">
              <div className="flex items-start justify-between gap-3">
                <span className="font-medium text-slate-800">{item.label}</span>
                <span className="shrink-0 rounded-full bg-slate-100 px-2 py-0.5 text-xs text-slate-500">{item.required ? "required" : "optional"}</span>
              </div>
              {item.value && <div className="mt-1 break-all text-slate-700">{item.value}</div>}
              {item.description && <div className="mt-1 text-slate-500">{item.description}</div>}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

function ProgressPanel({ status, events }: { status: RecipeStatus | null; events: RecipeProgressEvent[] }) {
  const percent = status?.progressPercent ?? 0;
  return (
    <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <h2 className="text-lg font-semibold text-slate-950">Progress</h2>
          <p className="mt-1 text-sm text-slate-600">Live progress events from the Rust runtime, with the latest progress persisted in status.json.</p>
        </div>
        <div className="rounded-xl bg-slate-50 px-4 py-3 text-sm text-slate-700">
          <div><span className="font-medium">State:</span> {status?.progressState ?? "idle"}</div>
          <div><span className="font-medium">Operation:</span> {status?.progressOperation ?? "none"}</div>
          <div><span className="font-medium">Phase:</span> {status?.progressPhase ?? "none"}</div>
        </div>
      </div>
      <div className="mt-4 h-3 overflow-hidden rounded-full bg-slate-100">
        <div className="h-full rounded-full bg-slate-950 transition-all" style={{ width: `${Math.max(0, Math.min(100, percent))}%` }} />
      </div>
      <div className="mt-2 text-sm text-slate-600">
        {status?.progressMessage ?? "No active progress."} {status?.progressStep != null && status?.progressTotalSteps != null ? `(${status.progressStep}/${status.progressTotalSteps})` : ""}
      </div>
      {status?.progressError && <div className="mt-3 rounded-xl border border-red-200 bg-red-50 p-3 text-sm text-red-900">{status.progressError}</div>}
      <div className="mt-4 space-y-2">
        {events.length === 0 ? (
          <p className="text-sm text-slate-500">No live events yet.</p>
        ) : events.map((event) => (
          <div key={`${event.operationId}-${event.timestamp}-${event.phase}`} className="rounded-xl border border-slate-100 bg-slate-50 p-3 text-sm text-slate-700">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <span className="font-medium text-slate-950">{event.operation} · {event.phase}</span>
              <span>{event.percent}% · {event.state}</span>
            </div>
            <div className="mt-1">{event.message}</div>
            {event.error && <div className="mt-1 text-red-700">{event.error}</div>}
            <div className="mt-1 text-xs text-slate-400">{event.timestamp}</div>
          </div>
        ))}
      </div>
    </section>
  );
}

function ResourcePreflightPanel({ report }: { report: ResourcePreflightReport }) {
  return (
    <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <h2 className="text-lg font-semibold text-slate-950">Resource preflight</h2>
          <p className="mt-1 text-sm text-slate-600">{report.summary}</p>
        </div>
        <div className={`rounded-xl px-4 py-3 text-sm ${report.ok ? "bg-emerald-50 text-emerald-900" : "bg-red-50 text-red-900"}`}>
          <div><span className="font-medium">Blocking:</span> {report.blockingCount}</div>
          <div><span className="font-medium">Warnings:</span> {report.warningCount}</div>
          <div><span className="font-medium">Checked:</span> {report.checkedAt}</div>
        </div>
      </div>
      <div className="mt-4 grid gap-3 lg:grid-cols-2">
        {report.checks.map((check) => (
          <div key={check.id} className={`rounded-xl border p-4 text-sm ${check.status === "pass" ? "border-emerald-200 bg-emerald-50 text-emerald-950" : check.status === "warning" ? "border-amber-200 bg-amber-50 text-amber-950" : "border-red-200 bg-red-50 text-red-950"}`}>
            <div className="flex items-start justify-between gap-3">
              <div className="font-semibold">{check.label}</div>
              <div className="shrink-0 rounded-full bg-white/70 px-2 py-0.5 text-xs">{check.status}{check.required ? " · required" : " · optional"}</div>
            </div>
            <div className="mt-2">{check.message}</div>
            {check.details && <div className="mt-2 break-all text-xs opacity-80">{check.details}</div>}
          </div>
        ))}
      </div>
    </section>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between gap-4">
      <dt className="text-slate-500">{label}</dt>
      <dd className="max-w-40 truncate text-right font-medium text-slate-900" title={value}>{value}</dd>
    </div>
  );
}
