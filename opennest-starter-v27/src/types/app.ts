export type OpenNestAppStatus =
  | "not-installed"
  | "installing"
  | "installed"
  | "starting"
  | "running"
  | "stopped"
  | "failed";

export type OpenNestInstallType =
  | "docker"
  | "node"
  | "python"
  | "binary"
  | "custom";

export type OpenNestAppSourceType =
  | "open-source"
  | "closed-source"
  | "docker"
  | "local-web"
  | "script"
  | "template";

export type OpenNestAppPermissions = {
  fileAccess: boolean;
  networkAccess: boolean;
  browserControl: boolean;
  desktopControl: boolean;
  canPostContent: boolean;
  canReadClipboard: boolean;
  canWriteFiles: boolean;
};

export type OpenNestSystemRequirements = {
  os: string[];
  minMemoryGB?: number;
  minDiskGB?: number;
  gpuRequired?: boolean;
  dockerRequired?: boolean;
  nodeRequired?: boolean;
  pythonRequired?: boolean;
};

export type OpenNestInstallRecipe = {
  appId: string;
  installType: OpenNestInstallType;
  repoUrl?: string;
  dockerComposeFile?: string;
  installCommands?: string[];
  startCommands?: string[];
  stopCommands?: string[];
  healthCheckUrl?: string;
  defaultPort?: number;
  envTemplate?: Record<string, string>;
};

export type OpenNestAppManifest = {
  id: string;
  name: string;
  tagline: string;
  description: string;
  category: string;
  icon?: string;
  screenshots?: string[];
  version: string;
  sourceType: OpenNestAppSourceType;
  installType: OpenNestInstallType;
  repoUrl?: string;
  homepageUrl?: string;
  docsUrl?: string;
  localUrl?: string;
  permissions: OpenNestAppPermissions;
  systemRequirements: OpenNestSystemRequirements;
  installRecipe: OpenNestInstallRecipe;
};

export type OpenNestRuntimeConfig = {
  appId: string;
  installDir: string;
  localUrl?: string;
  port?: number;
  status: OpenNestAppStatus;
  lastStartedAt?: string;
  lastStoppedAt?: string;
  lastError?: string;
};