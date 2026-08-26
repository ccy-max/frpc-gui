import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import path from "path";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [vue()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  css: {
    preprocessorOptions: {
      scss: {
        additionalData: `@use "@/styles/variables.scss" as *;`,
      },
    },
  },
  clearScreen: false,
  build: {
    // 第三方库分包：ant-design-vue / 图标 独立缓存，业务代码改动不失效长缓存
    rollupOptions: {
      output: {
        manualChunks: {
          "vendor-antd": ["ant-design-vue"],
          "vendor-icons": ["@ant-design/icons-vue"],
          "vendor-tauri": [
            "@tauri-apps/api",
            "@tauri-apps/plugin-dialog",
          ],
        },
      },
    },
    chunkSizeWarningLimit: 700,
  },
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
