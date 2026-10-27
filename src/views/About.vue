<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { useAppStore } from '@/stores/app';
import { StarOutlined } from '@ant-design/icons-vue';
import { message } from 'ant-design-vue';
import packageJson from '../../package.json';

const { t } = useI18n();
const appStore = useAppStore();

const packageInfo = {
  version: packageJson.version,
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
        <p class="description">{{ t('about.description') }}</p>
      </div>
      <a-descriptions :column="1" bordered>
        <a-descriptions-item :label="t('about.version')">
          <a-tag color="blue">{{ packageInfo.version }}</a-tag>
        </a-descriptions-item>
        <a-descriptions-item :label="t('about.author')">{{ packageInfo.author }}</a-descriptions-item>
        <a-descriptions-item :label="t('about.license')">{{ packageInfo.license }}</a-descriptions-item>
      </a-descriptions>
      <div class="links">
        <a-space>
          <a-button type="primary" @click="openUrl(packageInfo.github)">
            <template #icon><StarOutlined /></template>
            {{ t('about.github') }}
          </a-button>
          <a-button @click="checkUpdate">检查更新</a-button>
        </a-space>
      </div>
    </a-card>
  </div>
</template>

<style scoped lang="scss">
.about-page {
  padding: 24px;
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: calc(100vh - 60px);
}

.about-card {
  max-width: 550px;
  width: 100%;
  border-radius: 16px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08);
  transition: all 0.3s ease;

  &:hover {
    transform: translateY(-4px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
  }
}

.logo-section {
  text-align: center;
  padding: 32px 0 24px;

  .app-name {
    font-size: 32px;
    font-weight: 800;
    margin: 0 0 8px;
    background: linear-gradient(135deg, #2563eb 0%, #7c3aed 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .description {
    color: #64748b;
    font-size: 15px;
    margin: 0;
    font-weight: 400;
  }
}

:deep(.ant-descriptions) {
  .ant-descriptions-item-label {
    font-weight: 600;
    color: #475569;
    background: #f8fafc;
  }

  .ant-descriptions-item-content {
    color: #1e293b;
  }
}

.links {
  margin-top: 32px;
  text-align: center;
  padding: 16px 0;

  .ant-btn {
    border-radius: 8px;
    font-weight: 500;
    transition: all 0.2s ease;

    &:hover {
      transform: translateY(-1px);
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
    }

    &:active {
      transform: scale(0.98);
    }
  }
}
</style>
