<script setup lang="ts">
import { useAppStore } from '@/stores/app';
import { StarOutlined } from '@ant-design/icons-vue';
import { message } from 'ant-design-vue';

const appStore = useAppStore();

const packageInfo = {
  version: '0.1.0',
  author: 'Zero',
  license: 'MIT',
  github: 'https://github.com/ccy-max/frpc-gui',
};

function openUrl(url: string) {
  appStore.openUrl(url);
}

async function checkUpdate() {
  const version = await appStore.checkAppUpdate();
  if (version) {
    message.info(`最新版本：${version}`);
  } else {
    message.warning('检查更新失败');
  }
}
</script>

<template>
  <div class="about-page">
    <a-card class="about-card">
      <div class="logo-section">
        <h1 class="app-name">FRPC GUI</h1>
        <p class="description">FRP 内网穿透桌面管理应用</p>
      </div>
      <a-descriptions :column="1" bordered>
        <a-descriptions-item label="版本">
          <a-tag color="blue">{{ packageInfo.version }}</a-tag>
        </a-descriptions-item>
        <a-descriptions-item label="作者">{{ packageInfo.author }}</a-descriptions-item>
        <a-descriptions-item label="许可证">{{ packageInfo.license }}</a-descriptions-item>
      </a-descriptions>
      <div class="links">
        <a-space>
          <a-button type="primary" @click="openUrl(packageInfo.github)">
            <template #icon><StarOutlined /></template>
            GitHub 仓库
          </a-button>
          <a-button @click="checkUpdate">检查更新</a-button>
        </a-space>
      </div>
    </a-card>
  </div>
</template>

<style scoped lang="scss">
.about-page { padding: 24px; display: flex; justify-content: center; align-items: center; height: calc(100vh - 60px); }
.about-card { max-width: 500px; width: 100%; }
.logo-section { text-align: center; padding: 24px 0; }
.app-name { font-size: 28px; font-weight: 700; margin: 0 0 8px; }
.description { color: #8c8c8c; font-size: 14px; }
.links { margin-top: 24px; text-align: center; }
</style>
