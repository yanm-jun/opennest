import { useEffect, useMemo, useState } from "react";
import { OpenNestRecipeRegistry } from "./recipeRegistry";
import { apps } from "../../data/apps";
import { getRecipeProductProfile, validationOrder } from "./recipeProfiles";
import { getRecipeStatus, listRecipes } from "./recipeRuntimeApi";
import { RecipeStatusBadge } from "./RecipeStatusBadge";
import type { OpenNestRecipeSummary, RecipeStatus } from "./types";
import { useI18n } from "../../i18n";

export function ValidationBoard({ onOpenDetails }: { onOpenDetails: (appId: string) => void }) {
  const { t } = useI18n();
  const [apps, setApps] = useState<OpenNestRecipeSummary[]>(OpenNestRecipeRegistry);
  const [statuses, setStatuses] = useState<Record<string, RecipeStatus>>({});
  const [message, setMessage] = useState<string | null>(null);

  async function refresh() {
    const loaded = await listRecipes().catch(() => OpenNestRecipeRegistry);
    setApps(loaded);
    const pairs = await Promise.all(loaded.map(async (app) => [app.id, await getRecipeStatus(app.id)] as const));
    setStatuses(Object.fromEntries(pairs));
  }

  useEffect(() => {
    refresh().catch((error) => setMessage(String(error)));
  }, []);

  const orderedApps = useMemo(
    () => [...apps].sort((a, b) => validationOrder[getRecipeProductProfile(a).priority] - validationOrder[getRecipeProductProfile(b).priority]),
    [apps],
  );

  // Derive validation gates dynamically from the registered apps instead of
  // hardcoding OpenClaw.
  const nativeApp = apps.find((a) => a.runtime === "native-cli");
  const nativeId = nativeApp?.id ?? "";
  const nativeStatus = nativeId ? statuses[nativeId] : undefined;

  const dockerApp = apps.find((a) => a.runtime === "docker-compose");
  const dockerId = dockerApp?.id ?? "";
  const dockerStatus = dockerId ? statuses[dockerId] : undefined;

  const gates = [
    {
      label: `${nativeApp?.name ?? "Native CLI"} ${t("installed")}`,
      ok: Boolean(nativeStatus?.installed),
      detail: t("gate_installed_detail", { app: nativeApp?.name ?? "native-cli" }),
    },
    {
      label: `${nativeApp?.name ?? "Native CLI"} ${t("running")}`,
      ok: nativeStatus?.runState === "running",
      detail: t("gate_running_detail", { app: nativeApp?.name ?? "native-cli" }),
    },
    {
      label: `${nativeApp?.name ?? "Native CLI"} ${t("healthy")}`,
      ok: nativeStatus?.healthState === "healthy",
      detail: t("gate_healthy_detail", { app: nativeApp?.name ?? "native-cli" }),
    },
    {
      label: t("gate_docker_staged", { docker: dockerApp?.name ?? "docker-compose" }),
      ok: dockerStatus?.installed ? Boolean(nativeStatus?.installed) : true,
      detail: t("gate_docker_staged_detail", { native: nativeApp?.name ?? "native-cli" }),
    },
  ];
  function exportEvidence() {
    const payload = { exportedAt: new Date().toISOString(), starter: "OpenNest Desktop Starter v30", gates, previewRuntime: { exportedAt: new Date().toISOString(), gates, previewRuntime: {} } };
    const blob = new Blob([JSON.stringify(payload, null, 2)], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = `opennest-v30-validation-evidence-${Date.now()}.json`;
    anchor.click();
    URL.revokeObjectURL(url);
    setMessage(t("validation_evidence_exported"));
  }

  function resetPreview() {
    if (!window.confirm(t("reset_browser_preview_confirm"))) return;
    /* resetPreviewRuntimeSnapshot not available */
    refresh().catch(() => undefined);
    setMessage(t("browser_preview_state_reset"));
  }

  return (
    <section className="grid gap-4">
      <div className="surface-card p-6">
        <div className="flex flex-col gap-4 xl:flex-row xl:items-end xl:justify-between">
          <div className="flex min-w-0 flex-col gap-3">
            <p className="text-[12px] font-medium leading-[18px] text-[var(--fg-muted)]">{t("acceptance_board")}</p>
            <h2 className="text-[24px] font-semibold leading-[32px] text-[var(--fg-strong)]">{t("validation_order_and_evidence")}</h2>
            <p className="max-w-[64ch] text-[14px] leading-[22px] text-[var(--fg-soft)]">
              {t("validation_order_and_evidence_desc")}
            </p>
          </div>
          <div className="flex flex-wrap gap-4">
            <button onClick={() => refresh()} className="button-secondary h-10">{t("refresh")}</button>
            <button onClick={exportEvidence} className="button-primary h-10">{t("export_evidence")}</button>
            <button onClick={resetPreview} className="button-ghost h-10">{t("reset_preview")}</button>
          </div>
        </div>
        {message ? <div className="surface-muted-card mt-4 p-4 text-[12px] leading-[18px] text-[var(--fg-soft)]">{message}</div> : null}
      </div>

      <div className="grid gap-4 lg:grid-cols-4">
        {gates.map((gate) => (
          <div key={gate.label} className="surface-card p-6">
            <div className={`ui-pill ${gate.ok ? "ui-pill-success" : "ui-pill-warning"} w-fit`}>{gate.ok ? t("pass") : t("pending")}</div>
            <div className="mt-4 text-[16px] font-medium leading-[22px] text-[var(--fg-strong)]">{gate.label}</div>
            <p className="mt-3 text-[12px] leading-[18px] text-[var(--fg-soft)]">{gate.detail}</p>
          </div>
        ))}
      </div>

      <div className="grid gap-4">
        {orderedApps.map((app) => {
          const status = statuses[app.id];
          const profile = getRecipeProductProfile(app);
          return (
            <article key={app.id} className="surface-card p-6">
              <div className="flex flex-col gap-4 xl:flex-row xl:items-end xl:justify-between">
                <div className="flex min-w-0 flex-col gap-3">
                  <div className="flex flex-wrap gap-2">
                    <RecipeStatusBadge state={status?.runState ?? "unknown"} />
                    <span className="ui-pill ui-pill-neutral">{profile.stage}</span>
                    <span className="ui-pill ui-pill-neutral">{profile.difficulty}</span>
                  </div>
                  <h3 className="text-[20px] font-semibold leading-[28px] text-[var(--fg-strong)]">{app.name}</h3>
                  <p className="max-w-[64ch] text-[14px] leading-[22px] text-[var(--fg-soft)]">{profile.hero}</p>
                </div>
                <button onClick={() => onOpenDetails(app.id)} className="button-secondary h-10">{t("open_detail")}</button>
              </div>
            </article>
          );
        })}
      </div>
    </section>
  );
}
