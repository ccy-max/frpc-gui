// 浏览器预览模式 - 仅在开发构建且非 Tauri 环境时加载 Mock
// 生产构建 (import.meta.env.PROD) 会通过 Tree-Shaking 完全剔除该分支
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import Antd from 'ant-design-vue';
import 'ant-design-vue/dist/reset.css';
import App from './App.vue';
import router from './router';
import './styles/global.scss';

async function bootstrap() {
  // 双重守卫：仅开发模式 + 非 Tauri 运行时才注入 Mock
  const isDev = import.meta.env.DEV;
  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
  if (isDev && !isTauri) {
    console.log('🎭 [DEV] Browser preview mode: loading Tauri API mocks');
    await import('./mock-tauri');
  }

  const app = createApp(App);
  const pinia = createPinia();
  app.use(pinia);
  app.use(router);
  app.use(Antd);
  app.mount('#app');
}

bootstrap();
