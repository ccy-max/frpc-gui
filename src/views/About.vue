<script setup lang="ts">
import { StarOutlined } from '@ant-design/icons-vue';

const packageInfo = {
  version: '0.1.0',
  author: 'Zero',
  license: 'MIT',
  github: 'https://github.com/ccy-max/frpc-gui',
};

function openUrl(url: string) {
  window.open(url, '_blank');
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
        <a-button type="primary" :icon="h(StarOutlined)" @click="openUrl(packageInfo.github)">GitHub 仓库</a-button>
      </div>
    </a-card>
  </div>
</template>

<script lang="ts">
import { h } from 'vue';
import { StarOutlined } from '@ant-design/icons-vue';
export default { methods: { h } };
</script>

<style scoped lang="scss">
.about-page {
  padding: 24px;
  display: flex;
  justify-content: center;
  align-items: center;
  height: calc(100vh - 60px);
}
.about-card { max-width: 500px; width: 100%; }
.logo-section { text-align: center; padding: 24px 0; }
.app-name { font-size: 28px; font-weight: 700; margin: 0 0 8px; }
.description { color: #8c8c8c; font-size: 14px; }
.links { margin-top: 24px; text-align: center; }
</style>
