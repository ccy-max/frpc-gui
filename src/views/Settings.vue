<script setup lang="ts">
import { ref, computed } from 'vue';
import { useAppStore } from '@/stores/app';
import { FolderOpenOutlined, ExportOutlined, ImportOutlined, CopyOutlined, SnippetsOutlined } from '@ant-design/icons-vue';
import { message, Modal } from 'ant-design-vue';
import { writeText, readText } from '@tauri-apps/plugin-clipboard-manager';

const appStore = useAppStore();
const activeTab = ref('general');

const transportProtocols = [
  { value: 'tcp', label: 'TCP' },
  { value: 'kcp', label: 'KCP' },
  { value: 'quic', label: 'QUIC' },
  { value: 'websocket', label: 'WebSocket' },
];

const logLevels = [
  { value: 'trace', label: 'Trace' },
  { value: 'debug', label: 'Debug' },
  { value: 'info', label: 'Info' },
  { value: 'warn', label: 'Warn' },
  { value: 'error', label: 'Error' },
];

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

const base64DialogVisible = ref(false);
const base64Mode = ref<'export' | 'import'>('export');
const base64Text = ref('');

function safeEncodeBase64(str: string): string {
  try { return btoa(unescape(encodeURIComponent(str))); }
  catch (e) { console.error('Base64 encode failed:', e); return ''; }
}

function safeDecodeBase64(base64: string): string {
  try { return decodeURIComponent(escape(atob(base64))); }
  catch (e) { console.error('Base64 decode failed:', e); throw new Error('Invalid Base64'); }
}

async function handleExportBase64() {
  base64Mode.value = 'export';
  const config = appStore.frpConfig;
  if (!config) { message.warning('没有可导出的配置'); return; }
  base64Text.value = safeEncodeBase64(JSON.stringify(config, null, 2));
  base64DialogVisible.value = true;
}

async function handleCopyBase64() {
  try { await writeText(base64Text.value); message.success('已复制到剪贴板'); }
  catch (e) { message.error('复制失败'); }
}

async function handleImportBase64() {
  base64Mode.value = 'import';
  base64Text.value = '';
  base64DialogVisible.value = true;
}

async function handlePasteBase64() {
  try {
    const text = await readText();
    if (text) base64Text.value = text.replace('frp://', '').trim();
  } catch (e) { message.error('读取剪贴板失败'); }
}

async function handleParseBase64() {
  if (!base64Text.value.trim()) { message.warning('请输入 Base64 配置'); return; }
  try {
    const json = safeDecodeBase64(base64Text.value.trim());
    const config = JSON.parse(json);
    if (!config.server_addr || !config.proxies) { message.error('无效的配置格式'); return; }
    appStore.frpConfig = config;
    await appStore.saveConfig(config);
    message.success('配置导入成功');
    base64DialogVisible.value = false;
  } catch (e: any) { message.error(`解析失败：${e.message}`); }
}

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
      if (result.success) message.success('已清空所有配置');
      else message.error(`清空失败：${result.error}`);
    },
  });
}

async function pickFile(field: string, ext: string[]) {
  const { open } = await import('@tauri-apps/plugin-dialog');
  const selected = await open({ filters: [{ name: '文件', extensions: ext }] });
  if (selected) {
    if (field === 'certFile') appStore.frpConfig!.tls.cert_file = selected as string;
    if (field === 'keyFile') appStore.frpConfig!.tls.key_file = selected as string;
    if (field === 'caFile') appStore.frpConfig!.tls.trusted_ca_file = selected as string;
  }
}
</script>

