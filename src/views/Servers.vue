<script setup lang="ts">
import { ref, computed } from 'vue';
import { useAppStore } from '@/stores/app';
import { useI18n } from 'vue-i18n';
import type { ServerConfig } from '@/types';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Plus } from '@element-plus/icons-vue';

const appStore = useAppStore();
const { t } = useI18n();

const dialogVisible = ref(false);
const editingServer = ref<ServerConfig | null>(null);

const formRef = ref();
const form = ref<ServerConfig>({
  id: '',
  name: '',
  serverAddr: '',
  serverPort: 7000,
  authMethod: 'token',
  token: '',
  user: '',
  metaToken: '',
  tlsEnable: false,
  logLevel: 'info',
  logMaxDays: 3,
  adminAddr: '127.0.0.1',
  adminPort: 7400,
  adminUser: 'admin',
  adminPassword: 'admin',
  enabled: true,
  createdAt: Date.now(),
  updatedAt: Date.now(),
});

const rules = {
  name: [{ required: true, message: t('server.placeholder.name'), trigger: 'blur' }],
  serverAddr: [{ required: true, message: t('server.placeholder.serverAddr'), trigger: 'blur' }],
  serverPort: [
    { required: true, message: t('server.placeholder.serverPort'), trigger: 'blur' },
    { type: 'number', min: 1, max: 65535, message: '端口范围 1-65535', trigger: 'blur' }
  ],
};

function openAddDialog() {
  editingServer.value = null;
  form.value = {
    id: crypto.randomUUID(),
    name: '',
    serverAddr: '',
    serverPort: 7000,
    authMethod: 'token',
    token: '',
    user: '',
    metaToken: '',
    tlsEnable: false,
    logLevel: 'info',
    logMaxDays: 3,
    adminAddr: '127.0.0.1',
    adminPort: 7400,
    adminUser: 'admin',
    adminPassword: 'admin',
    enabled: true,
    createdAt: Date.now(),
    updatedAt: Date.now(),
  };
  dialogVisible.value = true;
}

function openEditDialog(server: ServerConfig) {
  editingServer.value = server;
  form.value = { ...server };
  dialogVisible.value = true;
}

function handleSave() {
  formRef.value.validate(async (valid: boolean) => {
    if (!valid) return;
    
    if (editingServer.value) {
      appStore.updateServer(editingServer.value.id, form.value);
      ElMessage.success(t('message.saveSuccess'));
    } else {
      appStore.addServer(form.value);
      ElMessage.success(t('message.saveSuccess'));
    }
    
    dialogVisible.value = false;
  });
}

function handleDelete(server: ServerConfig) {
  ElMessageBox.confirm(t('message.deleteConfirm'), t('common.warning'), {
    confirmButtonText: t('common.confirm'),
    cancelButtonText: t('common.cancel'),
    type: 'warning',
  }).then(() => {
    appStore.deleteServer(server.id);
    ElMessage.success(t('message.deleteSuccess'));
  }).catch(() => {});
}

function testConnection(server: ServerConfig) {
  ElMessage.info('测试连接功能开发中...');
  // TODO: 调用后端测试连接
}

const servers = computed(() => appStore.servers);
</script>

