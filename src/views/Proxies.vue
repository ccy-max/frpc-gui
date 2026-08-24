<script setup lang="ts">
import { ref, computed } from 'vue';
import { useAppStore } from '@/stores/app';
import { useI18n } from 'vue-i18n';
import type { ProxyConfig } from '@/types';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Plus } from '@element-plus/icons-vue';

const appStore = useAppStore();
const { t } = useI18n();

const dialogVisible = ref(false);
const form = ref<ProxyConfig>({
  name: '',
  type: 'tcp',
  localIP: '127.0.0.1',
  localPort: 8080,
  remotePort: 8080,
  customDomains: [],
  subdomain: '',
  locations: [],
  httpUser: '',
  httpPassword: '',
  useEncryption: false,
  useCompression: false,
  secretKey: '',
  role: 'bind',
  serverName: '',
  enabled: true,
  createdAt: Date.now(),
  updatedAt: Date.now(),
});

const proxyTypes = [
  { value: 'tcp', label: 'TCP' },
  { value: 'udp', label: 'UDP' },
  { value: 'http', label: 'HTTP' },
  { value: 'https', label: 'HTTPS' },
  { value: 'stcp', label: 'STCP' },
  { value: 'xtcp', label: 'XTCP' },
];

function openAddDialog() {
  form.value = {
    name: '',
    type: 'tcp',
    localIP: '127.0.0.1',
    localPort: 8080,
    remotePort: 8080,
    customDomains: [],
    subdomain: '',
    locations: [],
    httpUser: '',
    httpPassword: '',
    useEncryption: false,
    useCompression: false,
    secretKey: '',
    role: 'bind',
    serverName: '',
    enabled: true,
    createdAt: Date.now(),
    updatedAt: Date.now(),
  };
  dialogVisible.value = true;
}

function handleSave() {
  if (!form.value.name) {
    ElMessage.warning('请输入代理名称');
    return;
  }
  appStore.addProxy(form.value);
  ElMessage.success(t('message.saveSuccess'));
  dialogVisible.value = false;
}

function handleDelete(proxy: ProxyConfig) {
  ElMessageBox.confirm(t('message.deleteConfirm'), t('common.warning'), {
    type: 'warning',
  }).then(() => {
    appStore.deleteProxy(proxy.name);
    ElMessage.success(t('message.deleteSuccess'));
  }).catch(() => {});
}

function toggleProxy(proxy: ProxyConfig) {
  appStore.updateProxy(proxy.name, { enabled: !proxy.enabled });
  ElMessage.success(proxy.enabled ? '已停止' : '已启动');
}

const proxies = computed(() => appStore.proxies);
</script>

<template>
  <div class="proxies-page">
    <div class="page-header">
      <h2 class="page-title">{{ t('proxy.title') }}</h2>
      <el-button type="primary" :icon="Plus" @click="openAddDialog">
        {{ t('proxy.addProxy') }}
      </el-button>
    </div>

    <el-table :data="proxies" style="width: 100%">
      <el-table-column prop="name" :label="t('proxy.name')" min-width="120" />
      <el-table-column prop="type" :label="t('proxy.type')" width="100">
        <template #default="{ row }">
          <el-tag size="small" type="info">{{ row.type.toUpperCase() }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="本地地址" min-width="150">
        <template #default="{ row }">
          {{ row.localIP }}:{{ row.localPort }}
        </template>
      </el-table-column>
      <el-table-column prop="remotePort" :label="t('proxy.remotePort')" width="100" />
      <el-table-column prop="enabled" :label="t('proxy.status')" width="100">
        <template #default="{ row }">
          <el-tag :type="row.enabled ? 'success' : 'info'" size="small">
            {{ row.enabled ? t('proxy.running') : t('proxy.stopped') }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column :label="t('common.operation')" width="220" fixed="right">
        <template #default="{ row }">
          <el-button size="small" :type="row.enabled ? 'warning' : 'success'" @click="toggleProxy(row as ProxyConfig)">
            {{ row.enabled ? t('proxy.stop') : t('proxy.start') }}
          </el-button>
          <el-button size="small" type="danger" @click="handleDelete(row as ProxyConfig)">
            {{ t('common.delete') }}
          </el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-empty v-if="proxies.length === 0" :description="t('proxy.title')" />

    <!-- 添加代理对话框 -->
    <el-dialog v-model="dialogVisible" :title="t('proxy.addProxy')" width="600px">
      <el-form :model="form" label-width="120px">
        <el-form-item :label="t('proxy.name')">
          <el-input v-model="form.name" />
        </el-form-item>
        
        <el-form-item :label="t('proxy.type')">
          <el-select v-model="form.type" style="width: 100%">
            <el-option v-for="item in proxyTypes" :key="item.value" :label="item.label" :value="item.value" />
          </el-select>
        </el-form-item>

        <el-row :gutter="16">
          <el-col :span="12">
            <el-form-item :label="t('proxy.localIP')">
              <el-input v-model="form.localIP" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item :label="t('proxy.localPort')">
              <el-input-number v-model="form.localPort" :min="1" :max="65535" style="width: 100%" />
            </el-form-item>
          </el-col>
        </el-row>

        <el-form-item :label="t('proxy.remotePort')">
          <el-input-number v-model="form.remotePort" :min="1" :max="65535" style="width: 100%" />
        </el-form-item>

        <el-form-item :label="t('proxy.useEncryption')">
          <el-switch v-model="form.useEncryption" />
        </el-form-item>

        <el-form-item :label="t('proxy.useCompression')">
          <el-switch v-model="form.useCompression" />
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="dialogVisible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" @click="handleSave">{{ t('common.save') }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped lang="scss">
.proxies-page {
  padding: 24px;
  height: calc(100vh - 60px);
  overflow-y: auto;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}
</style>
