import type { RecipeRunState } from "./types";

export function RecipeStatusBadge({ state }: { state: RecipeRunState }) {
  const label = state.replace("_", " ");
  const classes =
    state === "running"
      ? "bg-emerald-50 text-emerald-700 border-emerald-200"
      : state === "error"
        ? "bg-red-50 text-red-700 border-red-200"
        : state === "starting" || state === "stopping"
          ? "bg-amber-50 text-amber-700 border-amber-200"
          : "bg-slate-50 text-slate-700 border-slate-200";

  return (
    <span className={`inline-flex rounded-full border px-2.5 py-1 text-xs font-medium ${classes}`}>
      {label}
    </span>
  );
}
