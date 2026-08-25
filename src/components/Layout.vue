<script setup lang="ts">
import { ref, computed, watch } from 'vue';
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
    <a-layout-sider width="240" class="app-sider" :trigger="null">
      <div class="logo">
        <div class="logo-icon">
          <CloudServerOutlined />
        </div>
        <span class="logo-text">FRPC GUI</span>
      </div>

      <a-menu
        :selected-keys="[activeMenu]"
        mode="inline"
        theme="dark"
        class="app-menu"
        @select="handleMenuSelect"
      >
        <a-menu-item v-for="item in menuItems" :key="item.key">
          <template #icon>
            <component :is="item.icon" />
          </template>
          {{ item.label }}
        </a-menu-item>
      </a-menu>

      <div class="sider-footer">
        <div class="status-card">
          <a-badge :status="appStore.isRunning ? 'success' : 'default'" />
          <span class="status-text">{{ appStore.isRunning ? '运行中' : '已停止' }}</span>
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
  
  .logo {
    height: 64px;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 24px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);

    .logo-icon {
      width: 36px;
      height: 36px;
      display: flex;
      align-items: center;
      justify-content: center;
      background: rgba(37, 99, 235, 0.2);
      border-radius: 10px;
      color: #60a5fa;
      font-size: 20px;
    }

    .logo-text {
      font-size: 18px;
      font-weight: 700;
      color: #ffffff;
      letter-spacing: 0.5px;
    }
  }

  .app-menu {
    flex: 1;
    border-right: none !important;
    background: transparent !important;
    
    :deep(.ant-menu-item) {
      margin: 4px 8px;
      border-radius: 8px;
      transition: all 0.2s ease;
      
      &:hover {
        background: rgba(255, 255, 255, 0.08) !important;
      }
      
      &.ant-menu-item-selected {
        background: linear-gradient(90deg, rgba(37, 99, 235, 0.2), rgba(37, 99, 235, 0.1)) !important;
        color: #60a5fa !important;
        
        .anticon {
          color: #60a5fa !important;
        }
      }
    }
  }

  .sider-footer {
    padding: 16px 24px;
    border-top: 1px solid rgba(255, 255, 255, 0.1);

    .status-card {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 12px;
      background: rgba(255, 255, 255, 0.05);
      border-radius: 10px;

      .status-text {
        color: rgba(255, 255, 255, 0.85);
        font-size: 13px;
        font-weight: 500;
      }
    }
  }
}

.app-content {
  overflow-y: auto;
}
</style>
