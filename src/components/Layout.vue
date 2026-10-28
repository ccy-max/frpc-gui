<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { useAppStore } from '@/stores/app';
import { useRouter, useRoute } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import {
  DashboardOutlined,
  CloudServerOutlined,
  ApiOutlined,
  DownloadOutlined,
  FileTextOutlined,
  SettingOutlined,
  InfoCircleOutlined,
  BugOutlined,
  LineChartOutlined,
  SearchOutlined,
  ThunderboltOutlined,
  MenuFoldOutlined,
  MenuUnfoldOutlined,
} from '@ant-design/icons-vue';

const appStore = useAppStore();
const router = useRouter();
const route = useRoute();
const { t } = useI18n();

// C4：侧边栏可折叠（窄窗口初始折叠为图标模式，之后完全由用户手动控制）
// 历史 bug：onWindowResize 在每次 resize 时强制覆盖 collapsed 值，
// 用户手动展开后任何 resize 事件都会覆盖回去 → "收起后无法展开"
const collapsed = ref(window.innerWidth < 1000);

const activeMenu = ref('dashboard');

// N6：label 存 i18n key，模板用 t() 解析
const menuItems = [
  { key: 'dashboard', icon: DashboardOutlined, label: 'nav.dashboard' },
  { key: 'servers', icon: CloudServerOutlined, label: 'nav.servers' },
  { key: 'proxies', icon: ApiOutlined, label: 'nav.proxies' },
  { key: 'versions', icon: DownloadOutlined, label: 'nav.versions' },
  { key: 'logs', icon: FileTextOutlined, label: 'nav.logs' },
  { key: 'stats', icon: LineChartOutlined, label: 'nav.stats' },
  { key: 'inspector', icon: ThunderboltOutlined, label: 'nav.inspector' },
  { key: 'settings', icon: SettingOutlined, label: 'nav.settings' },
  { key: 'about', icon: InfoCircleOutlined, label: 'nav.about' },
];

watch(() => route.path, (newPath) => {
  const path = newPath.replace('/', '');
  if (path && menuItems.some(item => item.key === path)) {
    activeMenu.value = path;
  }
}, { immediate: true });

const handleMenuSelect = ({ key }: { key: string }) => {
  router.push(`/${key}`);
};

async function openDevtools() {
  try {
    await invoke('open_devtools');
  } catch (e) {
    console.error('打开开发者工具失败:', e);
  }
}

// ===== B1x 命令面板（Ctrl/Cmd+K）=====
const paletteOpen = ref(false);
const paletteKeyword = ref('');
const paletteIndex = ref(0);
const paletteInputRef = ref<HTMLInputElement>();

type Command = { id: string; label: string; hint: string; icon: string; run: () => void };

const allCommands = computed<Command[]>(() => {
  const nav: Command[] = menuItems.map(m => ({
    id: `nav-${m.key}`,
    label: t(m.label),
    hint: '跳转',
    icon: '↗',
    run: () => router.push(`/${m.key}`),
  }));
  const serverOps: Command[] = appStore.servers.flatMap(s => {
    const running = appStore.serverStatuses.get(s.id)?.running;
    const ops: Command[] = [];
    if (running) {
      ops.push({ id: `stop-${s.id}`, label: `停止「${s.name}」`, hint: '服务器操作', icon: '■', run: () => appStore.stopServer(s.id).catch(() => {}) });
      ops.push({ id: `restart-${s.id}`, label: `重启「${s.name}」`, hint: '服务器操作', icon: '⟳', run: () => appStore.restartServer(s.id).catch(() => {}) });
    } else {
      ops.push({ id: `start-${s.id}`, label: `启动「${s.name}」`, hint: '服务器操作', icon: '▶', run: () => appStore.startServer(s.id).catch(() => {}) });
    }
    return ops;
  });
  const misc: Command[] = [
    { id: 'new-server', label: '新建服务器', hint: '操作', icon: '+', run: () => { router.push('/servers'); setTimeout(() => window.dispatchEvent(new CustomEvent('open-add-server')), 100); } },
    { id: 'new-proxy', label: '新建代理', hint: '操作', icon: '+', run: () => { router.push('/proxies'); setTimeout(() => window.dispatchEvent(new CustomEvent('open-add-proxy')), 100); } },
    { id: 'devtools', label: '打开调试控制台', hint: '工具', icon: '⚙', run: openDevtools },
  ];
  return [...serverOps, ...misc, ...nav];
});

