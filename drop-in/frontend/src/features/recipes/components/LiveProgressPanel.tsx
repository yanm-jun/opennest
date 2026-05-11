import type { PreviewRecipeApp } from "../types";

export function LiveProgressPanel({ app }: { app: PreviewRecipeApp }) {
  const activeIndex = Math.max(app.progressStages.findIndex((stage) => stage.state === "active"), 0);
  const percent = Math.round(((activeIndex + 1) / app.progressStages.length) * 100);

  return (
    <section className="rounded-[28px] border border-slate-200 bg-white p-6 shadow-sm">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <h3 className="text-lg font-semibold text-slate-950">Live Progress</h3>
          <p className="mt-1 text-sm text-slate-600">Static preview of progress stages keyed off the current app selection.</p>
        </div>
        <div className="rounded-2xl bg-slate-50 px-4 py-3 text-sm text-slate-700">
          <div className="font-medium text-slate-900">{app.name}</div>
          <div className="mt-1">Current stage: {app.progressStages[activeIndex]?.label}</div>
        </div>
      </div>

      <div className="mt-5 h-3 overflow-hidden rounded-full bg-slate-100">
        <div className="h-full rounded-full bg-orange-500 transition-all" style={{ width: `${percent}%` }} />
      </div>

      <div className="mt-5 grid gap-3 lg:grid-cols-5">
        {app.progressStages.map((stage) => (
          <div
            key={stage.id}
            className={`rounded-2xl border px-4 py-3 ${
              stage.state === "complete"
                ? "border-emerald-200 bg-emerald-50"
                : stage.state === "active"
                  ? "border-orange-200 bg-orange-50"
                  : "border-slate-200 bg-slate-50"
            }`}
          >
            <div className="text-xs font-semibold uppercase tracking-wide text-slate-500">{stage.label}</div>
            <div className="mt-2 text-sm font-medium text-slate-900">{stage.state}</div>
            <p className="mt-2 text-sm leading-6 text-slate-600">{stage.description}</p>
          </div>
        ))}
      </div>
    </section>
  );
}
