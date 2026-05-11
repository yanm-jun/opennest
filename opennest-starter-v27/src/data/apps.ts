import type { OpenNestAppManifest } from "../types/app";

/**
 * OpenNest App Registry
 *
 * Every installable Agent / AI App lives here as an OpenNestAppManifest.
 * The platform never hardcodes app-specific logic.
 *
 * To add a new app:
 *   1. Add its manifest to the `apps` array below.
 *   2. Add its recipe JSON under recipes/<appId>/recipe.opennest.json.
 *   3. Register it in registry/apps.json.
 *
 * The appId is the single key that drives install, start, stop, logs and status.
 */
export const apps: OpenNestAppManifest[] = [
  {
    id: "openclaw",
    name: "OpenClaw",
    tagline: "Computer-control AI agent for local desktop automation.",
    description:
      "OpenClaw is a local AI assistant gateway. OpenNest manages install, secrets, gateway lifecycle, logs, and dashboard opening.",
    category: "Desktop Agent",
    version: "0.1.0",
    sourceType: "open-source",
    installType: "node",
    repoUrl: "https://github.com/openclaw/openclaw",
    homepageUrl: "https://github.com/openclaw/openclaw",
    docsUrl: "",
    localUrl: "http://127.0.0.1:18789",
    permissions: {
      fileAccess: true,
      networkAccess: true,
      browserControl: true,
      desktopControl: true,
      canPostContent: false,
      canReadClipboard: false,
      canWriteFiles: true,
    },
    systemRequirements: {
      os: ["windows"],
      minMemoryGB: 4,
      minDiskGB: 2,
      gpuRequired: false,
      dockerRequired: false,
      nodeRequired: true,
      pythonRequired: false,
    },
    installRecipe: {
      appId: "openclaw",
      installType: "node",
      repoUrl: "https://github.com/openclaw/openclaw",
      defaultPort: 18789,
      healthCheckUrl: "http://127.0.0.1:18789",
      envTemplate: {},
    },
  },

  {
    id: "open-webui",
    name: "Open WebUI",
    tagline: "Local AI chat UI managed by Docker Compose.",
    description:
      "A familiar chat interface packaged as a Docker-managed recipe with an explicit port, startup sequence, and dashboard handoff.",
    category: "AI Chat UI",
    version: "0.1.0",
    sourceType: "open-source",
    installType: "docker",
    repoUrl: "https://github.com/open-webui/open-webui",
    homepageUrl: "https://github.com/open-webui/open-webui",
    docsUrl: "",
    localUrl: "http://127.0.0.1:3000",
    permissions: {
      fileAccess: true,
      networkAccess: true,
      browserControl: false,
      desktopControl: false,
      canPostContent: false,
      canReadClipboard: false,
      canWriteFiles: true,
    },
    systemRequirements: {
      os: ["windows", "linux", "macos"],
      minMemoryGB: 8,
      minDiskGB: 10,
      gpuRequired: false,
      dockerRequired: true,
      nodeRequired: false,
      pythonRequired: false,
    },
    installRecipe: {
      appId: "open-webui",
      installType: "docker",
      repoUrl: "https://github.com/open-webui/open-webui",
      dockerComposeFile: "recipes/open-webui/docker-compose.yml",
      defaultPort: 3000,
      healthCheckUrl: "http://127.0.0.1:3000",
      envTemplate: {},
    },
  },

  {
    id: "flowise",
    name: "Flowise",
    tagline: "Build AI agents visually and run Flowise locally.",
    description:
      "Flowise is a visual AI agent builder that runs through Docker Compose. OpenNest writes the compose file and manages lifecycle.",
    category: "Visual Agent Builder",
    version: "0.1.0",
    sourceType: "open-source",
    installType: "docker",
    repoUrl: "https://github.com/FlowiseAI/Flowise",
    homepageUrl: "https://github.com/FlowiseAI/Flowise",
    docsUrl: "",
    localUrl: "http://127.0.0.1:3001",
    permissions: {
      fileAccess: true,
      networkAccess: true,
      browserControl: false,
      desktopControl: false,
      canPostContent: false,
      canReadClipboard: false,
      canWriteFiles: true,
    },
    systemRequirements: {
      os: ["windows", "linux", "macos"],
      minMemoryGB: 4,
      minDiskGB: 5,
      gpuRequired: false,
      dockerRequired: true,
      nodeRequired: false,
      pythonRequired: false,
    },
    installRecipe: {
      appId: "flowise",
      installType: "docker",
      repoUrl: "https://github.com/FlowiseAI/Flowise",
      dockerComposeFile: "recipes/flowise/docker-compose.yml",
      defaultPort: 3001,
      healthCheckUrl: "http://127.0.0.1:3001",
      envTemplate: {},
    },
  },

  {
    id: "dify",
    name: "Dify",
    tagline: "Self-host Dify through its official Docker Compose setup.",
    description:
      "Dify is a large multi-service agentic workflow stack. OpenNest uses the external-compose strategy to clone the official repo and manage lifecycle.",
    category: "Agentic Workflow Platform",
    version: "0.1.0",
    sourceType: "open-source",
    installType: "docker",
    repoUrl: "https://github.com/langgenius/dify",
    homepageUrl: "https://github.com/langgenius/dify",
    docsUrl: "",
    localUrl: "http://127.0.0.1:80",
    permissions: {
      fileAccess: true,
      networkAccess: true,
      browserControl: false,
      desktopControl: false,
      canPostContent: false,
      canReadClipboard: false,
      canWriteFiles: true,
    },
    systemRequirements: {
      os: ["windows", "linux", "macos"],
      minMemoryGB: 8,
      minDiskGB: 25,
      gpuRequired: false,
      dockerRequired: true,
      nodeRequired: false,
      pythonRequired: false,
    },
    installRecipe: {
      appId: "dify",
      installType: "docker",
      repoUrl: "https://github.com/langgenius/dify",
      defaultPort: 80,
      healthCheckUrl: "http://127.0.0.1:80",
      envTemplate: {},
    },
  },
];

/** Look up a single app manifest by id. */
export function getAppById(appId: string): OpenNestAppManifest | undefined {
  return apps.find((app) => app.id === appId);
}