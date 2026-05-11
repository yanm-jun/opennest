import type { OpenNestRecipeSummary } from "./types";

export const OpenNestRecipeRegistry: OpenNestRecipeSummary[] = [
  {
    id: "openclaw",
    name: "OpenClaw",
    summary: "Run OpenClaw locally with one-click setup.",
    category: "Local AI Assistant",
    runtime: "native-cli",
    ports: [18789],
    featured: true,
  },
  {
    id: "open-webui",
    name: "Open WebUI",
    summary: "Local AI chat UI managed by Docker Compose.",
    category: "AI Chat UI",
    runtime: "docker-compose",
    ports: [3000],
    featured: true,
  },
  {
    id: "flowise",
    name: "Flowise",
    summary: "Visual AI agent builder managed by Docker Compose.",
    category: "Visual Agent Builder",
    runtime: "docker-compose",
    ports: [3001],
    featured: true,
  },
  {
    id: "dify",
    name: "Dify",
    summary: "Large self-hosted agentic workflow stack through official Docker Compose.",
    category: "Agentic Workflow Platform",
    runtime: "external-compose",
    ports: [80],
    featured: false,
  },
];

export function getRecipeSummary(appId: string) {
  return OpenNestRecipeRegistry.find((recipe) => recipe.id === appId);
}