const filteredCommands = computed(() => {
  const kw = paletteKeyword.value.trim().toLowerCase();
  if (!kw) return allCommands.value.slice(0, 12);
  return allCommands.value.filter(c => c.label.toLowerCase().includes(kw)).slice(0, 12);
});

function openPalette() {
  paletteOpen.value = true;
  paletteKeyword.value = '';
  paletteIndex.value = 0;
  nextTick(() => paletteInputRef.value?.focus());
}
function closePalette() {
  paletteOpen.value = false;
}
function runCommand(cmd: Command) {
  closePalette();
  cmd.run();
}
function onPaletteKey(e: KeyboardEvent) {
  if (!paletteOpen.value) return;
  if (e.key === 'ArrowDown') {
    e.preventDefault();
    paletteIndex.value = Math.min(paletteIndex.value + 1, filteredCommands.value.length - 1);
  } else if (e.key === 'ArrowUp') {
    e.preventDefault();
    paletteIndex.value = Math.max(paletteIndex.value - 1, 0);
  } else if (e.key === 'Enter') {
    e.preventDefault();
    const cmd = filteredCommands.value[paletteIndex.value];
    if (cmd) runCommand(cmd);
  } else if (e.key === 'Escape') {
    closePalette();
  }
}
function onGlobalKey(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
    e.preventDefault();
    paletteOpen.value ? closePalette() : openPalette();
  }
}
watch(paletteKeyword, () => { paletteIndex.value = 0; });
onMounted(() => {
  window.addEventListener('keydown', onGlobalKey);
  window.addEventListener('keydown', onPaletteKey);
});
onUnmounted(() => {
  window.removeEventListener('keydown', onGlobalKey);
  window.removeEventListener('keydown', onPaletteKey);
});
</script>

<template>
  <a-layout class="app-container">
    <a-layout-sider
      :width="260"
      :collapsed-width="72"
      :collapsed="collapsed"
      :trigger="null"
      collapsible
      class="app-sider"
    >
      <!-- Logo 区域 -->
      <div class="logo">
        <div class="logo-icon">
          <CloudServerOutlined />
        </div>
        <div class="logo-text-wrapper" v-show="!collapsed">
          <span class="logo-text">FRPC GUI</span>
          <span class="logo-subtitle">内网穿透管理</span>
        </div>
      </div>

      <!-- 导航菜单 -->
      <a-menu
        :selected-keys="[activeMenu]"
        :inline-collapsed="collapsed"
        mode="inline"
        theme="dark"
        class="app-menu"
        @select="handleMenuSelect"
      >
        <a-menu-item v-for="item in menuItems" :key="item.key">
          <template #icon>
            <span class="menu-icon">
              <component :is="item.icon" />
            </span>
          </template>
          <span class="menu-label">{{ t(item.label) }}</span>
        </a-menu-item>
      </a-menu>

      <!-- 底部状态 -->
      <div class="sider-footer">
        <div class="status-card">
          <div class="status-indicator">
            <span class="status-dot" :class="appStore.isRunning ? 'running' : 'stopped'"></span>
            <span class="status-label">{{ appStore.isRunning ? '运行中' : '已停止' }}</span>
          </div>
          <div class="status-info">
            <span class="status-value">{{ appStore.servers.length }} 台服务器</span>
            <span class="status-divider">·</span>
            <span class="status-value">{{ appStore.activeProxiesCount }} 个活跃代理</span>
          </div>
        </div>
        <a-button size="small" block class="sider-btn" @click="openPalette" style="margin-top: 8px;">
          <template #icon><SearchOutlined /></template>
          命令面板 <kbd class="kbd-hint">Ctrl K</kbd>
        </a-button>
        <a-button size="small" block class="sider-btn" @click="openDevtools" style="margin-top: 6px;">
          <template #icon><BugOutlined /></template>
          <span v-show="!collapsed">调试控制台</span>
        </a-button>
        <!-- C4：手动折叠切换 -->
        <a-button size="small" block class="sider-btn" @click="collapsed = !collapsed" style="margin-top: 6px;">
          <template #icon><MenuFoldOutlined v-if="!collapsed" /><MenuUnfoldOutlined v-else /></template>
          <span v-show="!collapsed">收起侧栏</span>
        </a-button>
      </div>
    </a-layout-sider>

    <a-layout-content class="app-content">
      <router-view />
    </a-layout-content>

    <!-- B1x 命令面板 -->
    <Teleport to="body">
      <div v-if="paletteOpen" class="palette-mask" @click.self="closePalette">
        <div class="palette-box">
          <input
            ref="paletteInputRef"
            v-model="paletteKeyword"
            class="palette-input"
            placeholder="输入命令、服务器名或跳转页面…（↑↓ 选择，Enter 执行，Esc 关闭）"
            autofocus
          />
          <div class="palette-list">
            <div
              v-for="(cmd, i) in filteredCommands"
              :key="cmd.id"
              class="palette-item"
              :class="{ active: i === paletteIndex }"
              @click="runCommand(cmd)"
              @mouseenter="paletteIndex = i"
            >
              <span class="palette-icon">{{ cmd.icon }}</span>
              <span class="palette-label">{{ cmd.label }}</span>
              <span class="palette-hint">{{ cmd.hint }}</span>
            </div>
            <div v-if="!filteredCommands.length" class="palette-empty">无匹配命令</div>
          </div>
        </div>
      </div>
    </Teleport>
  </a-layout>
