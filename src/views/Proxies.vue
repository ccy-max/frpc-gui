<script setup lang="ts">
import { ref, computed } from 'vue';
import { useAppStore } from '@/stores/app';
import { Plus } from '@element-plus/icons-vue';

const appStore = useAppStore();

const dialogVisible = ref(false);
const form = ref({
  name: '',
  type: 'tcp',
  local_ip: '127.0.0.1',
  local_port: 8080,
  remote_port: 8080,
  enabled: true,
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
    local_ip: '127.0.0.1',
    local_port: 8080,
    remote_port: 8080,
    enabled: true,
  };
  dialogVisible.value = true;
}

function handleSave() {
  if (!form.value.name) {
    return;
  }
  appStore.addProxy(form.value);
  dialogVisible.value = false;
}

function handleDelete(proxy: any) {
  appStore.deleteProxy(proxy.name);
}

function toggleProxy(proxy: any) {
  appStore.updateProxy(proxy.name, { enabled: !proxy.enabled });
}

const proxies = computed(() => appStore.proxies);
</script>

<template>
  <div class="proxies-page">
    <div class="page-header">
      <h2 class="page-title">代理管理</h2>
      <el-button type="primary" :icon="Plus" @click="openAddDialog">
        添加代理
      </el-button>
    </div>

    <el-table :data="proxies" style="width: 100%">
      <el-table-column prop="name" label="名称" min-width="120" />
      <el-table-column prop="type" label="类型" width="100">
        <template #default="{ row }">
          <el-tag size="small" type="info">{{ row.type.toUpperCase() }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="本地地址" min-width="150">
        <template #default="{ row }">
          {{ row.local_ip }}:{{ row.local_port }}
        </template>
      </el-table-column>
      <el-table-column prop="remote_port" label="远程端口" width="100" />
      <el-table-column prop="enabled" label="状态" width="100">
        <template #default="{ row }">
          <el-tag :type="row.enabled ? 'success' : 'info'" size="small">
            {{ row.enabled ? '运行中' : '已停止' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column label="操作" width="220" fixed="right">
        <template #default="{ row }">
          <el-button size="small" :type="row.enabled ? 'warning' : 'success'" @click="toggleProxy(row)">
            {{ row.enabled ? '停止' : '启动' }}
          </el-button>
          <el-button size="small" type="danger" @click="handleDelete(row)">
            删除
          </el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-empty v-if="proxies.length === 0" description="暂无代理配置" />

    <!-- 添加代理对话框 -->
    <el-dialog v-model="dialogVisible" title="添加代理" width="500px">
      <el-form :model="form" label-width="100px">
        <el-form-item label="名称" required>
          <el-input v-model="form.name" placeholder="请输入代理名称" />
        </el-form-item>
        
        <el-form-item label="类型" required>
          <el-select v-model="form.type" style="width: 100%">
            <el-option v-for="item in proxyTypes" :key="item.value" :label="item.label" :value="item.value" />
          </el-select>
        </el-form-item>

        <el-row :gutter="16">
          <el-col :span="12">
            <el-form-item label="本地 IP">
              <el-input v-model="form.local_ip" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="本地端口">
              <el-input-number v-model="form.local_port" :min="1" :max="65535" style="width: 100%" />
            </el-form-item>
          </el-col>
        </el-row>

        <el-form-item label="远程端口">
          <el-input-number v-model="form.remote_port" :min="1" :max="65535" style="width: 100%" />
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
