import { useEffect, useMemo, useState } from "react";
import {
  checkRecipeEnvironment,
  getRecipeStatus,
  listRecipes,
  readRecipeLogs,
  repairRecipe,
  startRecipe,
} from "./recipeRuntimeApi";
import type { OpenNestRecipeSummary, RuntimeActionError } from "./types";
import { translateRecipeText, useI18n } from "../../i18n";

type ErrorCenterItem = {
  app: OpenNestRecipeSummary;
  error: RuntimeActionError;
};

function fallbackError(message: string): RuntimeActionError {
  return {
    code: "RUNTIME_ERROR",
    title: "运行时错误",
    message,
    detail: message,
    likelyCause: "运行时状态已经失败，但没有返回更细的结构化错误。",
    nextAction: "先查看日志，再点 Retry 或 Re-detect。",
    repairable: false,
  };
}

export function ErrorCenter({ onOpenDetails }: { onOpenDetails: (appId: string) => void }) {
  const { lang, t } = useI18n();
  const [items, setItems] = useState<ErrorCenterItem[]>([]);
  const [selectedAppId, setSelectedAppId] = useState<string | null>(null);
  const [logs, setLogs] = useState<string[]>([]);
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState<string | null>(null);

  async function refresh() {
    const apps = await listRecipes();
    const collected = await Promise.all(
      apps.map(async (app) => {
        const status = await getRecipeStatus(app.id);
        const runtimeError = status.runtimeError ?? await getRecipeStatus(app.id) ?? (status.lastError ? fallbackError(status.lastError) : null);
        return runtimeError ? { app, error: runtimeError } : null;
      }),
    );
    const nextItems = collected.filter((item): item is ErrorCenterItem => Boolean(item));
    setItems(nextItems);
    setSelectedAppId((current) => {
      if (current && nextItems.some((item) => item.app.id === current)) return current;
      return nextItems[0]?.app.id ?? null;
    });
  }

  async function refreshLogs(appId: string) {
    const nextLogs = await readRecipeLogs(appId);
    setLogs(nextLogs);
  }

  async function run(label: string, appId: string, action: () => Promise<{ ok?: boolean; message?: string; error?: { message?: string; detail?: string; nextAction?: string } }>) {
    setBusy(`${label}:${appId}`);
    setMessage(null);
    try {
      const result = await action();
      if (result.ok === false && result.error) {
        setMessage([result.error.message, result.error.detail && result.error.detail !== result.error.message ? `Detail: ${result.error.detail}` : null, result.error.nextAction ? `Next: ${result.error.nextAction}` : null].filter(Boolean).join("\n"));
      } else {
        setMessage(result.message ?? `${label} completed.`);
      }
      await refresh();
      await refreshLogs(appId);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(null);
    }
  }

  useEffect(() => {
    refresh().catch((error) => setMessage(String(error)));
  }, []);

  useEffect(() => {
    if (!selectedAppId) {
      setLogs([]);
      return;
    }
    refreshLogs(selectedAppId).catch((error) => setMessage(String(error)));
  }, [selectedAppId]);

  const selected = useMemo(() => items.find((item) => item.app.id === selectedAppId) ?? null, [items, selectedAppId]);

  return (
    <section className="grid gap-4">
      <div className="surface-card p-6">
        <div className="flex flex-col gap-4 xl:flex-row xl:items-end xl:justify-between">
          <div className="flex min-w-0 flex-col gap-3">
            <p className="text-[12px] font-medium leading-[18px] text-[var(--fg-muted)]">{t("failure_center")}</p>
            <h2 className="text-[24px] font-semibold leading-[32px] text-[var(--fg-strong)]">{t("structured_runtime_failures")}</h2>
            <p className="max-w-[64ch] text-[14px] leading-[22px] text-[var(--fg-soft)]">
              {t("structured_runtime_failures_desc")}
            </p>
          </div>
          <button onClick={() => refresh()} className="button-secondary h-10">{t("refresh")}</button>
        </div>
        {message ? <div className="surface-muted-card mt-4 p-4 whitespace-pre-wrap text-[12px] leading-[18px] text-[var(--fg-soft)]">{message}</div> : null}
      </div>

      {items.length === 0 ? (
        <div className="surface-card p-6 text-[14px] leading-[22px] text-[var(--fg-soft)]">{t("no_active_runtime_errors")}</div>
      ) : (
        <div className="grid gap-4 lg:grid-cols-[304px_minmax(0,1fr)]">
          <div className="grid gap-4">
            {items.map((item) => {
              const active = item.app.id === selectedAppId;
              const recipe = translateRecipeText(item.app, lang);
              return (
                <button
                  key={item.app.id}
                  onClick={() => setSelectedAppId(item.app.id)}
                  className={`surface-card p-6 text-left ${active ? "bg-[var(--gray-900)] text-[var(--fg-inverse)] border-[rgba(15,23,42,0.92)]" : ""}`}
                >
                  <div className="flex flex-col gap-2">
                    <span className={`text-[12px] leading-[18px] ${active ? "text-[var(--fg-soft-inverse)]" : "text-[var(--fg-muted)]"}`}>{item.error.code}</span>
                    <div className="text-[16px] font-medium leading-[22px]">{recipe.name}</div>
                    <div className={`text-[12px] leading-[18px] ${active ? "text-[var(--fg-soft-inverse)]" : "text-[var(--fg-soft)]"}`}>{item.error.title}</div>
                  </div>
                </button>
              );
            })}
          </div>

          {selected ? (
            <div className="grid gap-4">
              <div className="surface-card p-6">
                <div className="flex flex-col gap-4 xl:flex-row xl:items-end xl:justify-between">
                  <div className="flex flex-col gap-2">
                    <p className="text-[12px] leading-[18px] text-[var(--fg-muted)]">{translateRecipeText(selected.app, lang).name}</p>
                    <h3 className="text-[24px] font-semibold leading-[32px] text-[var(--fg-strong)]">{selected.error.title}</h3>
                    <p className="text-[14px] leading-[22px] text-[var(--fg-soft)]">{selected.error.message}</p>
                  </div>
                  <div className="flex flex-wrap gap-4">
                    <button disabled={!!busy} onClick={() => run(t("retry"), selected.app.id, () => startRecipe(selected.app.id))} className="button-secondary h-10 disabled:opacity-50">{t("retry")}</button>
                    <button disabled={!!busy || !selected.error.repairable} onClick={() => run(t("repair"), selected.app.id, () => repairRecipe(selected.app.id))} className="button-primary h-10 disabled:opacity-50">{t("repair")}</button>
                    <button disabled={!!busy} onClick={() => run(t("re_detect"), selected.app.id, () => checkRecipeEnvironment(selected.app.id))} className="button-ghost h-10 disabled:opacity-50">{t("re_detect")}</button>
                    <button onClick={() => onOpenDetails(selected.app.id)} className="button-ghost h-10">{t("open_detail")}</button>
                  </div>
                </div>

                <div className="mt-4 grid gap-4 lg:grid-cols-2">
                  <InfoCard label={t("likely_cause")} value={selected.error.likelyCause ?? t("not_provided")} />
                  <InfoCard label={t("next_action")} value={selected.error.nextAction ?? t("review_logs_first")} />
                  <InfoCard label={t("repair_action")} value={selected.error.repairAction ?? t("none")} />
                  <InfoCard label={t("exit_code")} value={selected.error.exitCode != null ? String(selected.error.exitCode) : "N/A"} mono />
                </div>

                <div className="surface-muted-card mt-4 p-6">
                  <div className="text-[12px] font-medium leading-[18px] text-[var(--fg-muted)]">{t("original_error")}</div>
                  <pre className="mt-3 overflow-auto whitespace-pre-wrap text-[12px] leading-[18px] text-[var(--fg-soft)]">{selected.error.detail ?? selected.error.message}</pre>
                </div>

                <div className="mt-4 grid gap-4 lg:grid-cols-2">
                  <InfoCard label={t("stdout")} value={selected.error.stdout ?? "N/A"} mono />
                  <InfoCard label={t("stderr")} value={selected.error.stderr ?? "N/A"} mono />
                </div>
              </div>

              <div className="surface-card p-6">
                <div className="flex items-end justify-between gap-4">
                  <div className="flex flex-col gap-2">
                    <h3 className="text-[20px] font-semibold leading-[28px] text-[var(--fg-strong)]">{t("logs")}</h3>
                    <p className="text-[12px] leading-[18px] text-[var(--fg-soft)]">{t("selected_app_runtime_output")}</p>
                  </div>
                  <button onClick={() => selectedAppId && refreshLogs(selectedAppId)} className="button-ghost h-10">{t("refresh_logs")}</button>
                </div>
                <pre className="mt-4 max-h-96 overflow-auto rounded-[10px] bg-[var(--gray-900)] p-6 text-[12px] leading-[18px] text-slate-100">
                  {logs.length ? logs.join("\n") : t("no_logs_yet")}
                </pre>
              </div>
            </div>
          ) : null}
        </div>
      )}
    </section>
  );
}

function InfoCard({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) {
  return (
    <div className="surface-muted-card p-6">
      <div className="text-[12px] font-medium leading-[18px] text-[var(--fg-muted)]">{label}</div>
      <div className={`mt-3 whitespace-pre-wrap text-[14px] leading-[22px] text-[var(--fg-soft)] ${mono ? "font-mono text-[12px] leading-[18px]" : ""}`}>{value}</div>
    </div>
  );
}
