<script setup lang="ts">
import { useAppStore } from '@/stores/app';
import { FolderOpenOutlined } from '@ant-design/icons-vue';
import { message, Modal } from 'ant-design-vue';

const appStore = useAppStore();

async function handleConfigChange() {
  await appStore.saveSettings();
  message.success('设置已保存');
}

function handleReset() {
  Modal.confirm({
    title: '确认清空',
    content: '将清空所有配置、下载的 FRP 版本和日志。此操作不可恢复！',
    okText: '确认清空',
    okType: 'danger',
    cancelText: '取消',
    onOk: async () => {
      const result = await appStore.resetAllConfig();
      if (result.success) {
        message.success('已清空所有配置');
      } else {
        message.error(`清空失败：${result.error}`);
      }
    },
  });
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
                <template #suffix>
                  <a-button type="text" size="small" @click="appStore.pickFrpcPath()">
                    <template #icon><FolderOpenOutlined /></template>
                  </a-button>
                </template>
              </a-input>
            </a-form-item>
            <a-form-item label="配置文件路径">
              <a-input
                v-model:value="appStore.configPath"
                placeholder="点击右侧按钮选择配置文件保存位置"
                readonly
              >
                <template #suffix>
                  <a-button type="text" size="small" @click="appStore.pickConfigPath()">
                    <template #icon><FolderOpenOutlined /></template>
                  </a-button>
                </template>
              </a-input>
            </a-form-item>
            <a-form-item label="日志目录">
              <a-input
                v-model:value="appStore.logPath"
                placeholder="点击右侧按钮选择日志目录"
                readonly
              >
                <template #suffix>
                  <a-button type="text" size="small" @click="appStore.pickLogPath()">
                    <template #icon><FolderOpenOutlined /></template>
                  </a-button>
                </template>
              </a-input>
            </a-form-item>
          </a-form>
        </a-card>

        <a-card title="数据管理" style="margin-top: 16px">
          <a-space direction="vertical" style="width: 100%">
            <a-button block @click="appStore.openAppData()">打开数据目录</a-button>
            <a-button block danger @click="handleReset">一键清空所有配置</a-button>
          </a-space>
        </a-card>
      </a-col>
    </a-row>
  </div>
</template>

<style scoped lang="scss">
.settings-page { padding: 24px; height: calc(100vh - 60px); overflow-y: auto; }
.page-title { font-size: 24px; font-weight: 600; margin-bottom: 24px; }
</style>
