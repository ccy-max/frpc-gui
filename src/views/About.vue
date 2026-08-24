<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { Star } from '@element-plus/icons-vue';

function openUrl(url: string) {
  window.open(url, '_blank');
}

const { t } = useI18n();

const packageInfo = {
  version: '0.1.0',
  author: 'Zero',
  license: 'MIT',
  github: 'https://github.com/your-repo/frpc-gui',
};
</script>

<template>
  <div class="about-page">
    <el-card class="about-card">
      <div class="logo-section">
        <el-icon :size="64" color="var(--el-color-primary)"><Monitor /></el-icon>
        <h1 class="app-name">{{ t('common.appTitle') }}</h1>
        <p class="description">{{ t('about.description') }}</p>
      </div>

      <el-descriptions :column="1" border>
        <el-descriptions-item :label="t('about.version')">
          <el-tag>{{ packageInfo.version }}</el-tag>
        </el-descriptions-item>
        <el-descriptions-item :label="t('about.author')">
          {{ packageInfo.author }}
        </el-descriptions-item>
        <el-descriptions-item :label="t('about.license')">
          {{ packageInfo.license }}
        </el-descriptions-item>
      </el-descriptions>

      <div class="links">
        <el-button type="primary" :icon="Star" @click="openUrl(packageInfo.github)">
          {{ t('about.github') }}
        </el-button>
      </div>
    </el-card>
  </div>
</template>

<style scoped lang="scss">
.about-page {
  padding: 24px;
  display: flex;
  justify-content: center;
  align-items: center;
  height: calc(100vh - 60px);
}

.about-card {
  max-width: 500px;
  width: 100%;
}

.logo-section {
  text-align: center;
  padding: 24px 0;
}

.app-name {
  font-size: 28px;
  font-weight: 700;
  margin: 16px 0 8px;
}

.description {
  color: var(--el-text-color-secondary);
  font-size: 14px;
}

.links {
  margin-top: 24px;
  text-align: center;
}
</style>
