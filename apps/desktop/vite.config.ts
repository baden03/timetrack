import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  // The message catalog lives at the repository root.
  server: { port: 1420, strictPort: true, fs: { allow: ["../.."] } },
});
