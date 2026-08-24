<script setup lang="ts">
import { ref, computed } from 'vue';
import { useAppStore } from '@/stores/app';
import { PlusOutlined } from '@ant-design/icons-vue';
import { message, Modal } from 'ant-design-vue';

const appStore = useAppStore();
const modalVisible = ref(false);
const editingProxyName = ref<string | null>(null);

const form = ref<any>({
  name: '',
  type: 'tcp',
  local_ip: '127.0.0.1',
  local_port: 8080,
  remote_port: 8080,
  enabled: true,
  // 带宽/流量
  bandwidth_limit: '',
  traffic_limit: 0,
  // HTTP/HTTPS
  custom_domains: [] as string[],
  subdomain: '',
  locations: [] as string[],
  http_user: '',
  http_password: '',
  host_header_rewrite: '',
  // STCP/XTCP
  secret_key: '',
  allow_users: [] as string[],
  // 健康检查
  health_check_type: 'tcp' as 'tcp' | 'http' | undefined,
  health_check_interval_s: 10,
  health_check_timeout_s: 3,
  health_check_max_unhealthy_times: 3,
  health_check_path: '/health',
  // 其他
  use_encryption: false,
  use_compression: false,
});

const proxyTypes = [
  { value: 'tcp', label: 'TCP' },
  { value: 'udp', label: 'UDP' },
  { value: 'http', label: 'HTTP' },
  { value: 'https', label: 'HTTPS' },
  { value: 'stcp', label: 'STCP' },
  { value: 'xtcp', label: 'XTCP' },
  { value: 'sudp', label: 'SUDP' },
  { value: 'tcpmux', label: 'TCPMUX' },
];

const healthCheckTypes = [
  { value: '', label: '无' },
  { value: 'tcp', label: 'TCP' },
  { value: 'http', label: 'HTTP' },
];

const columns = [
  { title: '名称', dataIndex: 'name', key: 'name' },
  { title: '类型', dataIndex: 'type', key: 'type', width: 100 },
  { title: '本地地址', key: 'local', customRender: ({ record }: any) => `${record.local_ip}:${record.local_port}` },
  { title: '远程端口', dataIndex: 'remote_port', key: 'remote_port', width: 100 },
  { title: '状态', key: 'enabled', width: 100 },
  { title: '操作', key: 'action', width: 180, fixed: 'right' as const },
];

function openAdd() {
  editingProxyName.value = null;
  form.value = {
    name: '',
    type: 'tcp',
    local_ip: '127.0.0.1',
    local_port: 8080,
    remote_port: 8080,
    enabled: true,
    bandwidth_limit: '',
    traffic_limit: 0,
    custom_domains: [],
    subdomain: '',
    locations: [],
    http_user: '',
    http_password: '',
    host_header_rewrite: '',
    secret_key: '',
    allow_users: [],
    health_check_type: undefined,
    health_check_interval_s: 10,
    health_check_timeout_s: 3,
    health_check_max_unhealthy_times: 3,
    health_check_path: '/health',
    use_encryption: false,
    use_compression: false,
  };
  modalVisible.value = true;
}

function openEdit(proxy: any) {
  editingProxyName.value = proxy.name;
  form.value = { ...proxy };
  modalVisible.value = true;
}

function handleSave() {
  if (!form.value.name) {
    message.warning('请输入代理名称');
    return;
  }
  
  if (editingProxyName.value) {
    appStore.updateProxy(editingProxyName.value, form.value);
    message.success('保存成功');
  } else {
    appStore.addProxy(form.value);
    message.success('添加成功');
  }
  modalVisible.value = false;
}

