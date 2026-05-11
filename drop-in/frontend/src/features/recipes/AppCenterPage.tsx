import { useEffect, useMemo, useState } from "react";
import { BadgeCheck, Search } from "lucide-react";
import { openRecipeDashboard } from "./recipeRuntimeApi";
import { AppCard } from "./components/AppCard";
import { AppDetailPanel } from "./components/AppDetailPanel";
import { AppSidebar } from "./components/AppSidebar";
import { InstallPlanPanel } from "./components/InstallPlanPanel";
import { LiveProgressPanel } from "./components/LiveProgressPanel";
import { RuntimeControls } from "./components/RuntimeControls";
import { capabilityCards, previewApps, runtimeGuardItems, summaryMetrics } from "./data/previewApps";
import type { PreflightGateState, PreviewRecipeApp } from "./types";

export function AppCenterPage({ onOpenApp }: { onOpenApp?: (appId: string) => void }) {
  const fallbackApp = previewApps[0];
  const [query, setQuery] = useState("");
  const [selectedAppId, setSelectedAppId] = useState(fallbackApp?.id ?? "");
  const [gateStateByApp, setGateStateByApp] = useState<Record<string, PreflightGateState>>({});
  const [actionMessage, setActionMessage] = useState<string | null>(null);

  const filteredApps = useMemo(() => {
    const search = query.trim().toLowerCase();
    if (!search) return previewApps;

    return previewApps.filter((app) =>
      [app.name, app.tagline, app.runtimeLabel, app.category]
        .join(" ")
        .toLowerCase()
        .includes(search),
    );
  }, [query]);

  useEffect(() => {
    if (!filteredApps.some((app) => app.id === selectedAppId)) {
      setSelectedAppId(filteredApps[0]?.id ?? "");
    }
  }, [filteredApps, selectedAppId]);

  const selectedApp: PreviewRecipeApp = filteredApps.find((app) => app.id === selectedAppId)
    ?? previewApps.find((app) => app.id === selectedAppId)
    ?? fallbackApp;

  if (!selectedApp) {
    return (
      <div className="rounded-[28px] border border-slate-200 bg-white p-6 text-sm text-slate-600 shadow-sm">
        No App Center preview data is available.
      </div>
    );
  }

  const gateState = gateStateByApp[selectedApp.id] ?? "review_required";
  const selectedAccepted = gateState === "accepted";

  async function handleDashboard() {
    if (selectedApp.availability === "planned") {
      setActionMessage(`${selectedApp.name} is planned only. Dashboard is not available yet.`);
      return;
    }

    try {
      await openRecipeDashboard(selectedApp.id);
      setActionMessage(`Requested dashboard open for ${selectedApp.name}.`);
      onOpenApp?.(selectedApp.id);
    } catch (error) {
      setActionMessage(error instanceof Error ? error.message : String(error));
    }
  }

  function handlePreviewAction(action: string) {
    const previewOnly = new Set(["install", "start", "check", "stop", "restart", "logs", "rollback", "keep-data", "remove-data"]);
    if (previewOnly.has(action)) {
      setActionMessage(`${selectedApp.name}: "${action}" is UI preview only. Connect this button to recipeRuntimeApi in the host app.`);
      return;
    }

    setActionMessage(`${selectedApp.name}: ${action}`);
  }

  return (
    <div className="min-h-screen bg-[linear-gradient(180deg,#fffdf8_0%,#f5f3ee_100%)] p-6 text-slate-900">
      <div className="mx-auto grid max-w-[1600px] gap-6 xl:grid-cols-[280px_minmax(0,1fr)]">
        <AppSidebar guardItems={runtimeGuardItems} />

        <main className="space-y-6">
          <section className="rounded-[30px] border border-slate-200 bg-white/95 p-6 shadow-sm">
            <div className="flex flex-col gap-5 xl:flex-row xl:items-start xl:justify-between">
              <div>
                <div className="flex flex-wrap gap-2">
                  <span className="rounded-full bg-orange-100 px-3 py-1 text-xs font-medium text-orange-700">Recipe System</span>
                  <span className="rounded-full bg-sky-100 px-3 py-1 text-xs font-medium text-sky-700">Desktop MVP</span>
                </div>
                <h1 className="mt-4 text-3xl font-semibold tracking-tight text-slate-950">OpenNest App Center</h1>
                <p className="mt-2 max-w-3xl text-sm leading-6 text-slate-600">
                  把 GitHub 开源 AI 项目变成普通用户能点开的本地应用
                </p>
              </div>

              <label className="flex w-full max-w-md items-center gap-3 rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3">
                <Search className="h-4 w-4 text-slate-400" />
                <input
                  value={query}
                  onChange={(event) => setQuery(event.target.value)}
                  placeholder="Search apps, runtimes, recipes"
                  className="w-full bg-transparent text-sm text-slate-900 outline-none placeholder:text-slate-400"
                />
              </label>
            </div>

            <div className="mt-6 grid gap-4 md:grid-cols-3">
              {summaryMetrics.map((metric) => (
                <div key={metric.id} className="rounded-[24px] border border-slate-200 bg-[#fcfcfa] p-5">
                  <div className="text-sm font-semibold text-slate-950">{metric.title}</div>
                  <div className="mt-2 text-2xl font-semibold text-slate-900">{metric.value}</div>
                  <p className="mt-2 text-sm leading-6 text-slate-600">{metric.description}</p>
                </div>
              ))}
            </div>
          </section>

          <section className="grid gap-6 2xl:grid-cols-[minmax(0,1.3fr)_420px]">
            <div className="space-y-6">
              <section className="rounded-[30px] border border-slate-200 bg-white p-6 shadow-sm">
                <div className="flex items-center justify-between gap-3">
                  <div>
                    <h2 className="text-xl font-semibold text-slate-950">Featured Apps</h2>
                    <p className="mt-1 text-sm text-slate-600">Recipe-backed app previews for real local runtimes, future adapters, and guarded desktop installs.</p>
                  </div>
                  <div className="inline-flex items-center gap-2 rounded-full bg-slate-100 px-3 py-1 text-xs font-medium text-slate-600">
                    <BadgeCheck className="h-4 w-4" />
                    {filteredApps.length} shown
                  </div>
                </div>

                <div className="mt-5 grid gap-4 lg:grid-cols-2">
                  {filteredApps.map((app) => (
                    <AppCard
                      key={app.id}
                      app={app}
                      selected={app.id === selectedApp.id}
                      onSelect={() => {
                        setSelectedAppId(app.id);
                        setActionMessage(null);
                      }}
                    />
                  ))}
                </div>

                {filteredApps.length === 0 ? (
                  <div className="mt-4 rounded-2xl border border-dashed border-slate-300 bg-slate-50 px-4 py-6 text-sm text-slate-500">
                    No apps match the current search.
                  </div>
                ) : null}
              </section>

              <InstallPlanPanel
                app={selectedApp}
                gateState={gateState}
                onAccept={() => {
                  setGateStateByApp((current) => ({ ...current, [selectedApp.id]: "accepted" }));
                  setActionMessage(`${selectedApp.name}: install plan accepted in local preview state.`);
                }}
                onClear={() => {
                  setGateStateByApp((current) => ({ ...current, [selectedApp.id]: "review_required" }));
                  setActionMessage(`${selectedApp.name}: install plan acceptance cleared.`);
                }}
              />

              <LiveProgressPanel app={selectedApp} />

              <RuntimeControls onAction={handlePreviewAction} />
            </div>

            <div className="space-y-6">
              <AppDetailPanel
                app={selectedApp}
                accepted={selectedAccepted}
                onInstall={() => handlePreviewAction("install")}
                onStart={() => handlePreviewAction("start")}
                onDashboard={() => {
                  void handleDashboard();
                }}
              />

              {actionMessage ? (
                <div className="rounded-[24px] border border-slate-200 bg-white px-5 py-4 text-sm text-slate-700 shadow-sm">
                  {actionMessage}
                </div>
              ) : null}

              <section className="rounded-[28px] border border-slate-200 bg-white p-6 shadow-sm">
                <h3 className="text-lg font-semibold text-slate-950">Platform Capabilities</h3>
                <div className="mt-4 grid gap-3">
                  {capabilityCards.map((card) => (
                    <div key={card.id} className="rounded-2xl bg-slate-50 px-4 py-3">
                      <div className="text-sm font-medium text-slate-900">{card.title}</div>
                      <p className="mt-1 text-sm leading-6 text-slate-600">{card.description}</p>
                    </div>
                  ))}
                </div>
              </section>
            </div>
          </section>
        </main>
      </div>
    </div>
  );
}
