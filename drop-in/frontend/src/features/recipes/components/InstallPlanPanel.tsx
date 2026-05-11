import type { PreviewRecipeApp, PreflightGateState } from "../types";

export function InstallPlanPanel({
  app,
  gateState,
  onAccept,
  onClear,
}: {
  app: PreviewRecipeApp;
  gateState: PreflightGateState;
  onAccept: () => void;
  onClear: () => void;
}) {
  const accepted = gateState === "accepted";

  return (
    <section className="rounded-[28px] border border-slate-200 bg-white p-6 shadow-sm">
      <div className="grid gap-6 xl:grid-cols-[1.4fr_0.9fr]">
        <div>
          <h3 className="text-lg font-semibold text-slate-950">Install Plan</h3>
          <p className="mt-1 text-sm text-slate-600">Preview what OpenNest will prepare before runtime install commands are connected.</p>

          <div className="mt-4 space-y-3">
            {app.installPlanPreview.map((step, index) => (
              <div key={step.id} className="rounded-2xl bg-slate-50 px-4 py-3">
                <div className="flex items-start gap-3">
                  <div className="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-white text-xs font-semibold text-slate-700">
                    {index + 1}
                  </div>
                  <div>
                    <div className="text-sm font-medium text-slate-900">{step.label}</div>
                    <p className="mt-1 text-sm leading-6 text-slate-600">{step.description}</p>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        <div className="rounded-[24px] border border-slate-200 bg-[#fcfcfa] p-5">
          <h3 className="text-lg font-semibold text-slate-950">Preflight Gate</h3>
          <p className="mt-1 text-sm text-slate-600">Front-end state only for now. Later this can connect to backend plan acceptance commands.</p>

          <div className={`mt-4 rounded-2xl border px-4 py-3 text-sm ${accepted ? "border-emerald-200 bg-emerald-50 text-emerald-900" : "border-amber-200 bg-amber-50 text-amber-900"}`}>
            <div className="font-medium">{accepted ? "Plan accepted" : "Review required"}</div>
            <p className="mt-1">
              {accepted
                ? "The current preview session allows follow-up runtime actions."
                : "Install and start should remain gated until the user reviews the current plan."}
            </p>
          </div>

          <div className="mt-4 flex flex-wrap gap-3">
            <button
              type="button"
              onClick={onAccept}
              className="rounded-2xl bg-slate-900 px-4 py-2.5 text-sm font-medium text-white"
            >
              Accept plan
            </button>
            <button
              type="button"
              onClick={onClear}
              className="rounded-2xl border border-slate-200 bg-white px-4 py-2.5 text-sm font-medium text-slate-800"
            >
              Clear acceptance
            </button>
          </div>
        </div>
      </div>
    </section>
  );
}