<template>
  <div>
    <a-page-header title="设置" />
    
    <a-tabs v-model:activeKey="activeTab">
      <a-tab-pane key="general" tab="通用">
        <a-form layout="vertical" :wrapper-col="{ span: 12 }">
          <a-form-item label="语言">
            <a-select v-model:value="appStore.language" @change="handleConfigChange" style="width: 200px">
              <a-select-option value="zh-CN">简体中文</a-select-option>
              <a-select-option value="en-US">English</a-select-option>
            </a-select>
          </a-form-item>
          <a-form-item label="主题">
            <a-select v-model:value="appStore.theme" @change="handleConfigChange" style="width: 200px">
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

      <a-tab-pane key="paths" tab="路径">
        <a-form layout="vertical" :wrapper-col="{ span: 20 }">
          <a-form-item label="FRP 可执行文件">
            <a-input v-model:value="appStore.frpcPath" placeholder="点击右侧按钮选择 frpc" readonly>
              <template #suffix>
                <a-button type="link" @click="appStore.pickFrpcPath()"><FolderOpenOutlined /></a-button>
              </template>
            </a-input>
          </a-form-item>
          <a-form-item label="配置文件">
            <a-input v-model:value="appStore.configPath" placeholder="点击右侧按钮选择保存位置" readonly>
              <template #suffix>
                <a-button type="link" @click="appStore.pickConfigPath()"><FolderOpenOutlined /></a-button>
              </template>
            </a-input>
          </a-form-item>
          <a-form-item label="日志目录">
            <a-input v-model:value="appStore.logPath" placeholder="点击右侧按钮选择目录" readonly>
              <template #suffix>
                <a-button type="link" @click="appStore.pickLogPath()"><FolderOpenOutlined /></a-button>
              </template>
            </a-input>
          </a-form-item>
        </a-form>
      </a-tab-pane>

      <a-tab-pane key="advanced" tab="高级">
        <a-form layout="vertical" :wrapper-col="{ span: 12 }">
          <a-form-item label="传输协议">
            <a-select v-model:value="transportProtocol" style="width: 200px">
              <a-select-option v-for="p in transportProtocols" :key="p.value" :value="p.value">{{ p.label }}</a-select-option>
            </a-select>
          </a-form-item>
          
          <a-divider orientation="left">TLS 配置</a-divider>
          
          <a-form-item label="启用 TLS">
            <a-switch v-model:checked="tlsEnable" />
          </a-form-item>
          
          <template v-if="tlsEnable">
            <a-form-item label="证书文件">
              <a-input v-model:value="tlsCertFile" placeholder="选择证书文件 (.crt/.pem)" readonly>
                <template #suffix>
                  <a-button type="link" @click="pickFile('certFile', ['crt', 'pem'])"><FolderOpenOutlined /></a-button>
                </template>
              </a-input>
            </a-form-item>
            <a-form-item label="密钥文件">
              <a-input v-model:value="tlsKeyFile" placeholder="选择密钥文件 (.key/.pem)" readonly>
                <template #suffix>
                  <a-button type="link" @click="pickFile('keyFile', ['key', 'pem'])"><FolderOpenOutlined /></a-button>
                </template>
              </a-input>
            </a-form-item>
            <a-form-item label="CA 文件">
              <a-input v-model:value="tlsCaFile" placeholder="选择 CA 文件 (.crt/.pem)" readonly>
                <template #suffix>
                  <a-button type="link" @click="pickFile('caFile', ['crt', 'pem'])"><FolderOpenOutlined /></a-button>
                </template>
              </a-input>
            </a-form-item>
          </template>
          
          <a-divider orientation="left">心跳配置</a-divider>
          
          <a-row :gutter="24">
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
          
          <a-row :gutter="24">
            <a-col :span="12">
              <a-form-item label="日志级别">
                <a-select v-model:value="logLevel" style="width: 200px">
                  <a-select-option v-for="l in logLevels" :key="l.value" :value="l.value">{{ l.label }}</a-select-option>
                </a-select>
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="日志保留天数">
                <a-input-number v-model:value="logMaxDays" :min="0" style="width: 200px" />
              </a-form-item>
            </a-col>
          </a-row>
          
          <a-divider orientation="left">管理控制台</a-divider>
          
          <a-row :gutter="24">
            <a-col :span="12">
              <a-form-item label="管理地址">
                <a-input v-model:value="adminAddr" placeholder="127.0.0.1" style="width: 200px" />
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="管理端口">
                <a-input-number v-model:value="adminPort" :min="0" :max="65535" style="width: 200px" />
              </a-form-item>
            </a-col>
          </a-row>
          <a-row :gutter="24">
            <a-col :span="12">
              <a-form-item label="管理用户">
                <a-input v-model:value="adminUser" style="width: 200px" />
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="管理密码">
                <a-input-password v-model:value="adminPassword" style="width: 200px" />
              </a-form-item>
            </a-col>
          </a-row>
        </a-form>
      </a-tab-pane>

      <a-tab-pane key="data" tab="数据">
        <a-space direction="vertical" style="width: 100%; max-width: 400px">
          <a-button block @click="appStore.openAppData()">打开数据目录</a-button>
          <a-divider>配置分享</a-divider>
          <a-button block @click="handleExportBase64">Base64 导出配置</a-button>
          <a-button block @click="handleImportBase64">Base64 导入配置</a-button>
          <a-divider danger>危险操作</a-divider>
          <a-button block danger @click="handleReset">一键清空所有配置</a-button>
        </a-space>
      </a-tab-pane>
    </a-tabs>

    <a-modal
      v-model:open="base64DialogVisible"
      :title="base64Mode === 'export' ? '导出配置' : '导入配置'"
      width="600px"
      @ok="base64Mode === 'import' ? handleParseBase64() : base64DialogVisible = false"
    >
      <div v-if="base64Mode === 'export'">
        <p>复制以下 Base64 字符串分享给他人：</p>
        <a-textarea v-model:value="base64Text" :rows="10" readonly style="font-family: monospace; font-size: 12px" />
        <div style="margin-top: 16px; text-align: right">
          <a-button type="primary" @click="handleCopyBase64"><CopyOutlined /> 复制</a-button>
        </div>
      </div>
      <div v-else>
        <p>粘贴 Base64 配置字符串：</p>
        <a-textarea v-model:value="base64Text" :rows="10" placeholder="frp://..." style="font-family: monospace; font-size: 12px" />
        <div style="margin-top: 16px; text-align: right">
          <a-button @click="handlePasteBase64"><SnippetsOutlined /> 从剪贴板粘贴</a-button>
        </div>
      </div>
    </a-modal>
  </div>
</template>
