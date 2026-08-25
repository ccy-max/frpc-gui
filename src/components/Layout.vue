<script setup lang="ts">
import { ref, watch } from 'vue';
import { useAppStore } from '@/stores/app';
import { useRouter, useRoute } from 'vue-router';
import {
  DashboardOutlined,
  CloudServerOutlined,
  ApiOutlined,
  DownloadOutlined,
  FileTextOutlined,
  SettingOutlined,
  InfoCircleOutlined,
} from '@ant-design/icons-vue';

const appStore = useAppStore();
const router = useRouter();
const route = useRoute();

const activeMenu = ref('dashboard');

const menuItems = [
  { key: 'dashboard', icon: DashboardOutlined, label: '概览' },
  { key: 'servers', icon: CloudServerOutlined, label: '服务器管理' },
  { key: 'proxies', icon: ApiOutlined, label: '代理管理' },
  { key: 'versions', icon: DownloadOutlined, label: '版本管理' },
  { key: 'logs', icon: FileTextOutlined, label: '日志查看' },
  { key: 'settings', icon: SettingOutlined, label: '设置' },
  { key: 'about', icon: InfoCircleOutlined, label: '关于' },
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
</script>

<template>
  <a-layout class="app-container">
    <a-layout-sider width="260" class="app-sider" :trigger="null">
      <!-- Logo 区域 -->
      <div class="logo">
        <div class="logo-icon">
          <CloudServerOutlined />
        </div>
        <div class="logo-text-wrapper">
          <span class="logo-text">FRPC GUI</span>
          <span class="logo-subtitle">内网穿透管理</span>
        </div>
      </div>

      <!-- 导航菜单 -->
      <a-menu
        :selected-keys="[activeMenu]"
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
          <span class="menu-label">{{ item.label }}</span>
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
      </div>
    </a-layout-sider>

    <a-layout-content class="app-content">
      <router-view />
    </a-layout-content>
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
}
</style>
