<script setup lang="ts">
import { ref } from 'vue';
import { useAppStore } from '@/stores/app';
import { useRouter } from 'vue-router';

const appStore = useAppStore();
const router = useRouter();

const activeMenu = ref('dashboard');

const menuItems = [
  { key: 'dashboard', icon: 'DataBoard', label: '概览' },
  { key: 'servers', icon: 'Server', label: '服务器管理' },
  { key: 'proxies', icon: 'Connection', label: '代理管理' },
  { key: 'versions', icon: 'Download', label: '版本管理' },
  { key: 'logs', icon: 'Document', label: '日志查看' },
  { key: 'settings', icon: 'Setting', label: '设置' },
  { key: 'about', icon: 'InfoFilled', label: '关于' },
];

const handleMenuSelect = (key: string) => {
  activeMenu.value = key;
  router.push(`/${key}`);
};
</script>

<template>
  <el-container class="app-container">
    <!-- 侧边栏 -->
    <el-aside width="220px" class="app-aside">
      <div class="logo">
        <el-icon :size="28" color="var(--el-color-primary)"><Monitor /></el-icon>
        <span class="logo-text">FRPC GUI</span>
      </div>
      
      <el-menu
        :default-active="activeMenu"
        class="app-menu"
        @select="handleMenuSelect"
      >
        <el-menu-item
          v-for="item in menuItems"
          :key="item.key"
          :index="item.key"
        >
          <el-icon><component :is="item.icon" /></el-icon>
          <span>{{ item.label }}</span>
        </el-menu-item>
      </el-menu>
      
      <!-- 底部状态 -->
      <div class="aside-footer">
        <el-tag :type="appStore.isRunning ? 'success' : 'info'" size="small">
          <span class="status-dot" :class="appStore.isRunning ? 'running' : 'stopped'"></span>
          {{ appStore.isRunning ? '运行中' : '已停止' }}
        </el-tag>
      </div>
    </el-aside>

    <!-- 主内容区 -->
    <el-main class="app-main">
      <router-view v-slot="{ Component }">
        <transition name="fade" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </el-main>
  </el-container>
</template>

<style scoped lang="scss">
.app-container {
  height: 100vh;
  width: 100%;
}

.app-aside {
  background: var(--app-sidebar-bg);
  border-right: 1px solid var(--app-border-color);
  display: flex;
  flex-direction: column;
  
  .logo {
    height: 60px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    
    .logo-text {
      font-size: 18px;
      font-weight: 600;
      color: var(--app-sidebar-text);
    }
  }
  
  .app-menu {
    flex: 1;
    border-right: none;
    background: transparent;
    padding-top: 8px;
    
    :deep(.el-menu-item) {
      height: 48px;
      margin: 4px 8px;
      border-radius: 8px;
      color: var(--app-sidebar-text);
      
      &:hover {
        background-color: rgba(255, 255, 255, 0.1);
      }
      
      &.is-active {
        background-color: var(--el-color-primary);
        color: #ffffff;
      }
    }
  }
  
  .aside-footer {
    padding: 16px;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }
}

.app-main {
  padding: 0;
  overflow: hidden;
  background: var(--app-bg-color);
}
</style>