</template>

<style scoped lang="scss">
.app-container {
  height: 100vh;
  background: #f8fafc;
}

.app-sider {
  background: linear-gradient(180deg, #0f172a 0%, #1e293b 100%);
  overflow: hidden;
  box-shadow: 4px 0 24px rgba(0, 0, 0, 0.12);
  
  // Logo 区域
  .logo {
    height: 72px;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 20px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(255, 255, 255, 0.02);

    .logo-icon {
      width: 40px;
      height: 40px;
      display: flex;
      align-items: center;
      justify-content: center;
      background: linear-gradient(135deg, rgba(37, 99, 235, 0.3), rgba(124, 58, 237, 0.2));
      border-radius: 12px;
      color: #60a5fa;
      font-size: 22px;
      box-shadow: 0 4px 12px rgba(37, 99, 235, 0.3);
    }

    .logo-text-wrapper {
      display: flex;
      flex-direction: column;

      .logo-text {
        font-size: 18px;
        font-weight: 700;
        color: #ffffff;
        letter-spacing: 0.5px;
        line-height: 1.2;
      }

      .logo-subtitle {
        font-size: 11px;
        color: rgba(255, 255, 255, 0.5);
        margin-top: 2px;
        font-weight: 400;
      }
    }
  }

  // 导航菜单
  .app-menu {
    flex: 1;
    border-right: none !important;
    background: transparent !important;
    padding: 12px 8px;
    
    :deep(.ant-menu-item) {
      height: 48px;
      margin: 4px 8px;
      padding: 0 16px !important;
      border-radius: 10px;
      transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
      display: flex;
      align-items: center;
      
      &:hover {
        background: rgba(255, 255, 255, 0.08) !important;
        transform: translateX(4px);
      }
      
      &.ant-menu-item-selected {
        background: linear-gradient(90deg, rgba(37, 99, 235, 0.25), rgba(37, 99, 235, 0.15)) !important;
        color: #60a5fa !important;
        box-shadow: 0 2px 8px rgba(37, 99, 235, 0.2);
        
        .menu-icon {
          color: #60a5fa !important;
          transform: scale(1.1);
        }
        
        .menu-label {
          font-weight: 600;
        }
      }

      .menu-icon {
        font-size: 18px;
        margin-right: 12px;
        color: rgba(255, 255, 255, 0.65);
        transition: all 0.2s ease;
        display: flex;
        align-items: center;
      }

      .menu-label {
        font-size: 14px;
        color: rgba(255, 255, 255, 0.85);
        font-weight: 500;
        transition: all 0.2s ease;
      }
    }
  }

  // 底部状态
  .sider-footer {
    padding: 16px 20px;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(255, 255, 255, 0.02);

    .status-card {
      padding: 14px 16px;
      background: rgba(255, 255, 255, 0.06);
      border-radius: 12px;
      backdrop-filter: blur(10px);
      border: 1px solid rgba(255, 255, 255, 0.08);

      .status-indicator {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-bottom: 10px;

        .status-dot {
          width: 8px;
          height: 8px;
          border-radius: 50%;
          animation: pulse 2s infinite;

          &.running {
            background: #10b981;
            box-shadow: 0 0 12px rgba(16, 185, 129, 0.6);
          }

          &.stopped {
            background: #64748b;
            box-shadow: 0 0 8px rgba(100, 116, 139, 0.4);
          }
        }

        .status-label {
          color: rgba(255, 255, 255, 0.9);
          font-size: 13px;
          font-weight: 600;
        }
      }

      .status-info {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 12px;
        color: rgba(255, 255, 255, 0.6);

        .status-value {
          font-weight: 500;
        }

        .status-divider {
          color: rgba(255, 255, 255, 0.3);
        }
      }
    }
  }
}

// 脉冲动画
@keyframes pulse {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.6;
    transform: scale(1.1);
  }
}

