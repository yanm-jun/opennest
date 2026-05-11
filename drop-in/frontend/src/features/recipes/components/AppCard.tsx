import { CheckCircle2, CircleDashed, Cpu, ShieldAlert } from "lucide-react";
import type { PreviewRecipeApp } from "../types";

function statusTone(status: PreviewRecipeApp["status"]) {
  switch (status) {
    case "running":
      return "bg-emerald-100 text-emerald-700";
    case "installed":
      return "bg-sky-100 text-sky-700";
    case "planned":
      return "bg-amber-100 text-amber-700";
    default:
      return "bg-slate-100 text-slate-700";
  }
}

function riskTone(riskLevel: PreviewRecipeApp["riskLevel"]) {
  switch (riskLevel) {
    case "high":
      return "text-rose-700 bg-rose-50";
    case "medium":
      return "text-amber-700 bg-amber-50";
    default:
      return "text-emerald-700 bg-emerald-50";
  }
}

function initials(name: string) {
  return name
    .split(" ")
    .map((part) => part[0])
    .join("")
    .slice(0, 2)
    .toUpperCase();
}

export function AppCard({
  app,
  selected,
  onSelect,
}: {
  app: PreviewRecipeApp;
  selected: boolean;
  onSelect: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onSelect}
      className={`rounded-[26px] border p-5 text-left shadow-sm transition ${
        selected
          ? "border-orange-300 bg-orange-50/70 ring-1 ring-orange-200"
          : "border-slate-200 bg-white hover:-translate-y-0.5 hover:border-slate-300"
      }`}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-center gap-3">
          <div className={`flex h-12 w-12 items-center justify-center rounded-2xl text-sm font-semibold ${selected ? "bg-orange-500 text-white" : "bg-slate-100 text-slate-700"}`}>
            {initials(app.name)}
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h3 className="text-base font-semibold text-slate-950">{app.name}</h3>
              {app.badge ? (
                <span className="rounded-full bg-white px-2 py-0.5 text-[11px] font-medium text-slate-600">
                  {app.badge}
                </span>
              ) : null}
            </div>
            <p className="mt-0.5 text-xs text-slate-500">{app.runtimeLabel}</p>
          </div>
        </div>

        <span className={`rounded-full px-2.5 py-1 text-[11px] font-medium ${statusTone(app.status)}`}>
          {app.status}
        </span>
      </div>

      <p className="mt-4 text-sm leading-6 text-slate-600">{app.tagline}</p>

      <div className="mt-5 grid gap-2 text-xs text-slate-600 sm:grid-cols-2">
        <div className="flex items-center gap-2 rounded-2xl bg-slate-50 px-3 py-2">
          <Cpu className="h-3.5 w-3.5" />
          <span>{app.runtime}</span>
        </div>
        <div className={`flex items-center gap-2 rounded-2xl px-3 py-2 ${riskTone(app.riskLevel)}`}>
          <ShieldAlert className="h-3.5 w-3.5" />
          <span>Risk {app.riskLevel}</span>
        </div>
        <div className="flex items-center gap-2 rounded-2xl bg-slate-50 px-3 py-2">
          <CheckCircle2 className="h-3.5 w-3.5" />
          <span>{app.health}</span>
        </div>
        <div className="flex items-center gap-2 rounded-2xl bg-slate-50 px-3 py-2">
          <CircleDashed className="h-3.5 w-3.5" />
          <span>Port {app.port}</span>
        </div>
      </div>
    </button>
  );
}
