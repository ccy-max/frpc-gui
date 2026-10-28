<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue';
import { theme as antdThemeApi, message } from 'ant-design-vue';
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

// ===== 关闭确认对话框（自绘 a-modal）=====
// 不用 plugin-dialog：其自定义按钮的返回值是按钮文字，跨平台行为不一致，
// 曾导致三个按钮全部点击无反应。自绘 Modal 行为完全可控。
const closeDialogVisible = ref(false);
const closeActionLoading = ref(false);

async function handleExitWithStop() {
  closeActionLoading.value = true;
  try {
    // 后端先杀 frpc（含孤儿进程）再 process::exit(0)
    await invoke('exit_app');
  } catch (e) {
    console.error('退出失败:', e);
    message.error(`退出失败：${String(e)}`);
    closeActionLoading.value = false;
  }
  // 成功时进程已退出，不会执行到这里
}

async function handleMinimizeToTray() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().hide();
    closeDialogVisible.value = false;
  } catch (e) {
    console.error('最小化失败:', e);
    message.error(`最小化失败：${String(e)}`);
  }
}

function handleCancelClose() {
  // 仅关闭对话框，窗口保持打开，frpc 继续运行
  closeDialogVisible.value = false;
}

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
  // 历史 bug 1：后端 close_shown 标志置 true 后永不复位 →
  //   点 × 或取消后再点关闭会直接放行 → 窗口关闭但 frpc 残留。
  // 历史 bug 2（v1.1.5）：用 plugin-dialog 的 message() + 自定义 buttons，
  //   其返回值是「按钮文字」而非 'Yes'/'No'，导致 choice === 'Yes' 永不成立，
  //   点任何按钮都无反应（用户实测三个按钮全失效）。
  // 修复：改用自绘 a-modal，返回值完全在我们掌控中，跨平台行为一致。
  await listen('window-close-requested', () => {
    closeDialogVisible.value = true;
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

    <!-- 关闭确认对话框（自绘，避免原生对话框返回值不可控） -->
    <a-modal
      v-model:open="closeDialogVisible"
      title="关闭 FRPC GUI"
      :mask-closable="false"
      :keyboard="false"
      :footer="null"
      width="460px"
      centered
      wrap-class-name="close-dialog"
      @cancel="handleCancelClose"
    >
      <div class="close-dialog-body">
        <p class="close-dialog-question">请选择关闭方式：</p>

        <div class="close-option" @click="handleExitWithStop">
          <div class="close-option-icon close-icon-danger">⏻</div>
          <div class="close-option-text">
            <div class="close-option-title">停止并退出</div>
            <div class="close-option-desc">终止所有 frpc 进程后关闭程序</div>
          </div>
          <a-spin v-if="closeActionLoading" size="small" />
        </div>

        <div class="close-option" @click="handleMinimizeToTray">
          <div class="close-option-icon close-icon-primary">—</div>
          <div class="close-option-text">
            <div class="close-option-title">最小化到托盘</div>
            <div class="close-option-desc">程序隐藏到系统托盘，frpc 继续运行</div>
          </div>
        </div>

        <div class="close-option" @click="handleCancelClose">
          <div class="close-option-icon close-icon-muted">←</div>
          <div class="close-option-text">
            <div class="close-option-title">取消</div>
            <div class="close-option-desc">保持窗口打开，不做任何操作</div>
          </div>
        </div>

        <div v-if="appStore.isRunning" class="close-dialog-warning">
          ⚠️ 当前有 {{ appStore.runningServersCount }} 个服务器正在运行、{{ appStore.activeProxiesCount }} 个活跃代理
        </div>
      </div>
    </a-modal>
  </a-config-provider>
</template>

<style>
/* 关闭对话框（非 scoped：Modal 会挂到 body） */
.close-dialog-body {
  padding: 4px 0;
}
.close-dialog-question {
  font-size: 14px;
  color: #64748b;
  margin: 0 0 14px;
}
.close-option {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.16s ease;
  margin-bottom: 10px;
  background: #fff;
}
.close-option:hover {
  border-color: #2563eb;
  background: #f8fafc;
  transform: translateX(2px);
}
.close-option-icon {
  width: 34px;
  height: 34px;
  flex-shrink: 0;
  border-radius: 9px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  font-weight: 700;
}
.close-icon-danger {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}
.close-icon-primary {
  background: rgba(37, 99, 235, 0.1);
  color: #2563eb;
}
.close-icon-muted {
  background: rgba(148, 163, 184, 0.14);
  color: #64748b;
}
.close-option-text {
  flex: 1;
  min-width: 0;
}
.close-option-title {
  font-size: 14px;
  font-weight: 600;
  color: #1e293b;
  line-height: 1.3;
}
.close-option-desc {
  font-size: 12px;
  color: #94a3b8;
  margin-top: 2px;
  line-height: 1.4;
}
.close-dialog-warning {
  margin-top: 4px;
  padding: 9px 12px;
  background: rgba(251, 191, 36, 0.1);
  border-radius: 8px;
  font-size: 12px;
  color: #b45309;
  line-height: 1.5;
}

/* 深色模式 */
html.dark .close-dialog-question { color: #94a3b8; }
html.dark .close-option {
  background: #1e293b;
  border-color: #334155;
}
html.dark .close-option:hover {
  background: #0f172a;
  border-color: #3b82f6;
}
html.dark .close-option-title { color: #e2e8f0; }
html.dark .close-option-desc { color: #64748b; }
html.dark .close-dialog-warning {
  background: rgba(251, 191, 36, 0.14);
  color: #fbbf24;
}
</style>

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
