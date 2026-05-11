import { apps } from "../../data/apps";
import type { OpenNestRecipeSummary } from "./types";
import type { RuntimeKind } from "./types";

/**
 * Convert an OpenNestAppManifest (from apps.ts) into the lightweight
 * OpenNestRecipeSummary shape that App Center cards and detail pages consume.
 */
function toSummary(app: (typeof apps)[number]): OpenNestRecipeSummary {
  return {
    id: app.id,
    name: app.name,
    summary: app.tagline,
    description: app.description,
    category: app.category,
    runtime: app.installType === "docker" ? "docker-compose" as RuntimeKind : "native-cli" as RuntimeKind,
    ports: app.installRecipe.defaultPort ? [app.installRecipe.defaultPort] : [],
    featured: app.installType === "node" || (app.installType === "docker" && (app.systemRequirements.minDiskGB ?? 999) <= 10),
    sourceType: app.sourceType,
    icon: app.icon,
    screenshots: app.screenshots,
    tags: [],
    difficulty: app.systemRequirements.dockerRequired ? "Medium" as const : "Easy" as const,
    priority: app.installType === "node" ? "first" as const : app.systemRequirements.minDiskGB && app.systemRequirements.minDiskGB <= 10 ? "second" as const : "third" as const,
    homepage: app.homepageUrl,
    sourceUrl: app.repoUrl,
  };
}

/**
 * Private module-level cache built from apps.ts when the module first loads.
 */
function buildRegistry(): OpenNestRecipeSummary[] {
  return apps.map(toSummary);
}

export const OpenNestRecipeRegistry: OpenNestRecipeSummary[] = buildRegistry();

export function getRecipeSummary(appId: string): OpenNestRecipeSummary | undefined {
  return OpenNestRecipeRegistry.find((r) => r.id === appId);
}