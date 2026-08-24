<script setup lang="ts">
import { ref, computed } from 'vue';
import { useAppStore } from '@/stores/app';
import { FolderOpenOutlined } from '@ant-design/icons-vue';
import { message, Modal } from 'ant-design-vue';

const appStore = useAppStore();

const activeTab = ref('basic');

// 传输协议选项
const transportProtocols = [
  { value: 'tcp', label: 'TCP' },
  { value: 'kcp', label: 'KCP' },
  { value: 'quic', label: 'QUIC' },
  { value: 'websocket', label: 'WebSocket' },
];

// 日志级别选项
const logLevels = [
  { value: 'trace', label: 'Trace' },
  { value: 'debug', label: 'Debug' },
  { value: 'info', label: 'Info' },
  { value: 'warn', label: 'Warn' },
  { value: 'error', label: 'Error' },
];

// 配置访问器 - 避免可选链赋值问题
const transportProtocol = computed({
  get: () => appStore.frpConfig?.transport.protocol || 'tcp',
  set: (v) => { if (appStore.frpConfig) appStore.frpConfig.transport.protocol = v; }
});

const tlsEnable = computed({
  get: () => appStore.frpConfig?.tls.enable ?? false,
  set: (v) => { if (appStore.frpConfig) appStore.frpConfig.tls.enable = v; }
});

const tlsCertFile = computed({
  get: () => appStore.frpConfig?.tls.cert_file || '',
  set: (v) => { if (appStore.frpConfig) appStore.frpConfig.tls.cert_file = v; }
});

const tlsKeyFile = computed({
  get: () => appStore.frpConfig?.tls.key_file || '',
  set: (v) => { if (appStore.frpConfig) appStore.frpConfig.tls.key_file = v; }
});

const tlsCaFile = computed({
  get: () => appStore.frpConfig?.tls.trusted_ca_file || '',
  set: (v) => { if (appStore.frpConfig) appStore.frpConfig.tls.trusted_ca_file = v; }
});

const heartbeatInterval = computed({
  get: () => appStore.frpConfig?.heartbeat_interval || 30,
  set: (v) => { if (appStore.frpConfig) appStore.frpConfig.heartbeat_interval = v; }
});

const heartbeatTimeout = computed({
  get: () => appStore.frpConfig?.heartbeat_timeout || 90,
  set: (v) => { if (appStore.frpConfig) appStore.frpConfig.heartbeat_timeout = v; }
});

const logLevel = computed({
  get: () => appStore.frpConfig?.log.level || 'info',
  set: (v) => { if (appStore.frpConfig) appStore.frpConfig.log.level = v; }
});

const logMaxDays = computed({
  get: () => appStore.frpConfig?.log.max_days || 3,
  set: (v) => { if (appStore.frpConfig) appStore.frpConfig.log.max_days = v; }
});

const adminAddr = computed({
  get: () => appStore.frpConfig?.admin.addr || '127.0.0.1',
  set: (v) => { if (appStore.frpConfig) appStore.frpConfig.admin.addr = v; }
});

const adminPort = computed({
  get: () => appStore.frpConfig?.admin.port || 7400,
  set: (v) => { if (appStore.frpConfig) appStore.frpConfig.admin.port = v; }
});

const adminUser = computed({
  get: () => appStore.frpConfig?.admin.user || '',
  set: (v) => { if (appStore.frpConfig) appStore.frpConfig.admin.user = v; }
});

const adminPassword = computed({
  get: () => appStore.frpConfig?.admin.password || '',
  set: (v) => { if (appStore.frpConfig) appStore.frpConfig.admin.password = v; }
});

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

async function pickFile(field: string, ext: string[]) {
  const { open } = await import('@tauri-apps/plugin-dialog');
  const selected = await open({
    filters: [{ name: '文件', extensions: ext }],
  });
  if (selected) {
    if (field === 'certFile') appStore.frpConfig!.tls.cert_file = selected as string;
    if (field === 'keyFile') appStore.frpConfig!.tls.key_file = selected as string;
    if (field === 'caFile') appStore.frpConfig!.tls.trusted_ca_file = selected as string;
  }
}
</script>