.app-content {
  overflow-y: auto;
  background: linear-gradient(180deg, #f1f5f9 0%, #f8fafc 100%);
  position: relative;
  // 顶部装饰条
  &::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 4px;
    background: linear-gradient(90deg, #2563eb 0%, #7c3aed 50%, #2563eb 100%);
    opacity: 0.8;
  }
  
  // 背景网格装饰
  &::after {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-image: 
      radial-gradient(circle at 20% 30%, rgba(37, 99, 235, 0.03) 0%, transparent 50%),
      radial-gradient(circle at 80% 70%, rgba(124, 58, 237, 0.03) 0%, transparent 50%);
    pointer-events: none;
  }
}
</style>

<style lang="scss">
/* 命令面板（Teleport 到 body，非 scoped）*/
.sider-btn {
  background: rgba(255, 255, 255, 0.06) !important;
  border-color: rgba(255, 255, 255, 0.15) !important;
  color: rgba(255, 255, 255, 0.7) !important;
  border-radius: 8px !important;
  .kbd-hint {
    float: right;
    font-size: 10px;
    background: rgba(255, 255, 255, 0.12);
    border-radius: 4px;
    padding: 0 5px;
    line-height: 16px;
    margin-top: 1px;
    font-family: monospace;
  }
}
.palette-mask {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.55);
  backdrop-filter: blur(3px);
  z-index: 2000;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 15vh;
}
.palette-box {
  width: 560px;
  max-width: 90vw;
  background: #ffffff;
  border-radius: 14px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.35);
  overflow: hidden;
}
.palette-input {
  width: 100%;
  border: none;
  outline: none;
  padding: 18px 20px;
  font-size: 15px;
  border-bottom: 1px solid #eef2f7;
  background: transparent;
  color: #1e293b;
}
.palette-list {
  max-height: 380px;
  overflow-y: auto;
  padding: 6px;
}
.palette-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  border-radius: 8px;
  cursor: pointer;
  color: #334155;
  &.active { background: linear-gradient(90deg, rgba(37, 99, 235, 0.12), rgba(37, 99, 235, 0.06)); }
  .palette-icon { width: 20px; text-align: center; color: #2563eb; font-weight: 700; }
  .palette-label { flex: 1; font-size: 14px; }
  .palette-hint { font-size: 12px; color: #94a3b8; }
}
.palette-empty { padding: 24px; text-align: center; color: #94a3b8; font-size: 14px; }
html.dark {
  .palette-box { background: #1e293b; }
  .palette-input { color: #e2e8f0; border-bottom-color: #334155; }
  .palette-item {
    color: #cbd5e1;
    &.active { background: rgba(37, 99, 235, 0.25); }
    .palette-icon { color: #60a5fa; }
  }
}
</style>
