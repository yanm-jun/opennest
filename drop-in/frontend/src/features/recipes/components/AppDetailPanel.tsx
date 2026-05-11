import { ExternalLink, HeartPulse, Network, ShieldCheck, TerminalSquare } from "lucide-react";
import type { PreviewRecipeApp } from "../types";

export function AppDetailPanel({
  app,
  accepted,
  onInstall,
  onStart,
  onDashboard,
}: {
  app: PreviewRecipeApp;
  accepted: boolean;
  onInstall: () => void;
  onStart: () => void;
  onDashboard: () => void;
}) {
  return (
    <section className="rounded-[28px] border border-slate-200 bg-white p-6 shadow-sm">
      <div className="flex items-start justify-between gap-4">
        <div>
          <p className="text-xs font-medium uppercase tracking-[0.22em] text-slate-400">App Detail</p>
          <h2 className="mt-2 text-2xl font-semibold text-slate-950">{app.name}</h2>
          <p className="mt-3 text-sm leading-6 text-slate-600">{app.description}</p>
        </div>
        <span className={`rounded-full px-3 py-1 text-xs font-medium ${app.availability === "planned" ? "bg-amber-100 text-amber-700" : "bg-emerald-100 text-emerald-700"}`}>
          {app.availability === "planned" ? "planned" : "active preview"}
        </span>
      </div>

      <div className="mt-5 grid gap-3 md:grid-cols-2">
        <DetailChip icon={TerminalSquare} label="Runtime" value={app.runtimeLabel} />
        <DetailChip icon={Network} label="Port" value={app.port} />
        <DetailChip icon={HeartPulse} label="Health" value={app.health} />
        <DetailChip icon={ShieldCheck} label="Risk" value={app.riskLevel} />
      </div>

      <div className="mt-6 flex flex-wrap gap-3">
        <button
          type="button"
          onClick={onInstall}
          disabled={app.availability === "planned"}
          className="rounded-2xl bg-slate-900 px-4 py-2.5 text-sm font-medium text-white disabled:cursor-not-allowed disabled:bg-slate-200 disabled:text-slate-500"
        >
          Install
        </button>
        <button
          type="button"
          onClick={onStart}
          disabled={app.availability === "planned" || !accepted}
          className="rounded-2xl border border-slate-200 bg-white px-4 py-2.5 text-sm font-medium text-slate-800 disabled:cursor-not-allowed disabled:opacity-50"
        >
          Start
        </button>
        <button
          type="button"
          onClick={onDashboard}
          disabled={app.availability === "planned"}
          className="inline-flex items-center gap-2 rounded-2xl border border-slate-200 bg-slate-50 px-4 py-2.5 text-sm font-medium text-slate-800 disabled:cursor-not-allowed disabled:opacity-50"
        >
          Dashboard
          <ExternalLink className="h-4 w-4" />
        </button>
      </div>

      <div className={`mt-5 rounded-2xl border px-4 py-3 text-sm ${accepted ? "border-emerald-200 bg-emerald-50 text-emerald-900" : "border-amber-200 bg-amber-50 text-amber-900"}`}>
        {accepted
          ? "Current install plan has been accepted in local preview state."
          : "Install plan review is required before runtime actions should unlock."}
      </div>
    </section>
  );
}

function DetailChip({
  icon: Icon,
  label,
  value,
}: {
  icon: typeof TerminalSquare;
  label: string;
  value: string;
}) {
  return (
    <div className="rounded-2xl bg-slate-50 px-4 py-3">
      <div className="flex items-center gap-2 text-slate-500">
        <Icon className="h-4 w-4" />
        <span className="text-xs font-medium uppercase tracking-wide">{label}</span>
      </div>
      <div className="mt-2 text-sm font-medium text-slate-900">{value}</div>
    </div>
  );
}
