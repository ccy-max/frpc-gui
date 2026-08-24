<script setup lang="ts">
import { ref } from 'vue';
import { useAppStore } from '@/stores/app';
import { useRouter } from 'vue-router';
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

const handleMenuSelect = (key: string) => {
  activeMenu.value = key;
  router.push(`/${key}`);
};
</script>

<template>
  <a-layout class="app-container">
    <!-- 侧边栏 -->
    <a-layout-sider width="220" class="app-sider" :trigger="null">
      <div class="logo">
        <span class="logo-text">FRPC GUI</span>
      </div>

      <a-menu
        :selected-keys="[activeMenu]"
        mode="inline"
        theme="dark"
        class="app-menu"
        @click="handleMenuSelect"
      >
        <a-menu-item v-for="item in menuItems" :key="item.key">
          <component :is="item.icon" />
          <span>{{ item.label }}</span>
        </a-menu-item>
      </a-menu>

      <!-- 底部状态 -->
      <div class="sider-footer">
        <a-badge
          :status="appStore.isRunning ? 'success' : 'default'"
          :text="appStore.isRunning ? '运行中' : '已停止'"
        />
      </div>
    </a-layout-sider>

    <!-- 主内容区 -->
    <a-layout-content class="app-content">
      <router-view />
    </a-layout-content>
  </a-layout>
</template>

<style scoped lang="scss">
.app-container {
  height: 100vh;
  width: 100%;
}

.app-sider {
  display: flex;
  flex-direction: column;

  .logo {
    height: 60px;
    display: flex;
    align-items: center;
    justify-content: center;

    .logo-text {
      font-size: 18px;
      font-weight: 600;
      color: #ffffff;
    }
  }

  .app-menu {
    flex: 1;
    border-right: none;
    padding-top: 8px;
  }

  .sider-footer {
    padding: 16px 24px;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }
}

.app-content {
  padding: 0;
  overflow: hidden;
  background: var(--app-bg-color);
}
</style>
