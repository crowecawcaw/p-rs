import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { fileURLToPath } from "node:url";

export default defineConfig({
  // Relative asset URLs, so the same build works at the site root and under pr/<n>/.
  base: "./",
  plugins: [react()],
  // Demo models are read straight from the repository (tests/, design/).
  server: { fs: { allow: [fileURLToPath(new URL("..", import.meta.url))] } },
  worker: { format: "es" },
  build: { chunkSizeWarningLimit: 1000 },
});
