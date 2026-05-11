import { useEffect, useMemo, useState } from "react";
import { OpenNestRecipeRegistry } from "./recipeRegistry";
import {
  checkRecipeHealth,
  getRecipeStatus,
  listRecipes,
  openRecipeDashboard,
  readRecipeLogs,
  repairRecipe,
  restartRecipe,
  startRecipe,
  stopRecipe,
  uninstallRecipe,
} from "./recipeRuntimeApi";
import { RecipeStatusBadge } from "./RecipeStatusBadge";
import type { OpenNestRecipeSummary, RecipeStatus, RuntimeActionResult } from "./types";
import { translateInstallState, translateRecipeText, translateRunState, translateRuntimeKind, useI18n } from "../../i18n";

type ActionResultLike = Pick<RuntimeActionResult, "ok" | "message" | "error">;

type Section = {
  id: string;
  title: string;
  description: string;
  empty: string;
  apps: OpenNestRecipeSummary[];
};

export function MyLibraryRecipes({
  onOpenDetails,
  onOpenErrorCenter,
}: {
  onOpenDetails: (appId: string) => void;
  onOpenErrorCenter: () => void;
}) {
  const { lang, t } = useI18n();
  const [apps, setApps] = useState<OpenNestRecipeSummary[]>(OpenNestRecipeRegistry);
  const [statuses, setStatuses] = useState<Record<string, RecipeStatus>>({});
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [logsByAppId, setLogsByAppId] = useState<Record<string, string[]>>({});

  async function loadApps() {
    try {
      const loaded = await listRecipes();
      if (loaded.length) {
        setApps(loaded);
        return loaded;
      }
    } catch {
      // Browser preview / backend unavailable: keep bundled frontend registry as a safe fallback.
    }
    setApps(OpenNestRecipeRegistry);
    return OpenNestRecipeRegistry;
  }

  async function refresh() {
    const nextApps = await loadApps();
    const pairs = await Promise.all(nextApps.map(async (app) => [app.id, await getRecipeStatus(app.id)] as const));
    setStatuses(Object.fromEntries(pairs));
  }

  async function runAction(label: string, appId: string, action: () => Promise<ActionResultLike>) {
    setBusy(`${label}:${appId}`);
    setMessage(null);
    try {
      const result = await action();
      if (result && result.ok === false) {
        const error = result.error;
        setMessage(
          error
            ? [error.message, error.detail && error.detail !== error.message ? `Detail: ${error.detail}` : null, error.nextAction ? `Next: ${error.nextAction}` : null, error.code ? `Code: ${error.code}` : null]
                .filter(Boolean)
                .join("\n")
            : `${label} failed.`,
        );
      } else {
        setMessage(result?.message ?? `${label} completed.`);
      }
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      await refresh().catch(() => undefined);
      setBusy(null);
    }
  }

  async function openLogs(appId: string) {
    try {
      const logs = await readRecipeLogs(appId);
      setLogsByAppId((current) => ({ ...current, [appId]: logs }));
      setMessage(`${t("view_logs")}: ${appId}`);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    }
  }

  useEffect(() => {
    refresh().catch(() => undefined);
  }, []);

  const relevantApps = useMemo(
    () =>
      apps.filter((app) => {
        const status = statuses[app.id];
        if (!status) return false;
        return status.installed || status.runState === "running" || status.runState === "error" || status.installState === "installing" || status.installState === "error" || status.planReviewed;
      }),
    [apps, statuses],
  );

  const sections = useMemo<Section[]>(() => {
    const running = relevantApps.filter((app) => statuses[app.id]?.runState === "running");
    const failed = relevantApps.filter((app) => {
      const status = statuses[app.id];
      return status?.runState === "error" || status?.installState === "error";
    });
    const incomplete = relevantApps.filter((app) => {
      const status = statuses[app.id];
      return status?.installState === "installing" || (!!status?.planReviewed && !status.installed);
    });
    const installed = relevantApps.filter((app) => {
      const status = statuses[app.id];
      return status?.installed && status.runState !== "running" && status.runState !== "error" && status.installState !== "error";
    });
    return [
      { id: "running", title: t("running_apps"), description: t("running_apps_desc"), empty: t("no_running_apps"), apps: running },
      { id: "failed", title: t("failed_apps"), description: t("failed_apps_desc"), empty: t("no_failed_apps"), apps: failed },
      { id: "incomplete", title: t("incomplete_installs"), description: t("incomplete_installs_desc"), empty: t("no_incomplete_installs"), apps: incomplete },
      { id: "installed", title: t("installed_apps"), description: t("installed_apps_desc"), empty: t("no_installed_apps"), apps: installed },
    ];
  }, [relevantApps, statuses, t]);

  if (!relevantApps.length) {
    return <div className="surface-card p-6 text-[14px] leading-[22px] text-[var(--fg-soft)]">{t("no_local_apps_managed_yet")}</div>;
  }

  return (
    <section className="grid gap-4">
      <div className="surface-card p-6">
        <div className="flex flex-col gap-4 xl:flex-row xl:items-end xl:justify-between">
          <div className="flex min-w-0 flex-col gap-3">
            <p className="text-[12px] font-medium leading-[18px] text-[var(--fg-muted)]">{t("local_library")}</p>
            <h2 className="text-[24px] font-semibold leading-[32px] text-[var(--fg-strong)]">{t("installed_runtimes_and_local_state")}</h2>
            <p className="max-w-[64ch] text-[14px] leading-[22px] text-[var(--fg-soft)]">
              {t("installed_runtimes_and_local_state_desc")}
            </p>
          </div>
          <button onClick={() => refresh()} className="button-secondary h-10">{t("refresh_library")}</button>
        </div>
        {message ? <div className="surface-muted-card mt-4 p-4 whitespace-pre-wrap text-[12px] leading-[18px] text-[var(--fg-soft)]">{message}</div> : null}
      </div>

      {sections.map((section) => (
        <section key={section.id} className="surface-card p-6">
          <div className="flex flex-col gap-3 xl:flex-row xl:items-end xl:justify-between">
            <div className="flex flex-col gap-2">
              <h3 className="text-[20px] font-semibold leading-[28px] text-[var(--fg-strong)]">{section.title}</h3>
              <p className="text-[12px] leading-[18px] text-[var(--fg-soft)]">{section.description}</p>
            </div>
            <span className="ui-pill ui-pill-neutral">{t("app_count", { count: section.apps.length })}</span>
          </div>

          {section.apps.length === 0 ? (
            <p className="mt-4 text-[14px] leading-[22px] text-[var(--fg-soft)]">{section.empty}</p>
          ) : (
            <div className="mt-4 grid gap-4">
              {section.apps.map((app) => {
                const status = statuses[app.id];
                if (!status) return null;
                const recipe = translateRecipeText(app, lang);
                const dashboardUrl = app.runtime === "native-cli" ? `Embedded ${app.name} Desktop window` : status.effectiveDashboardUrl ?? status.dashboardUrl ?? "N/A";
                const logs = logsByAppId[app.id];
                return (
                  <article key={app.id} className="surface-panel p-6">
                    <div className="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
                      <div className="flex min-w-0 flex-col gap-4">
                        <div className="flex flex-col gap-2">
                          <h4 className="text-[16px] font-medium leading-[22px] text-[var(--fg-strong)]">{recipe.name}</h4>
                          <p className="text-[12px] leading-[18px] text-[var(--fg-soft)]">{translateRuntimeKind(app.runtime, lang)}</p>
                        </div>
                        <div className="flex flex-wrap gap-2">
                          <RecipeStatusBadge state={status.runState} />
                          <MetaBadge label={`${t("install_state")}: ${translateInstallState(status.installState, lang)}`} tone={status.installState === "error" ? "error" : status.installState === "installing" ? "warning" : "neutral"} />
                          {status.healthState ? <MetaBadge label={`${t("health")}: ${status.healthState}`} tone={status.healthState === "unhealthy" ? "error" : status.healthState === "healthy" ? "success" : "neutral"} /> : null}
                        </div>
                        <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
                          <LibraryField label={t("install_path")} value={status.installDir ?? "N/A"} mono />
                          <LibraryField label={t("current_port")} value={app.runtime === "native-cli" ? "Managed internally" : status.effectivePort ? String(status.effectivePort) : "N/A"} mono={app.runtime !== "native-cli"} />
                          <LibraryField label={t("dashboard_url")} value={dashboardUrl} mono />
                          <LibraryField label={t("last_start")} value={status.lastStartedAt ?? "N/A"} mono />
                          <LibraryField label={t("last_health_check")} value={status.healthCheckedAt ?? "N/A"} mono />
                          <LibraryField label={t("logs_path")} value={status.logsPath ?? "N/A"} mono />
                        </div>
                      </div>
                      <div className="grid gap-4 sm:grid-cols-2 xl:w-[296px]">
                        <ActionButton disabled={!!busy} onClick={() => runAction(t("open_dashboard"), app.id, () => openRecipeDashboard(app.id))}>{t("open_dashboard")}</ActionButton>
                        <ActionButton disabled={!!busy} onClick={() => runAction(t("start"), app.id, () => startRecipe(app.id))}>{t("start")}</ActionButton>
                        <ActionButton disabled={!!busy} onClick={() => runAction(t("stop"), app.id, () => stopRecipe(app.id))}>{t("stop")}</ActionButton>
                        <ActionButton disabled={!!busy} onClick={() => runAction(t("restart"), app.id, () => restartRecipe(app.id))}>{t("restart")}</ActionButton>
                        <ActionButton disabled={!!busy} onClick={() => runAction(t("check_health"), app.id, () => checkRecipeHealth(app.id))}>{t("check_health")}</ActionButton>
                        <ActionButton disabled={!!busy} onClick={() => openLogs(app.id)}>{t("view_logs")}</ActionButton>
                        <ActionButton disabled={!!busy} onClick={() => runAction(t("repair"), app.id, () => repairRecipe(app.id))}>{t("repair")}</ActionButton>
                        <ActionButton disabled={!!busy} onClick={() => runAction(t("uninstall"), app.id, () => uninstallRecipe(app.id, false))}>{t("uninstall")}</ActionButton>
                        {status.runtimeError || status.lastError || status.runState === "error" || status.installState === "error" ? (
                          <ActionButton onClick={onOpenErrorCenter}>{t("error_center")}</ActionButton>
                        ) : null}
                        <button onClick={() => onOpenDetails(app.id)} className="button-primary h-10">{t("open_detail")}</button>
                      </div>
                    </div>
                    {logs ? (
                      <pre className="mt-4 max-h-72 overflow-auto rounded-[10px] bg-[var(--gray-900)] p-6 text-[12px] leading-[18px] text-slate-100">
                        {logs.length ? logs.join("\n") : t("no_logs_yet")}
                      </pre>
                    ) : null}
                  </article>
                );
              })}
            </div>
          )}
        </section>
      ))}
    </section>
  );
}

function ActionButton({
  children,
  disabled,
  onClick,
}: {
  children: string;
  disabled?: boolean;
  onClick: () => void;
}) {
  return (
    <button disabled={disabled} onClick={onClick} className="button-secondary h-10 disabled:cursor-not-allowed disabled:opacity-50">
      {children}
    </button>
  );
}

function MetaBadge({
  label,
  tone,
}: {
  label: string;
  tone: "neutral" | "success" | "warning" | "error";
}) {
  const classes = tone === "success" ? "ui-pill-success" : tone === "warning" ? "ui-pill-warning" : tone === "error" ? "ui-pill-danger" : "ui-pill-neutral";
  return <span className={`ui-pill ${classes}`}>{label}</span>;
}

function LibraryField({
  label,
  value,
  mono = false,
}: {
  label: string;
  value: string;
  mono?: boolean;
}) {
  return (
    <div className="surface-muted-card p-6">
      <div className="text-[12px] font-medium leading-[18px] text-[var(--fg-muted)]">{label}</div>
      <div className={`mt-3 break-all text-[14px] leading-[22px] text-[var(--fg-soft)] ${mono ? "font-mono text-[12px] leading-[18px]" : ""}`}>{value}</div>
    </div>
  );
}
