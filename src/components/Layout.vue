<script setup lang="ts">
import { ref } from 'vue';
import { useAppStore } from '@/stores/app';
import { useI18n } from 'vue-i18n';

const appStore = useAppStore();
const { t } = useI18n();

const activeMenu = ref('dashboard');

const menuItems = [
  { key: 'dashboard', icon: 'DataBoard', label: t('nav.dashboard') },
  { key: 'servers', icon: 'Server', label: t('nav.servers') },
  { key: 'proxies', icon: 'Connection', label: t('nav.proxies') },
  { key: 'versions', icon: 'Download', label: t('nav.versions') },
  { key: 'logs', icon: 'Document', label: t('nav.logs') },
  { key: 'settings', icon: 'Setting', label: t('nav.settings') },
  { key: 'about', icon: 'InfoFilled', label: t('nav.about') },
];
</script>

<template>
  <el-container class="app-container">
    <!-- 侧边栏 -->
    <el-aside width="220px" class="app-aside">
      <div class="logo">
        <el-icon :size="28" color="var(--el-color-primary)"><Monitor /></el-icon>
        <span class="logo-text">{{ t('common.appTitle') }}</span>
      </div>
      
      <el-menu
        :default-active="activeMenu"
        class="app-menu"
        @select="activeMenu = $event"
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
        <el-tag :type="appStore.runningServersCount > 0 ? 'success' : 'info'" size="small">
          {{ t('dashboard.runningServers') }}: {{ appStore.runningServersCount }}
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
  background: var(--el-bg-color);
  border-right: 1px solid var(--el-border-color-light);
  display: flex;
  flex-direction: column;
  
  .logo {
    height: 60px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    border-bottom: 1px solid var(--el-border-color-light);
    
    .logo-text {
      font-size: 18px;
      font-weight: 600;
      color: var(--el-text-color-primary);
    }
  }
  
  .app-menu {
    flex: 1;
    border-right: none;
    padding-top: 8px;
    
    :deep(.el-menu-item) {
      height: 48px;
      margin: 4px 8px;
      border-radius: 8px;
      
      &:hover {
        background-color: var(--el-fill-color-light);
      }
      
      &.is-active {
        background-color: var(--el-color-primary-light-9);
        color: var(--el-color-primary);
      }
    }
  }
  
  .aside-footer {
    padding: 16px;
    border-top: 1px solid var(--el-border-color-light);
  }
}

.app-main {
  padding: 0;
  overflow: hidden;
  background: var(--el-fill-color-blank);
}
</style>
