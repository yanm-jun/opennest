import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const failures = [];

function readJson(relativePath) {
  const full = path.join(root, relativePath);
  try {
    return JSON.parse(fs.readFileSync(full, "utf8"));
  } catch (error) {
    throw new Error(`${relativePath} is not valid JSON: ${error.message}`);
  }
}

const requiredFiles = [
  "package.json",
  "index.html",
  "vite.config.ts",
  "src/main.tsx",
  "src/App.tsx",
  "src/features/recipes/RecipeAppCenter.tsx",
  "src/features/recipes/RecipeDetailPage.tsx",
  "src/features/recipes/ValidationBoard.tsx",
  "src/features/recipes/recipeRegistry.ts",
  "src/features/recipes/recipeProfiles.ts",
  "registry/apps.json",
  "src-tauri/Cargo.toml",
  "src-tauri/src/lib.rs",
  "src-tauri/src/recipe_runtime/recipe_loader.rs"
];

for (const file of requiredFiles) {
  if (!fs.existsSync(path.join(root, file))) failures.push(`Missing required file: ${file}`);
}

try {
  const pkg = readJson("package.json");
  if (!pkg.scripts?.["validate:recipes"]) failures.push("package.json should expose validate:recipes.");
} catch (error) {
  failures.push(error.message);
}

try {
  const registry = readJson("registry/apps.json");
  if (!Array.isArray(registry.apps) || registry.apps.length < 5) {
    failures.push("registry/apps.json should contain at least 5 apps including the demo app.");
  }
  for (const app of registry.apps ?? []) {
    for (const file of ["app.json", "recipe.json", "requirements.json", "install-plan.json", "runtime.json", "README.md"]) {
      const recipeFile = path.join(root, "recipes", app.id, file);
      if (!fs.existsSync(recipeFile)) {
        failures.push(`Missing template file for ${app.id}: recipes/${app.id}/${file}`);
      }
    }
  }
} catch (error) {
  failures.push(error.message);
}

const libText = fs.existsSync(path.join(root, "src-tauri/src/lib.rs")) ? fs.readFileSync(path.join(root, "src-tauri/src/lib.rs"), "utf8") : "";
for (const command of ["recipe_list_apps", "recipe_get_status", "recipe_get_install_plan"]) {
  if (!libText.includes(`commands::${command}`)) failures.push(`Command not registered in lib.rs: ${command}`);
}

if (failures.length) {
  console.error("OpenNest starter verification failed:\n");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}

console.log("OpenNest starter verification passed.");
console.log("Checked core shell files, recipe template layout, registry entries, and template-loader wiring.");
