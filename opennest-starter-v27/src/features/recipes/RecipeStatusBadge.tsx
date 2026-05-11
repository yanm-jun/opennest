import type { RecipeRunState } from "./types";

export function RecipeStatusBadge({ state }: { state: RecipeRunState }) {
  const label = state.replace("_", " ");
  const classes =
    state === "running"
      ? "ui-pill-success"
      : state === "error"
        ? "ui-pill-danger"
        : state === "starting" || state === "stopping"
          ? "ui-pill-warning"
          : "ui-pill-neutral";

  return (
    <span className={`ui-pill ${classes}`}>
      {label}
    </span>
  );
}
