<script setup lang="ts">
import { ref, computed } from 'vue';
import { useAppStore } from '@/stores/app';
import { PlusOutlined } from '@ant-design/icons-vue';
import { message, Modal } from 'ant-design-vue';

const appStore = useAppStore();
const searchKeyword = ref('');
const modalVisible = ref(false);

const form = ref<any>({
  name: '', type: 'tcp', local_ip: '127.0.0.1', local_port: 8080,
  remote_port: 8080, enabled: true,
  bandwidth_limit: '', traffic_limit: 0,
  custom_domains: [] as string[], subdomain: '', locations: [] as string[],
  http_user: '', http_password: '', host_header_rewrite: '',
  secret_key: '', allow_users: [] as string[],
  health_check_type: undefined as 'tcp' | 'http' | undefined,
  health_check_interval_s: 10, health_check_timeout_s: 3,
  health_check_max_unhealthy_times: 3, health_check_path: '/health',
  use_encryption: false, use_compression: false,
});

const proxyTypes = [
  { value: 'tcp', label: 'TCP' }, { value: 'udp', label: 'UDP' },
  { value: 'http', label: 'HTTP' }, { value: 'https', label: 'HTTPS' },
  { value: 'stcp', label: 'STCP' }, { value: 'xtcp', label: 'XTCP' },
  { value: 'sudp', label: 'SUDP' }, { value: 'tcpmux', label: 'TCPMUX' },
];

const filteredProxies = computed(() => {
  if (!searchKeyword.value.trim()) return appStore.proxies;
  const kw = searchKeyword.value.toLowerCase();
  return appStore.proxies.filter(p =>
    p.name.toLowerCase().includes(kw) || p.type.toLowerCase().includes(kw)
  );
});

const columns = [
  { title: '名称', dataIndex: 'name', key: 'name', ellipsis: true, width: 150 },
  { title: '类型', dataIndex: 'type', key: 'type', width: 100 },
  { title: '本地地址', key: 'local', width: 180, customRender: ({ record }: any) => `${record.local_ip}:${record.local_port}` },
  { title: '远程端口', dataIndex: 'remote_port', key: 'remote_port', width: 100 },
  { title: '状态', key: 'enabled', width: 80 },
  { title: '操作', key: 'action', width: 220, fixed: 'right' as const },
];

function openAdd() {
  modalVisible.value = true;
  form.value = {
    name: '', type: 'tcp', local_ip: '127.0.0.1', local_port: 8080,
    remote_port: 8080, enabled: true, bandwidth_limit: '', traffic_limit: 0,
    custom_domains: [], subdomain: '', locations: [],
    http_user: '', http_password: '', host_header_rewrite: '',
    secret_key: '', allow_users: [], health_check_type: undefined,
    health_check_interval_s: 10, health_check_timeout_s: 3,
    health_check_max_unhealthy_times: 3, health_check_path: '/health',
    use_encryption: false, use_compression: false,
  };
}

function openEdit(proxy: any) {
  modalVisible.value = true;
  form.value = { ...proxy };
}

function handleSave() {
  if (!form.value.name) { message.warning('请输入代理名称'); return; }
  appStore.addProxy(form.value);
  message.success('保存成功');
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

const isHttpType = computed(() => ['http', 'https'].includes(form.value.type));
const isStcpType = computed(() => ['stcp', 'xtcp', 'sudp'].includes(form.value.type));
</script>

<template>
  <div class="page-container">
    <a-page-header title="代理管理">
      <template #extra>
        <a-button type="primary" @click="openAdd">
          <template #icon><PlusOutlined /></template>
          添加代理
        </a-button>
      </template>
    </a-page-header>

    <a-card :bordered="false">
      <!-- 搜索栏 -->
      <div class="search-bar" style="margin-bottom: 16px">
        <a-input
          v-model:value="searchKeyword"
          placeholder="搜索代理名称或类型"
          style="width: 300px"
          allow-clear
        />
      </div>

      <!-- 表格 -->
      <a-table
        :data-source="filteredProxies"
        :columns="columns"
        row-key="name"
        :pagination="{ pageSize: 10, showSizeChanger: true }"
        :scroll="{ x: 1200 }"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'type'">
            <a-tag color="blue">{{ record.type.toUpperCase() }}</a-tag>
          </template>
          <template v-if="column.key === 'enabled'">
            <a-tag :color="record.enabled ? 'green' : 'default'">
              {{ record.enabled ? '运行中' : '已停止' }}
            </a-tag>
          </template>
          <template v-if="column.key === 'action'">
            <a-space>
              <a-button
                size="small"
                :type="record.enabled ? 'default' : 'primary'"
                @click="toggleProxy(record)"
              >
                {{ record.enabled ? '停止' : '启动' }}
              </a-button>
              <a-button size="small" @click="openEdit(record)">编辑</a-button>
              <a-popconfirm title="确定删除？" @confirm="handleDelete(record)">
                <a-button size="small" danger>删除</a-button>
              </a-popconfirm>
            </a-space>
          </template>
        </template>
      </a-table>

      <a-empty v-if="filteredProxies.length === 0" description="暂无代理配置" style="margin-top: 48px" />
    </a-card>

    <!-- 编辑对话框 -->
    <a-modal
      v-model:open="modalVisible"
      :title="form.name ? '编辑代理' : '添加代理'"
      @ok="handleSave"
      width="900px"
    >
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
                <a-select-option v-for="t in proxyTypes" :key="t.value" :value="t.value">
                  {{ t.label }}
                </a-select-option>
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

        <!-- HTTP/HTTPS 配置 -->
        <template v-if="isHttpType">
          <a-divider orientation="left">HTTP/HTTPS 配置</a-divider>
          <a-form-item label="自定义域名">
            <a-select v-model:value="form.custom_domains" mode="tags" style="width: 100%" />
          </a-form-item>
          <a-row :gutter="16">
            <a-col :span="12">
              <a-form-item label="子域名">
                <a-input v-model:value="form.subdomain" />
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="Host 重写">
                <a-input v-model:value="form.host_header_rewrite" />
              </a-form-item>
            </a-col>
          </a-row>
          <a-row :gutter="16">
            <a-col :span="12">
              <a-form-item label="HTTP 用户">
                <a-input v-model:value="form.http_user" />
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="HTTP 密码">
                <a-input-password v-model:value="form.http_password" />
              </a-form-item>
            </a-col>
          </a-row>
        </template>

        <!-- STCP/XTCP 配置 -->
        <template v-if="isStcpType">
          <a-divider orientation="left">STCP/XTCP 配置</a-divider>
          <a-form-item label="共享密钥">
            <a-input-password v-model:value="form.secret_key" />
          </a-form-item>
          <a-form-item label="允许访问者">
            <a-select v-model:value="form.allow_users" mode="tags" style="width: 100%" />
          </a-form-item>
        </template>

        <!-- 健康检查 -->
        <a-divider orientation="left">健康检查</a-divider>
        <a-row :gutter="16">
          <a-col :span="8">
            <a-form-item label="检查类型">
              <a-select v-model:value="form.health_check_type" allow-clear>
                <a-select-option value="tcp">TCP</a-select-option>
                <a-select-option value="http">HTTP</a-select-option>
              </a-select>
            </a-form-item>
          </a-col>
          <a-col :span="8" v-if="form.health_check_type === 'http'">
            <a-form-item label="检查路径">
              <a-input v-model:value="form.health_check_path" />
            </a-form-item>
          </a-col>
        </a-row>
        <a-row :gutter="16" v-if="form.health_check_type">
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

<style scoped>
.page-container {
  padding: 24px;
}

.search-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>