<template>
  <div class="settings-page">
    <h2 class="page-title">设置</h2>
    
    <a-tabs v-model:activeKey="activeTab">
      <!-- 通用设置 -->
      <a-tab-pane key="general" tab="通用">
        <a-form layout="vertical" style="max-width: 500px">
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
          <a-form-item label="启动选项">
            <a-space direction="vertical">
              <a-checkbox v-model:checked="appStore.autoStart" @change="handleConfigChange">开机自启</a-checkbox>
              <a-checkbox v-model:checked="appStore.minimizeToTray" @change="handleConfigChange">最小化到托盘</a-checkbox>
              <a-checkbox v-model:checked="appStore.closeToTray" @change="handleConfigChange">关闭到托盘</a-checkbox>
            </a-space>
          </a-form-item>
        </a-form>
      </a-tab-pane>

      <!-- 路径设置 -->
      <a-tab-pane key="paths" tab="路径">
        <a-form layout="vertical" style="max-width: 500px">
          <a-form-item label="FRP 可执行文件">
            <a-input
              v-model:value="appStore.frpcPath"
              placeholder="点击右侧按钮选择 frpc"
              readonly
            >
              <template #suffix>
                <a-button type="text" @click="appStore.pickFrpcPath()">
                  <template #icon><FolderOpenOutlined /></template>
                </a-button>
              </template>
            </a-input>
          </a-form-item>
          <a-form-item label="配置文件">
            <a-input
              v-model:value="appStore.configPath"
              placeholder="点击右侧按钮选择保存位置"
              readonly
            >
              <template #suffix>
                <a-button type="text" @click="appStore.pickConfigPath()">
                  <template #icon><FolderOpenOutlined /></template>
                </a-button>
              </template>
            </a-input>
          </a-form-item>
          <a-form-item label="日志目录">
            <a-input
              v-model:value="appStore.logPath"
              placeholder="点击右侧按钮选择目录"
              readonly
            >
              <template #suffix>
                <a-button type="text" @click="appStore.pickLogPath()">
                  <template #icon><FolderOpenOutlined /></template>
                </a-button>
              </template>
            </a-input>
          </a-form-item>
        </a-form>
      </a-tab-pane>

      <!-- 高级配置 -->
      <a-tab-pane key="advanced" tab="高级">
        <a-form layout="vertical" style="max-width: 500px">
          <a-form-item label="传输协议">
            <a-select v-model:value="transportProtocol" style="width: 100%">
              <a-select-option v-for="p in transportProtocols" :key="p.value" :value="p.value">
                {{ p.label }}
              </a-select-option>
            </a-select>
          </a-form-item>
          
          <a-divider orientation="left">TLS 配置</a-divider>
          
          <a-form-item label="启用 TLS">
            <a-switch v-model:checked="tlsEnable" />
          </a-form-item>
          
          <a-form-item label="证书文件" v-if="tlsEnable">
            <a-input
              v-model:value="tlsCertFile"
              placeholder="选择证书文件"
              readonly
            >
              <template #suffix>
                <a-button type="text" @click="pickFile('certFile', ['crt', 'pem'])">
                  <template #icon><FolderOpenOutlined /></template>
                </a-button>
              </template>
            </a-input>
          </a-form-item>
          
          <a-form-item label="密钥文件" v-if="tlsEnable">
            <a-input
              v-model:value="tlsKeyFile"
              placeholder="选择密钥文件"
              readonly
            >
              <template #suffix>
                <a-button type="text" @click="pickFile('keyFile', ['key', 'pem'])">
                  <template #icon><FolderOpenOutlined /></template>
                </a-button>
              </template>
            </a-input>
          </a-form-item>
          
          <a-form-item label="CA 文件" v-if="tlsEnable">
            <a-input
              v-model:value="tlsCaFile"
              placeholder="选择 CA 文件"
              readonly
            >
              <template #suffix>
                <a-button type="text" @click="pickFile('caFile', ['crt', 'pem'])">
                  <template #icon><FolderOpenOutlined /></template>
                </a-button>
              </template>
            </a-input>
          </a-form-item>
          
          <a-divider orientation="left">心跳配置</a-divider>
          
          <a-row :gutter="16">
            <a-col :span="12">
              <a-form-item label="心跳间隔 (秒)">
                <a-input-number v-model:value="heartbeatInterval" :min="0" style="width: 100%" />
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="心跳超时 (秒)">
                <a-input-number v-model:value="heartbeatTimeout" :min="0" style="width: 100%" />
              </a-form-item>
            </a-col>
          </a-row>
          
          <a-divider orientation="left">日志配置</a-divider>
          
          <a-form-item label="日志级别">
            <a-select v-model:value="logLevel" style="width: 100%">
              <a-select-option v-for="l in logLevels" :key="l.value" :value="l.value">
                {{ l.label }}
              </a-select-option>
            </a-select>
          </a-form-item>
          
          <a-form-item label="日志保留天数">
            <a-input-number v-model:value="logMaxDays" :min="0" style="width: 100%" />
          </a-form-item>
          
          <a-divider orientation="left">管理控制台</a-divider>
          
          <a-row :gutter="16">
            <a-col :span="12">
              <a-form-item label="管理地址">
                <a-input v-model:value="adminAddr" placeholder="127.0.0.1" />
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="管理端口">
                <a-input-number v-model:value="adminPort" :min="0" :max="65535" style="width: 100%" />
              </a-form-item>
            </a-col>
          </a-row>
          
          <a-row :gutter="16">
            <a-col :span="12">
              <a-form-item label="管理用户">
                <a-input v-model:value="adminUser" />
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="管理密码">
                <a-input-password v-model:value="adminPassword" />
              </a-form-item>
            </a-col>
          </a-row>
        </a-form>
      </a-tab-pane>

      <!-- 数据管理 -->
      <a-tab-pane key="data" tab="数据">
        <a-space direction="vertical" style="width: 100%">
          <a-button block @click="appStore.openAppData()">打开数据目录</a-button>
          <a-button block danger @click="handleReset">一键清空所有配置</a-button>
        </a-space>
      </a-tab-pane>
    </a-tabs>
  </div>
</template>

<style scoped lang="scss">
.settings-page { padding: 24px; height: calc(100vh - 60px); overflow-y: auto; }
.page-title { font-size: 24px; font-weight: 600; margin-bottom: 24px; }
</style>
