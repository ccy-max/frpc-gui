<script setup lang="ts">
import { ref, computed } from 'vue';
import { useAppStore } from '@/stores/app';
import { PlusOutlined } from '@ant-design/icons-vue';
import { message, Modal } from 'ant-design-vue';

const appStore = useAppStore();

const modalVisible = ref(false);
const editingId = ref<string | null>(null);
const form = ref<any>({
  id: '',
  name: '',
  serverAddr: '',
  serverPort: 7000,
  token: '',
  tlsEnable: false,
  enabled: true,
});

const columns = [
  { title: '名称', dataIndex: 'name', key: 'name' },
  { title: '服务器地址', dataIndex: 'serverAddr', key: 'serverAddr' },
  { title: '端口', dataIndex: 'serverPort', key: 'serverPort', width: 80 },
  { title: 'TLS', key: 'tlsEnable', width: 60 },
  { title: '启用', key: 'enabled', width: 80 },
  { title: '操作', key: 'action', width: 180, fixed: 'right' as const },
];

function openAdd() {
  editingId.value = null;
  form.value = { id: '', name: '', serverAddr: '', serverPort: 7000, token: '', tlsEnable: false, enabled: true };
  modalVisible.value = true;
}

function openEdit(server: any) {
  editingId.value = server.id;
  form.value = { ...server };
  modalVisible.value = true;
}

function handleSave() {
  if (!form.value.name || !form.value.serverAddr) {
    message.warning('请填写必填项');
    return;
  }
  if (editingId.value) {
    appStore.updateServer(editingId.value, form.value);
    message.success('保存成功');
  } else {
    form.value.id = Date.now().toString();
    appStore.addServer(form.value);
    message.success('保存成功');
  }
  modalVisible.value = false;
}

function handleDelete(server: any) {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除服务器 "${server.name}" 吗？`,
    onOk: () => {
      appStore.deleteServer(server.id);
      message.success('删除成功');
    },
  });
}

const servers = computed(() => appStore.servers);
</script>

<template>
  <div class="servers-page">
    <div class="page-header">
      <h2 class="page-title">服务器管理</h2>
      <a-button type="primary" :icon="h(PlusOutlined)" @click="openAdd">添加服务器</a-button>
    </div>

    <a-table :data-source="servers" :columns="columns" row-key="id" :pagination="false">
      <template #bodyCell="{ column, record }">
        <template v-if="column.key === 'tlsEnable'">
          <a-tag :color="record.tlsEnable ? 'green' : 'default'">{{ record.tlsEnable ? '✓' : '✗' }}</a-tag>
        </template>
        <template v-if="column.key === 'enabled'">
          <a-switch v-model:checked="record.enabled" size="small" />
        </template>
        <template v-if="column.key === 'action'">
          <a-button size="small" type="primary" @click="openEdit(record)">编辑</a-button>
          <a-button size="small" danger @click="handleDelete(record)" style="margin-left: 8px">删除</a-button>
        </template>
      </template>
    </a-table>

    <a-empty v-if="servers.length === 0" description="暂无服务器配置" style="margin-top: 48px" />

    <!-- 编辑/添加对话框 -->
    <a-modal v-model:open="modalVisible" :title="editingId ? '编辑服务器' : '添加服务器'" @ok="handleSave">
      <a-form :model="form" layout="vertical">
        <a-form-item label="名称" required>
          <a-input v-model:value="form.name" placeholder="请输入服务器名称" />
        </a-form-item>
        <a-form-item label="服务器地址" required>
          <a-input v-model:value="form.serverAddr" placeholder="例如：127.0.0.1" />
        </a-form-item>
        <a-form-item label="端口" required>
          <a-input-number v-model:value="form.serverPort" :min="1" :max="65535" style="width: 100%" />
        </a-form-item>
        <a-form-item label="令牌">
          <a-input-password v-model:value="form.token" placeholder="请输入令牌" />
        </a-form-item>
        <a-form-item label="启用 TLS">
          <a-switch v-model:checked="form.tlsEnable" />
        </a-form-item>
        <a-form-item label="启用">
          <a-switch v-model:checked="form.enabled" />
        </a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>

<script lang="ts">
import { h } from 'vue';
import { PlusOutlined } from '@ant-design/icons-vue';
export default { methods: { h } };
</script>

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
}
</style>