function handleDelete(proxy: any) {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除代理 "${proxy.name}" 吗？`,
    onOk: () => {
      appStore.deleteProxy(proxy.name);
      message.success('删除成功');
    },
  });
}

async function toggleProxy(proxy: any) {
  await appStore.modifyProxyStatus(proxy.name, !proxy.enabled);
  message.info(proxy.enabled ? '已停止' : '已启动');
}

const proxies = computed(() => appStore.proxies);

// 显示高级选项
const showAdvanced = ref(false);

// 根据类型显示不同字段
const isHttpType = computed(() => ['http', 'https'].includes(form.value.type));
const isStcpType = computed(() => ['stcp', 'xtcp', 'sudp'].includes(form.value.type));
</script>

<template>
  <div class="proxies-page">
    <div class="page-header">
      <h2 class="page-title">代理管理</h2>
      <a-button type="primary" @click="openAdd">
        <template #icon><PlusOutlined /></template>
        添加代理
      </a-button>
    </div>

    <a-table :data-source="proxies" :columns="columns" row-key="name" :pagination="false" scroll-x="1000">
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
          <a-button size="small" @click="openEdit(record)" style="margin-left: 8px">编辑</a-button>
          <a-button size="small" danger @click="handleDelete(record)" style="margin-left: 8px">删除</a-button>
        </template>
      </template>
    </a-table>

    <a-empty v-if="proxies.length === 0" description="暂无代理配置" style="margin-top: 48px" />

    <!-- 代理编辑对话框 -->
    <a-modal v-model:open="modalVisible" :title="editingProxyName ? '编辑代理' : '添加代理'" @ok="handleSave" width="800px">
      <a-form :model="form" layout="vertical">
        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item label="名称" required>
              <a-input v-model:value="form.name" placeholder="请输入代理名称" />
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="类型" required>
              <a-select v-model:value="form.type">
                <a-select-option v-for="t in proxyTypes" :key="t.value" :value="t.value">{{ t.label }}</a-select-option>
              </a-select>
            </a-form-item>
          </a-col>
        </a-row>

        <a-row :gutter="16">
          <a-col :span="8">
            <a-form-item label="本地 IP">
              <a-input v-model:value="form.local_ip" placeholder="127.0.0.1" />
            </a-form-item>
          </a-col>
          <a-col :span="8">
            <a-form-item label="本地端口">
              <a-input-number v-model:value="form.local_port" :min="1" :max="65535" style="width: 100%" />
            </a-form-item>
          </a-col>
          <a-col :span="8">
            <a-form-item label="远程端口">
              <a-input-number v-model:value="form.remote_port" :min="1" :max="65535" style="width: 100%" />
            </a-form-item>
          </a-col>
        </a-row>

        <!-- 带宽/流量限制 -->
        <a-divider orientation="left">
          <a-space>
            <span>带宽/流量</span>
            <a-button type="link" size="small" @click="showAdvanced = !showAdvanced">{{ showAdvanced ? '收起' : '展开' }}</a-button>
          </a-space>
        </a-divider>
        
        <div v-if="showAdvanced">
          <a-row :gutter="16">
            <a-col :span="12">
              <a-form-item label="带宽限制">
                <a-input v-model:value="form.bandwidth_limit" placeholder="例：1MB" addon-after="MB/s" />
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="流量限制">
                <a-input-number v-model:value="form.traffic_limit" :min="0" style="width: 100%" addon-after="GB" />
              </a-form-item>
            </a-col>
          </a-row>
        </div>

        <!-- HTTP/HTTPS 配置 -->
        <template v-if="isHttpType">
          <a-divider orientation="left">HTTP/HTTPS 配置</a-divider>
          
          <a-form-item label="自定义域名">
            <a-select v-model:value="form.custom_domains" mode="tags" style="width: 100%" placeholder="输入域名后回车" />
          </a-form-item>
          
          <a-row :gutter="16">
            <a-col :span="12">
              <a-form-item label="子域名">
                <a-input v-model:value="form.subdomain" placeholder="留空则使用自定义域名" />
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="Host 重写">
                <a-input v-model:value="form.host_header_rewrite" placeholder="例：example.com" />
              </a-form-item>
            </a-col>
          </a-row>
          
          <a-form-item label="路由路径">
            <a-select v-model:value="form.locations" mode="tags" style="width: 100%" placeholder="例：/api, /web" />
          </a-form-item>
          
          <a-row :gutter="16">
            <a-col :span="12">
              <a-form-item label="HTTP 用户">
                <a-input v-model:value="form.http_user" placeholder="Basic Auth 用户名" />
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="HTTP 密码">
                <a-input-password v-model:value="form.http_password" placeholder="Basic Auth 密码" />
              </a-form-item>
            </a-col>
          </a-row>
        </template>

        <!-- STCP/XTCP 配置 -->
        <template v-if="isStcpType">
          <a-divider orientation="left">STCP/XTCP 配置</a-divider>
          
          <a-form-item label="共享密钥">
            <a-input-password v-model:value="form.secret_key" placeholder="用于验证访问者" />
          </a-form-item>
          
          <a-form-item label="允许访问者">
            <a-select v-model:value="form.allow_users" mode="tags" style="width: 100%" placeholder="留空表示允许所有人" />
          </a-form-item>
        </template>

        <!-- 健康检查 -->
        <a-divider orientation="left">健康检查</a-divider>
        
        <a-row :gutter="16">
          <a-col :span="8">
            <a-form-item label="检查类型">
              <a-select v-model:value="form.health_check_type">
                <a-select-option v-for="t in healthCheckTypes" :key="t.value" :value="t.value">{{ t.label }}</a-select-option>
              </a-select>
            </a-form-item>
          </a-col>
          <a-col :span="8" v-if="form.health_check_type">
            <a-form-item label="检查路径" v-if="form.health_check_type === 'http'">
              <a-input v-model:value="form.health_check_path" placeholder="/health" />
            </a-form-item>
          </a-col>
        </a-row>
        
        <div v-if="form.health_check_type">
          <a-row :gutter="16">
            <a-col :span="6">
              <a-form-item label="间隔 (秒)">
                <a-input-number v-model:value="form.health_check_interval_s" :min="1" style="width: 100%" />
              </a-form-item>
            </a-col>
            <a-col :span="6">
              <a-form-item label="超时 (秒)">
                <a-input-number v-model:value="form.health_check_timeout_s" :min="1" style="width: 100%" />
              </a-form-item>
            </a-col>
            <a-col :span="6">
              <a-form-item label="最大失败次数">
                <a-input-number v-model:value="form.health_check_max_unhealthy_times" :min="1" style="width: 100%" />
              </a-form-item>
            </a-col>
          </a-row>
        </div>

        <!-- 其他选项 -->
        <a-divider orientation="left">其他选项</a-divider>
        
        <a-space>
          <a-checkbox v-model:checked="form.use_encryption">启用加密</a-checkbox>
          <a-checkbox v-model:checked="form.use_compression">启用压缩</a-checkbox>
        </a-space>
      </a-form>
    </a-modal>
  </div>
</template>

<style scoped lang="scss">
.proxies-page { padding: 24px; height: calc(100vh - 60px); overflow-y: auto; }
.page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; }
.page-title { font-size: 24px; font-weight: 600; }
</style>
