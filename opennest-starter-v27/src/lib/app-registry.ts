import { apps } from "../data/apps";

/** Return every registered app manifest. */
export function listApps() {
  return apps;
}

/**
 * Look up an app by id.
 * Throws a descriptive error when the app is not found so callers never
 * silently fall back to hardcoded defaults.
 */
export function getAppOrThrow(appId: string) {
  const app = apps.find((item) => item.id === appId);
  if (!app) {
    throw new Error(`App not found: ${appId}. Check src/data/apps.ts and registry/apps.json.`);
  }
  return app;
}