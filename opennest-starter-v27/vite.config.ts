import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(({ command }) => ({
  plugins: [react()],
  clearScreen: false,
  // Tauri bundles the frontend under a local file origin in release mode.
  // Use relative asset paths there so the app doesn't boot into a blank window.
  base: command === "serve" ? "/" : "./",
  server: {
    host: host || "127.0.0.1",
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
