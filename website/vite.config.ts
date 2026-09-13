import { reactRouter } from "@react-router/dev/vite";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";

export default defineConfig({
  plugins: [
    tailwindcss(),
    reactRouter(),
    tsconfigPaths({ projects: ["./tsconfig.json"], ignoreConfigErrors: true }),
  ],
  server: {
    host: "127.0.0.1",
    port: 4398,
    strictPort: true,
    allowedHosts: ["nutorch.test"],
  },
});
