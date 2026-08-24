<script setup lang="ts">
import { ref, computed } from 'vue';
import { useAppStore } from '@/stores/app';
import { useI18n } from 'vue-i18n';
import { Plus } from '@element-plus/icons-vue';

const appStore = useAppStore();
const { t } = useI18n();

const dialogVisible = ref(false);
const editingId = ref<string | null>(null);

const form = ref({
  id: '',
  name: '',
  serverAddr: '',
  serverPort: 7000,
  token: '',
  tlsEnable: false,
  enabled: true,
});

function openAddDialog() {
  editingId.value = null;
  form.value = {
    id: '',
    name: '',
    serverAddr: '',
    serverPort: 7000,
    token: '',
    tlsEnable: false,
    enabled: true,
  };
  dialogVisible.value = true;
}

function openEditDialog(server: any) {
  editingId.value = server.id;
  form.value = { ...server };
  dialogVisible.value = true;
}

function handleSave() {
  if (!form.value.name || !form.value.serverAddr) {
    return;
  }
  
  if (editingId.value) {
    appStore.updateServer(editingId.value, form.value);
  } else {
    form.value.id = Date.now().toString();
    appStore.addServer(form.value);
  }
  
  dialogVisible.value = false;
}

function handleDelete(server: any) {
  appStore.deleteServer(server.id);
}

const servers = computed(() => appStore.servers);
</script>

<template>
  <div class="servers-page">
    <div class="page-header">
      <h2 class="page-title">服务器管理</h2>
      <el-button type="primary" :icon="Plus" @click="openAddDialog">
        添加服务器
      </el-button>
    </div>

    <el-table :data="servers" style="width: 100%">
      <el-table-column prop="name" label="名称" min-width="150" />
      <el-table-column prop="serverAddr" label="服务器地址" min-width="200" />
      <el-table-column prop="serverPort" label="端口" width="100" />
      <el-table-column prop="tlsEnable" label="TLS" width="80">
        <template #default="{ row }">
          <el-tag :type="row.tlsEnable ? 'success' : 'info'" size="small">
            {{ row.tlsEnable ? '✓' : '✗' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="enabled" label="启用" width="100">
        <template #default="{ row }">
          <el-switch v-model="row.enabled" size="small" />
        </template>
      </el-table-column>
      <el-table-column label="操作" width="220" fixed="right">
        <template #default="{ row }">
          <el-button size="small" type="primary" @click="openEditDialog(row)">
            编辑
          </el-button>
          <el-button size="small" type="danger" @click="handleDelete(row)">
            删除
          </el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-empty v-if="servers.length === 0" description="暂无服务器配置" />

    <!-- 编辑/添加对话框 -->
    <el-dialog
      v-model="dialogVisible"
      :title="editingId ? '编辑服务器' : '添加服务器'"
      width="500px"
    >
      <el-form :model="form" label-width="100px">
        <el-form-item label="名称" required>
          <el-input v-model="form.name" placeholder="请输入服务器名称" />
        </el-form-item>
        
        <el-form-item label="服务器地址" required>
          <el-input v-model="form.serverAddr" placeholder="例如：127.0.0.1" />
        </el-form-item>
        
        <el-form-item label="端口" required>
          <el-input-number v-model="form.serverPort" :min="1" :max="65535" style="width: 100%" />
        </el-form-item>

        <el-form-item label="令牌">
          <el-input v-model="form.token" type="password" show-password />
        </el-form-item>

        <el-form-item label="启用 TLS">
          <el-switch v-model="form.tlsEnable" />
        </el-form-item>

        <el-form-item label="启用">
          <el-switch v-model="form.enabled" />
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleSave">保存</el-button>
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
