<script setup lang="ts">
import { useAppStore } from '@/stores/app';
import { FolderOutlined } from '@ant-design/icons-vue';

const appStore = useAppStore();

function handleConfigChange() {
  if (appStore.frpConfig) {
    appStore.saveConfig(appStore.frpConfig);
  }
}
</script>

<template>
  <div class="settings-page">
    <h2 class="page-title">设置</h2>
    <a-row :gutter="16">
      <a-col :span="12">
        <a-card title="通用设置">
          <a-form layout="vertical" style="max-width: 400px">
            <a-form-item label="语言">
              <a-select v-model:value="appStore.language" @change="handleConfigChange" style="width: 100%">
                <a-select-option value="zh-CN">简体中文</a-select-option>
                <a-select-option value="en-US">English</a-select-option>
              </a-select>
            </a-form-item>
            <a-form-item label="主题">
              <a-select v-model:value="appStore.theme" @change="handleConfigChange" style="width: 100%">
                <a-select-option value="light">浅色</a-select-option>
                <a-select-option value="dark">深色</a-select-option>
                <a-select-option value="auto">跟随系统</a-select-option>
              </a-select>
            </a-form-item>
            <a-form-item label="开机自启">
              <a-switch v-model:checked="appStore.processStatus.running" />
            </a-form-item>
          </a-form>
        </a-card>
      </a-col>
      <a-col :span="12">
        <a-card title="路径设置">
          <a-form layout="vertical" style="max-width: 400px">
            <a-form-item label="FRP 路径">
              <a-input readonly placeholder="请选择 frpc 可执行文件路径">
                <template #addonAfter>
                  <a-button :icon="h(FolderOutlined)" size="small">浏览</a-button>
                </template>
              </a-input>
            </a-form-item>
            <a-form-item label="配置路径">
              <a-input readonly placeholder="frpc.toml 保存路径">
                <template #addonAfter>
                  <a-button :icon="h(FolderOutlined)" size="small">浏览</a-button>
                </template>
              </a-input>
            </a-form-item>
          </a-form>
        </a-card>
      </a-col>
    </a-row>
  </div>
</template>

<script lang="ts">
import { h } from 'vue';
import { FolderOutlined } from '@ant-design/icons-vue';
export default { methods: { h } };
</script>

<style scoped lang="scss">
.settings-page { padding: 24px; height: calc(100vh - 60px); overflow-y: auto; }
.page-title { font-size: 24px; font-weight: 600; margin-bottom: 24px; }
</style>
