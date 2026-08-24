<script setup lang="ts">
import { ref, computed } from 'vue';
import { useAppStore } from '@/stores/app';
import { PlusOutlined } from '@ant-design/icons-vue';
import { message, Modal } from 'ant-design-vue';

const appStore = useAppStore();

const modalVisible = ref(false);
const form = ref<any>({
  name: '', type: 'tcp', local_ip: '127.0.0.1', local_port: 8080, remote_port: 8080, enabled: true,
});

const proxyTypes = [
  { value: 'tcp', label: 'TCP' },
  { value: 'udp', label: 'UDP' },
  { value: 'http', label: 'HTTP' },
  { value: 'https', label: 'HTTPS' },
  { value: 'stcp', label: 'STCP' },
  { value: 'xtcp', label: 'XTCP' },
];

const columns = [
  { title: '名称', dataIndex: 'name', key: 'name' },
  { title: '类型', dataIndex: 'type', key: 'type', width: 100 },
  { title: '本地地址', key: 'local', customRender: ({ record }: { record: any }) => `${record.local_ip}:${record.local_port}` },
  { title: '远程端口', dataIndex: 'remote_port', key: 'remote_port', width: 100 },
  { title: '状态', key: 'enabled', width: 100 },
  { title: '操作', key: 'action', width: 180, fixed: 'right' as const },
];

function openAdd() {
  form.value = { name: '', type: 'tcp', local_ip: '127.0.0.1', local_port: 8080, remote_port: 8080, enabled: true };
  modalVisible.value = true;
}

function handleSave() {
  if (!form.value.name) {
    message.warning('请输入代理名称');
    return;
  }
  appStore.addProxy(form.value);
  message.success('保存成功');
  modalVisible.value = false;
}

function handleDelete(proxy: any) {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除代理 "${proxy.name}" 吗？`,
    onOk: () => { appStore.deleteProxy(proxy.name); message.success('删除成功'); },
  });
}

function toggleProxy(proxy: any) {
  appStore.updateProxy(proxy.name, { enabled: !proxy.enabled });
  message.info(proxy.enabled ? '已停止' : '已启动');
}

const proxies = computed(() => appStore.proxies);
</script>

<template>
  <div class="proxies-page">
    <div class="page-header">
      <h2 class="page-title">代理管理</h2>
      <a-button type="primary" :icon="h(PlusOutlined)" @click="openAdd">添加代理</a-button>
    </div>

    <a-table :data-source="proxies" :columns="columns" row-key="name" :pagination="false">
      <template #bodyCell="{ column, record }">
        <template v-if="column.key === 'type'">
          <a-tag color="blue">{{ record.type.toUpperCase() }}</a-tag>
        </template>
        <template v-if="column.key === 'enabled'">
          <a-tag :color="record.enabled ? 'green' : 'default'">{{ record.enabled ? '运行中' : '已停止' }}</a-tag>
        </template>
        <template v-if="column.key === 'action'">
          <a-button size="small" :type="record.enabled ? 'default' : 'primary'" @click="toggleProxy(record)">
            {{ record.enabled ? '停止' : '启动' }}
          </a-button>
          <a-button size="small" danger @click="handleDelete(record)" style="margin-left: 8px">删除</a-button>
        </template>
      </template>
    </a-table>

    <a-empty v-if="proxies.length === 0" description="暂无代理配置" style="margin-top: 48px" />

    <a-modal v-model:open="modalVisible" title="添加代理" @ok="handleSave">
      <a-form :model="form" layout="vertical">
        <a-form-item label="名称" required>
          <a-input v-model:value="form.name" placeholder="请输入代理名称" />
        </a-form-item>
        <a-form-item label="类型" required>
          <a-select v-model:value="form.type">
            <a-select-option v-for="t in proxyTypes" :key="t.value" :value="t.value">{{ t.label }}</a-select-option>
          </a-select>
        </a-form-item>
        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item label="本地 IP">
              <a-input v-model:value="form.local_ip" />
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="本地端口">
              <a-input-number v-model:value="form.local_port" :min="1" :max="65535" style="width: 100%" />
            </a-form-item>
          </a-col>
        </a-row>
        <a-form-item label="远程端口">
          <a-input-number v-model:value="form.remote_port" :min="1" :max="65535" style="width: 100%" />
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
.proxies-page { padding: 24px; height: calc(100vh - 60px); overflow-y: auto; }
.page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; }
.page-title { font-size: 24px; font-weight: 600; }
</style>
