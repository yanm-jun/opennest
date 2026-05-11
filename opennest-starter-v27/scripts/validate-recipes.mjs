import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const recipesRoot = path.join(root, "recipes");
const registryPath = path.join(root, "registry", "apps.json");
const failures = [];
const warnings = [];

function readJson(filePath) {
  try {
    return JSON.parse(fs.readFileSync(filePath, "utf8"));
  } catch (error) {
    throw new Error(`${path.relative(root, filePath)} is not valid JSON: ${error.message}`);
  }
}

const registry = readJson(registryPath);
const registryIds = new Set((registry.apps ?? []).map((entry) => entry.id));
const recipeDirs = fs.readdirSync(recipesRoot, { withFileTypes: true }).filter((entry) => entry.isDirectory()).map((entry) => entry.name);

const requiredFiles = ["app.json", "recipe.json", "requirements.json", "install-plan.json", "runtime.json", "README.md"];
const allowedRuntimeTypes = new Set(["native-cli", "docker-compose", "external-compose", "local-web"]);
const allowedPriorities = new Set(["first", "second", "third", "last"]);

for (const appId of recipeDirs) {
  const dir = path.join(recipesRoot, appId);
  for (const file of requiredFiles) {
    if (!fs.existsSync(path.join(dir, file))) {
      failures.push(`recipes/${appId}/${file} is missing.`);
    }
  }

  if (!registryIds.has(appId)) {
    warnings.push(`recipes/${appId} exists but is not declared in registry/apps.json.`);
  }

  try {
    const app = readJson(path.join(dir, "app.json"));
    const requirements = readJson(path.join(dir, "requirements.json"));
    const recipe = readJson(path.join(dir, "recipe.json"));
    const installPlan = readJson(path.join(dir, "install-plan.json"));
    const runtime = readJson(path.join(dir, "runtime.json"));

    if (app.id !== appId) failures.push(`recipes/${appId}/app.json id must equal directory name.`);
    if (!app.name || !app.category || !app.description) failures.push(`recipes/${appId}/app.json must define name/category/description.`);
    if (app.priority && !allowedPriorities.has(app.priority)) failures.push(`recipes/${appId}/app.json priority must be one of first/second/third/last.`);

    if (!Array.isArray(requirements.ports)) failures.push(`recipes/${appId}/requirements.json must define ports as an array.`);
    if (typeof requirements.dockerRequired !== "boolean") failures.push(`recipes/${appId}/requirements.json must define dockerRequired as boolean.`);
    if (typeof requirements.nodeRequired !== "boolean") failures.push(`recipes/${appId}/requirements.json must define nodeRequired as boolean.`);
    if (typeof requirements.gitRequired !== "boolean") failures.push(`recipes/${appId}/requirements.json must define gitRequired as boolean.`);
    if (typeof requirements.gpuRequired !== "boolean") failures.push(`recipes/${appId}/requirements.json must define gpuRequired as boolean.`);

    if (!allowedRuntimeTypes.has(recipe.runtimeType)) failures.push(`recipes/${appId}/recipe.json runtimeType is invalid: ${recipe.runtimeType}`);
    if (!recipe.install || !recipe.start || !recipe.stop || !recipe.dashboard || !recipe.logs) {
      failures.push(`recipes/${appId}/recipe.json must define install/start/stop/dashboard/logs.`);
    }

    if (!("riskLevel" in installPlan) || !("ports" in installPlan) || !("rollback" in installPlan)) {
      failures.push(`recipes/${appId}/install-plan.json must define riskLevel/ports/rollback.`);
    }

    if (!("installedState" in runtime) || !("effectiveDashboardUrl" in runtime) || !("logsPath" in runtime)) {
      failures.push(`recipes/${appId}/runtime.json must define installedState/effectiveDashboardUrl/logsPath.`);
    }
  } catch (error) {
    failures.push(error.message);
  }
}

for (const app of registry.apps ?? []) {
  if (!recipeDirs.includes(app.id)) {
    failures.push(`registry/apps.json declares ${app.id}, but recipes/${app.id} does not exist.`);
  }
}

if (!registryIds.has("demo-local-web")) {
  failures.push("registry/apps.json must include demo-local-web to prove template-only app onboarding.");
}

if (failures.length) {
  console.error("Recipe validation failed:\n");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}

console.log(`Recipe validation passed for ${recipeDirs.length} recipe directories.`);
if (warnings.length) {
  console.log("\nWarnings:");
  for (const warning of warnings) console.log(`- ${warning}`);
}
