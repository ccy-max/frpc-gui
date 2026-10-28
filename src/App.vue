<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue';
import { theme as antdThemeApi } from 'ant-design-vue';
import zhCN from 'ant-design-vue/es/locale/zh_CN';
import enUS from 'ant-design-vue/es/locale/en_US';
import { useAppStore } from '@/stores/app';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

const appStore = useAppStore();

// antd 主题联动（A1）：dark class 只影响自绘样式，
// ConfigProvider 才让 Modal/Dropdown/message/DatePicker 等全部组件跟随
const antdTheme = computed(() => ({
  algorithm: appStore.isDarkMode ? antdThemeApi.darkAlgorithm : antdThemeApi.defaultAlgorithm,
  token: {
    colorPrimary: '#2563eb',
    borderRadius: 8,
  },
}));
const antdLocale = computed(() => (appStore.language === 'en-US' ? enUS : zhCN));

// 拦截右键菜单和刷新快捷键（防止 WebView 重新加载导致内存状态全部丢失）
// 生产环境禁用；开发环境保留右键方便调试
function onContextMenu(e: MouseEvent) {
  if (import.meta.env.PROD) {
    e.preventDefault();
  }
}

function onKeyDown(e: KeyboardEvent) {
  // F12 - 开启开发者工具（生产环境也放行，方便调试）
  if (e.key === 'F12') {
    return; // 不拦截，让 WebView 默认处理
  }
  // F5 / Ctrl+R / Ctrl+Shift+R / Cmd+R（Mac） - 阻止页面刷新
  if (e.key === 'F5' ||
      (e.ctrlKey && e.key.toLowerCase() === 'r') ||
      (e.metaKey && e.key.toLowerCase() === 'r')) {
    e.preventDefault();
  }
  // Ctrl+Shift+I - 开启开发者工具（备用快捷键）
  if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === 'i') {
    return; // 不拦截
  }
}

onMounted(async () => {
  document.addEventListener('contextmenu', onContextMenu);
  document.addEventListener('keydown', onKeyDown);

  appStore.init();
  // 延迟加载配置，避免阻塞渲染
  setTimeout(() => {
    appStore.loadConfig().catch(console.error);
  }, 100);

  // 拦截窗口关闭按钮：弹三选项对话框（停止并退出 / 最小化到托盘 / 取消）
  //
  // 历史 bug：原实现用 confirm()（仅"确定/取消"两按钮），语义含糊 ——
  //   点"取消"或对话框的 × → 什么都不做，但后端 close_shown 标志已被置 true，
  //   下次再点关闭按钮会直接放行 → 窗口关掉但 frpc 进程继续跑（残留）。
  // 现在后端每次关闭都 prevent_close（无标志），前端用三选项明确表达意图：
  //   Yes    = 停止 frpc 并退出
  //   No     = 最小化到托盘（frpc 继续运行）
  //   Cancel = 取消关闭（保持窗口，frpc 继续运行）
  await listen('window-close-requested', async () => {
    const { message } = await import('@tauri-apps/plugin-dialog');
    let choice: string;
    try {
      choice = await message(
        '请选择关闭方式：\n\n' +
        '• 停止并退出 —— 终止 frpc 进程后关闭程序\n' +
        '• 最小化到托盘 —— 程序隐藏，frpc 继续运行\n' +
        '• 取消 —— 保持窗口打开',
        {
          title: '关闭 FRPC GUI',
          kind: 'info',
          buttons: { yes: '停止并退出', no: '最小化到托盘', cancel: '取消' },
        },
      );
    } catch {
      // 对话框异常时保守处理：不关闭窗口（避免误杀/残留）
      return;
    }

    if (choice === 'Yes') {
      // 停止并退出：后端先杀 frpc（含孤儿）再 process::exit
      await invoke('exit_app').catch((e) => console.error('退出失败:', e));
    } else if (choice === 'No') {
      // 最小化到托盘：frpc 继续运行
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().hide().catch(console.error);
    }
    // choice === 'Cancel'：什么都不做，窗口保持打开
  });

  // 托盘"退出"菜单：后端已同步杀掉 frpc，这里只补一个确认提示
  // （后端 kill 后再 emit，前端弹窗只是告知，不阻塞退出）
  await listen('app-quit-requested', () => {
    console.info('[app] 托盘退出，frpc 进程已清理');
  });

  // 实时日志：监听后端 frpc-log 事件（frpc stdout → log channel → emit → 前端）
  // 历史 bug：后端正确 emit 了 frpc-log 事件，但前端从未 listen → 实时日志永远空白
  await listen('frpc-log', (event) => {
    const payload = event.payload as { line: string; timestamp: number };
    appStore.appendLiveLog(payload.line, payload.timestamp);
  });
});

onUnmounted(() => {
  document.removeEventListener('contextmenu', onContextMenu);
  document.removeEventListener('keydown', onKeyDown);
  appStore.cleanup();
});
</script>

<template>
  <a-config-provider :theme="antdTheme" :locale="antdLocale">
    <router-view v-slot="{ Component }">
      <transition name="page" mode="out-in">
        <component :is="Component" />
      </transition>
    </router-view>
  </a-config-provider>
</template>

<style>
html, body, #app {
  margin: 0;
  padding: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

/* 路由切换过渡（C3）：fade + 轻微上移，尊重 reduced-motion */
.page-enter-active, .page-leave-active {
  transition: opacity 0.18s ease, transform 0.18s ease;
}
.page-enter-from { opacity: 0; transform: translateY(8px); }
.page-leave-to { opacity: 0; }
@media (prefers-reduced-motion: reduce) {
  .page-enter-active, .page-leave-active { transition: none; }
}
</style>
