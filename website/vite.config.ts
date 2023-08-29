import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import { resolve } from "path";

const src = resolve(__dirname, "src");
const outDir = resolve(__dirname, "dist");

// https://vitejs.dev/config/
export default defineConfig({
  publicDir: resolve(__dirname, "public"),
  root: src,
  plugins: [react()],
  build: {
    outDir,
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: resolve(src, "index.html"),
        proceedToGitHub: resolve(src, "proceed-to-github.html"),
        playground: resolve(src, "playground", "index.html"),
      },
    },
    target: "esnext",
  },
});