<template>
  <div class="servers-page">
    <div class="page-header">
      <h2 class="page-title">{{ t('server.title') }}</h2>
      <el-button type="primary" :icon="Plus" @click="openAddDialog">
        {{ t('server.addServer') }}
      </el-button>
    </div>

    <!-- 服务器列表 -->
    <el-table :data="servers" style="width: 100%" v-loading="servers.length === 0">
      <el-table-column prop="name" :label="t('server.name')" min-width="150" />
      <el-table-column prop="serverAddr" :label="t('server.serverAddr')" min-width="150" />
      <el-table-column prop="serverPort" :label="t('server.serverPort')" width="100" />
      <el-table-column prop="authMethod" :label="t('server.authMethod')" width="120">
        <template #default="{ row }">
          <el-tag size="small">{{ row.authMethod || 'token' }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="tlsEnable" :label="t('server.tlsEnable')" width="100">
        <template #default="{ row }">
          <el-tag :type="row.tlsEnable ? 'success' : 'info'" size="small">
            {{ row.tlsEnable ? '✓' : '✗' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="enabled" :label="t('proxy.enabled')" width="100">
        <template #default="{ row }">
          <el-switch v-model="row.enabled" size="small" @change="appStore.updateServer(row.id, { enabled: row.enabled })" />
        </template>
      </el-table-column>
      <el-table-column :label="t('common.operation')" width="220" fixed="right">
        <template #default="{ row }">
          <el-button size="small" @click="testConnection(row as ServerConfig)">
            {{ t('server.testConnection') }}
          </el-button>
          <el-button size="small" type="primary" @click="openEditDialog(row as ServerConfig)">
            {{ t('common.edit') }}
          </el-button>
          <el-button size="small" type="danger" @click="handleDelete(row as ServerConfig)">
            {{ t('common.delete') }}
          </el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-empty v-if="servers.length === 0" :description="t('server.title')" />

    <!-- 编辑/添加对话框 -->
    <el-dialog
      v-model="dialogVisible"
      :title="editingServer ? t('server.editServer') : t('server.addServer')"
      width="600px"
    >
      <el-form ref="formRef" :model="form" :rules="rules" label-width="120px">
        <el-form-item :label="t('server.name')" prop="name">
          <el-input v-model="form.name" :placeholder="t('server.placeholder.name')" />
        </el-form-item>
        
        <el-row :gutter="16">
          <el-col :span="16">
            <el-form-item :label="t('server.serverAddr')" prop="serverAddr">
              <el-input v-model="form.serverAddr" :placeholder="t('server.placeholder.serverAddr')" />
            </el-form-item>
          </el-col>
          <el-col :span="8">
            <el-form-item :label="t('server.serverPort')" prop="serverPort">
              <el-input-number v-model="form.serverPort" :min="1" :max="65535" style="width: 100%" />
            </el-form-item>
          </el-col>
        </el-row>

        <el-form-item :label="t('server.authMethod')" prop="authMethod">
          <el-select v-model="form.authMethod" style="width: 100%">
            <el-option label="Token" value="token" />
            <el-option label="MultiUser" value="multiuser" />
            <el-option label="OIDC" value="oidc" />
          </el-select>
        </el-form-item>

        <el-form-item :label="t('server.token')" prop="token" v-if="form.authMethod === 'token'">
          <el-input v-model="form.token" type="password" show-password :placeholder="t('server.placeholder.token')" />
        </el-form-item>

        <el-form-item :label="t('server.user')" prop="user" v-if="form.authMethod === 'multiuser'">
          <el-input v-model="form.user" />
        </el-form-item>

        <el-form-item :label="t('server.metaToken')" prop="metaToken">
          <el-input v-model="form.metaToken" />
        </el-form-item>

        <el-form-item :label="t('server.tlsEnable')">
          <el-switch v-model="form.tlsEnable" />
        </el-form-item>

        <el-divider />

        <el-form-item :label="t('server.logLevel')">
          <el-select v-model="form.logLevel" style="width: 100%">
            <el-option label="Debug" value="debug" />
            <el-option label="Info" value="info" />
            <el-option label="Warn" value="warn" />
            <el-option label="Error" value="error" />
          </el-select>
        </el-form-item>

        <el-row :gutter="16">
          <el-col :span="12">
            <el-form-item :label="t('server.adminAddr')">
              <el-input v-model="form.adminAddr" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item :label="t('server.adminPort')">
              <el-input-number v-model="form.adminPort" :min="1" :max="65535" style="width: 100%" />
            </el-form-item>
          </el-col>
        </el-row>
      </el-form>

      <template #footer>
        <el-button @click="dialogVisible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" @click="handleSave">{{ t('common.save') }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped lang="scss">
.servers-page {
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
