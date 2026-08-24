<script setup lang="ts">
import { useAppStore } from '@/stores/app';
import { FolderOpenOutlined } from '@ant-design/icons-vue';
import { message } from 'ant-design-vue';
import { h } from 'vue';

const appStore = useAppStore();

async function handleConfigChange() {
  await appStore.saveSettings();
  message.success('设置已保存');
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
              <a-switch v-model:checked="appStore.autoStart" @change="handleConfigChange" />
            </a-form-item>
            <a-form-item label="最小化到托盘">
              <a-switch v-model:checked="appStore.minimizeToTray" @change="handleConfigChange" />
            </a-form-item>
            <a-form-item label="关闭到托盘">
              <a-switch v-model:checked="appStore.closeToTray" @change="handleConfigChange" />
            </a-form-item>
          </a-form>
        </a-card>
      </a-col>

      <a-col :span="12">
        <a-card title="路径设置">
          <a-form layout="vertical" style="max-width: 400px">
            <a-form-item label="FRP 可执行文件路径">
              <a-input
                v-model:value="appStore.frpcPath"
                placeholder="点击右侧按钮选择 frpc 可执行文件"
                readonly
              >
                <template #addonAfter>
                  <a-button :icon="h(FolderOpenOutlined)" size="small" @click="appStore.pickFrpcPath()" type="text" />
                </template>
              </a-input>
            </a-form-item>
            <a-form-item label="配置文件路径">
              <a-input
                v-model:value="appStore.configPath"
                placeholder="点击右侧按钮选择配置文件保存位置"
                readonly
              >
                <template #addonAfter>
                  <a-button :icon="h(FolderOpenOutlined)" size="small" @click="appStore.pickConfigPath()" type="text" />
                </template>
              </a-input>
            </a-form-item>
            <a-form-item label="日志目录">
              <a-input
                v-model:value="appStore.logPath"
                placeholder="点击右侧按钮选择日志目录"
                readonly
              >
                <template #addonAfter>
                  <a-button :icon="h(FolderOpenOutlined)" size="small" @click="appStore.pickLogPath()" type="text" />
                </template>
              </a-input>
            </a-form-item>
          </a-form>
        </a-card>
      </a-col>
    </a-row>
  </div>
</template>

<style scoped lang="scss">
.settings-page { padding: 24px; height: calc(100vh - 60px); overflow-y: auto; }
.page-title { font-size: 24px; font-weight: 600; margin-bottom: 24px; }
</style>
