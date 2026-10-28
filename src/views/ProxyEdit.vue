<script setup lang="ts">
/**
 * 代理编辑页（替代原弹窗）
 *
 * 表单逻辑（校验、类型分支、端口范围）从 Proxies.vue 的 handleSave 迁入，
 * 由父页面在列表视图内切换显示，不新增路由以免丢失筛选状态。
 */
import { ref, computed, watch } from 'vue';
import { useAppStore } from '@/stores/app';
import { message } from 'ant-design-vue';
import SubPageHeader from '@/components/SubPageHeader.vue';
import {
  SaveOutlined, UndoOutlined, PlusOutlined, MinusCircleOutlined,
} from '@ant-design/icons-vue';

const props = defineProps<{
  /** 传入则为编辑，null 为新建 */
  proxy: any | null;
  /** 新建时的默认服务器（来自默认设置或活动服务器） */
  defaultServerId?: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'saved'): void;
}>();

const appStore = useAppStore();
const saving = ref(false);

const editingName = computed(() => props.proxy?.name ?? null);
const isEdit = computed(() => !!props.proxy?.name);

const proxyTypes = [
  { value: 'tcp', label: 'TCP' }, { value: 'udp', label: 'UDP' },
  { value: 'http', label: 'HTTP' }, { value: 'https', label: 'HTTPS' },
  { value: 'stcp', label: 'STCP' }, { value: 'xtcp', label: 'XTCP' },
  { value: 'sudp', label: 'SUDP' }, { value: 'tcpmux', label: 'TCPMUX' },
];

const form = ref<any>(blankForm());

function blankForm() {
  return {
    name: '', type: 'tcp', local_ip: '127.0.0.1', local_port: 8080,
    remote_port: 8080,
    server_id: props.defaultServerId || '',
    enabled: true,
    bandwidth_limit: '', traffic_limit: 0,
    custom_domains: [] as string[], subdomain: '', locations: [] as string[],
    http_user: '', http_password: '', host_header_rewrite: '',
    secret_key: '', allow_users: [] as string[],
    health_check_type: undefined as 'tcp' | 'http' | undefined,
    health_check_interval_s: 10, health_check_timeout_s: 3,
    health_check_max_unhealthy_times: 3, health_check_path: '/health',
    use_encryption: false, use_compression: false,
  };
}

function resetForm() {
  if (props.proxy) {
    form.value = { ...blankForm(), ...JSON.parse(JSON.stringify(props.proxy)) };
  } else {
    form.value = blankForm();
  }
}

watch(() => props.proxy, resetForm, { immediate: true });

const isHttpType = computed(() => ['http', 'https'].includes(form.value.type));
const isStcpType = computed(() => ['stcp', 'xtcp', 'sudp'].includes(form.value.type));

// 端口类代理才需要 remote_port
const needsRemotePort = computed(() => ['tcp', 'udp', 'tcpmux'].includes(form.value.type));

async function handleSave() {
  const f = form.value;

  if (!f.name?.trim()) {
    message.warning('请输入代理名称');
    return;
  }
  if (f.name.includes(' ')) {
    message.warning('代理名称不能包含空格');
    return;
  }
  // 名称重复校验（编辑时排除自己）
  const dup = appStore.proxies.some(p => p.name === f.name && p.name !== editingName.value);
  if (dup) {
    message.error(`代理名称「${f.name}」已存在，请换一个`);
    return;
  }
  // 端口范围
  if (f.local_port < 1 || f.local_port > 65535) {
    message.error('本地端口必须在 1-65535 之间');
    return;
  }
  if (needsRemotePort.value && (f.remote_port < 1 || f.remote_port > 65535)) {
    message.error('远程端口必须在 1-65535 之间');
    return;
  }
  // IP 格式
  const ipOk = /^(\d{1,3}\.){3}\d{1,3}$/.test(f.local_ip) || f.local_ip === 'localhost';
  if (!ipOk) {
    message.error('本地 IP 格式不正确（例：127.0.0.1）');
    return;
  }
  // HTTP 域名
  if (isHttpType.value && f.custom_domains?.length) {
    const domainRegex = /^[a-zA-Z0-9][-a-zA-Z0-9]*(\.[a-zA-Z0-9][-a-zA-Z0-9]*)*$/;
    for (const d of f.custom_domains) {
      if (!domainRegex.test(d)) {
        message.error(`域名格式无效：${d}`);
        return;
      }
    }
  }
  if (isStcpType.value && !f.secret_key) {
    message.warning('STCP/XTCP 建议设置共享密钥以提高安全性');
  }

  saving.value = true;
  try {
    if (editingName.value) {
      await appStore.updateProxy(editingName.value, f);
      message.success('代理已保存');
    } else {
      await appStore.addProxy(f);
      message.success('代理已添加');
    }
    emit('saved');
  } catch (e) {
    message.error(`保存失败：${typeof e === 'string' ? e : (e as Error)?.message ?? String(e)}`);
  } finally {
    saving.value = false;
  }
}

// 类型说明（帮助用户选对类型）
const typeHints: Record<string, string> = {
  tcp: '把本地 TCP 端口映射到服务器端口，最常用',
  udp: 'UDP 协议端口映射，适用于游戏、DNS 等',
  http: '通过域名访问本地 Web 服务，支持自定义域名',
  https: 'HTTPS 域名映射，可在 frps 配置证书',
  stcp: '点对点安全隧道，需访问端配合使用共享密钥',
  xtcp: '点对点直连（P2P），穿透效果更好但依赖网络条件',
  sudp: 'UDP 版点对点安全隧道',
  tcpmux: '多路复用，多个 HTTP 服务共用同一端口',
};
const typeHint = computed(() => typeHints[form.value.type] || '');
</script>

<template>
  <SubPageHeader
    :title="isEdit ? '编辑代理' : '添加代理'"
    :subtitle="isEdit ? `修改「${proxy?.name}」的映射规则` : '把本地服务映射到公网'"
    back-text="返回代理列表"
    :auto-back="false"
    @back="emit('close')"
  >
    <template #actions>
      <a-button :disabled="saving" @click="emit('close')">
        <template #icon><UndoOutlined /></template>
        取消
      </a-button>
      <a-button type="primary" :loading="saving" @click="handleSave">
        <template #icon><SaveOutlined /></template>
        {{ isEdit ? '保存修改' : '添加代理' }}
      </a-button>
    </template>
  </SubPageHeader>

  <div class="form-wrap">
    <a-form :model="form" layout="vertical">
      <!-- 基础信息 -->
      <div class="form-section">
        <div class="section-title">基础信息</div>
        <a-row :gutter="12">
          <a-col :span="12">
            <a-form-item label="代理名称" required>
              <a-input v-model:value="form.name" placeholder="例如：web-dashboard" />
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="代理类型" required>
              <a-select v-model:value="form.type">
                <a-select-option v-for="t in proxyTypes" :key="t.value" :value="t.value">
                  {{ t.label }}
                </a-select-option>
              </a-select>
              <div v-if="typeHint" class="field-hint">{{ typeHint }}</div>
            </a-form-item>
          </a-col>
        </a-row>
        <a-form-item label="所属服务器">
          <a-select v-model:value="form.server_id" placeholder="选择关联的服务器" allow-clear>
            <a-select-option
              v-for="server in appStore.servers"
              :key="server.id"
              :value="server.id"
            >
              {{ server.name }} ({{ server.serverAddr }})
            </a-select-option>
          </a-select>
          <div v-if="appStore.servers.length === 0" class="field-hint warn">
            还没有服务器，建议先去「服务器管理」添加
          </div>
        </a-form-item>
      </div>

      <!-- 端口映射 -->
      <div class="form-section">
        <div class="section-title">端口映射</div>
        <a-row :gutter="12">
          <a-col :span="8">
            <a-form-item label="本地 IP" required>
              <a-input v-model:value="form.local_ip" placeholder="127.0.0.1" />
            </a-form-item>
          </a-col>
          <a-col :span="8">
            <a-form-item label="本地端口" required>
              <a-input-number v-model:value="form.local_port" :min="1" :max="65535" style="width: 100%" />
            </a-form-item>
          </a-col>
          <a-col :span="8">
            <a-form-item label="远程端口" :required="needsRemotePort">
              <a-input-number
                v-model:value="form.remote_port"
                :min="1" :max="65535"
                :disabled="!needsRemotePort"
                style="width: 100%"
              />
              <div v-if="!needsRemotePort" class="field-hint">该类型由 frps 自动分配</div>
            </a-form-item>
          </a-col>
        </a-row>
      </div>

      <!-- HTTP/HTTPS 配置 -->
      <div v-if="isHttpType" class="form-section">
        <div class="section-title">HTTP 路由</div>
        <a-form-item label="自定义域名">
          <a-select
            v-model:value="form.custom_domains"
            mode="tags"
            placeholder="输入域名后回车，可添加多个"
            style="width: 100%"
          />
        </a-form-item>
        <a-row :gutter="12">
          <a-col :span="12">
            <a-form-item label="子域名">
              <a-input v-model:value="form.subdomain" placeholder="留空则使用自定义域名" />
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="Host 重写">
              <a-input v-model:value="form.host_header_rewrite" placeholder="可选" />
            </a-form-item>
          </a-col>
        </a-row>
        <a-row :gutter="12">
          <a-col :span="12">
            <a-form-item label="HTTP 基本认证用户">
              <a-input v-model:value="form.http_user" placeholder="可选" />
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="HTTP 认证密码">
              <a-input-password v-model:value="form.http_password" placeholder="可选" />
            </a-form-item>
          </a-col>
        </a-row>
      </div>

      <!-- STCP/XTCP/SUDP 配置 -->
      <div v-if="isStcpType" class="form-section">
        <div class="section-title">
          安全隧道
          <span class="section-hint">访问端需使用相同密钥</span>
        </div>
        <a-form-item label="共享密钥">
          <a-input-password v-model:value="form.secret_key" placeholder="建议设置，防止未授权访问" />
        </a-form-item>
        <a-form-item label="允许访问者">
          <a-select
            v-model:value="form.allow_users"
            mode="tags"
            placeholder="留空表示不限制，输入用户名后回车"
            style="width: 100%"
          />
        </a-form-item>
      </div>

      <!-- 健康检查 -->
      <div class="form-section">
        <div class="section-title">
          健康检查
          <span class="section-hint">可选 · 后端异常时自动摘除</span>
        </div>
        <a-row :gutter="12">
          <a-col :span="8">
            <a-form-item label="检查类型">
              <a-select v-model:value="form.health_check_type" allow-clear placeholder="不启用">
                <a-select-option value="tcp">TCP 端口探测</a-select-option>
                <a-select-option value="http">HTTP 请求</a-select-option>
              </a-select>
            </a-form-item>
          </a-col>
          <a-col v-if="form.health_check_type === 'http'" :span="16">
            <a-form-item label="检查路径">
              <a-input v-model:value="form.health_check_path" placeholder="/health" />
            </a-form-item>
          </a-col>
        </a-row>
        <a-row v-if="form.health_check_type" :gutter="12">
          <a-col :span="8">
            <a-form-item label="间隔（秒）">
              <a-input-number v-model:value="form.health_check_interval_s" :min="1" style="width: 100%" />
            </a-form-item>
          </a-col>
          <a-col :span="8">
            <a-form-item label="超时（秒）">
              <a-input-number v-model:value="form.health_check_timeout_s" :min="1" style="width: 100%" />
            </a-form-item>
          </a-col>
          <a-col :span="8">
            <a-form-item label="最大失败次数">
              <a-input-number v-model:value="form.health_check_max_unhealthy_times" :min="1" style="width: 100%" />
            </a-form-item>
          </a-col>
        </a-row>
      </div>

      <!-- 高级选项 -->
      <div class="form-section">
        <div class="section-title">高级选项</div>
        <a-row :gutter="12">
          <a-col :span="12">
            <a-form-item label="带宽限制">
              <a-input v-model:value="form.bandwidth_limit" placeholder="例如 1MB，留空不限" />
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="流量限制（MB）">
              <a-input-number v-model:value="form.traffic_limit" :min="0" style="width: 100%" />
              <div class="field-hint">0 表示不限制</div>
            </a-form-item>
          </a-col>
        </a-row>
        <div class="switch-row">
          <div class="switch-text">
            <div class="switch-label">启用加密</div>
            <div class="switch-desc">与 frps 之间的流量加密传输</div>
          </div>
          <a-switch v-model:checked="form.use_encryption" />
        </div>
        <div class="switch-row">
          <div class="switch-text">
            <div class="switch-label">启用压缩</div>
            <div class="switch-desc">压缩传输内容，节省带宽但增加 CPU 开销</div>
          </div>
          <a-switch v-model:checked="form.use_compression" />
        </div>
        <div class="switch-row">
          <div class="switch-text">
            <div class="switch-label">启用此代理</div>
            <div class="switch-desc">关闭后不会随服务器一起启动</div>
          </div>
          <a-switch v-model:checked="form.enabled" />
        </div>
      </div>
    </a-form>
  </div>
</template>

<style scoped>
.form-wrap { max-width: 720px; }

.form-section {
  background: #fff;
  border: 1px solid #eef2f7;
  border-radius: 12px;
  padding: 18px 20px 4px;
  margin-bottom: 14px;
}

.section-title {
  font-size: 13px;
  font-weight: 600;
  color: #334155;
  margin-bottom: 14px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.section-hint {
  font-size: 11px;
  font-weight: 400;
  color: #94a3b8;
}

.field-hint {
  font-size: 11px;
  color: #94a3b8;
  margin-top: 4px;
  line-height: 1.4;

  &.warn { color: #d97706; }
}

.switch-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 0;
  border-top: 1px solid #f1f5f9;

  &:first-of-type { border-top: none; }
}

.switch-label {
  font-size: 13px;
  color: #334155;
  font-weight: 500;
}

.switch-desc {
  font-size: 11px;
  color: #94a3b8;
  margin-top: 2px;
}

/* 深色模式 */
html.dark .form-section {
  background: #1e293b;
  border-color: #334155;
}
html.dark .section-title { color: #cbd5e1; }
html.dark .section-hint { color: #64748b; }
html.dark .field-hint { color: #64748b; }
html.dark .switch-row { border-top-color: #334155; }
html.dark .switch-label { color: #cbd5e1; }
html.dark .switch-desc { color: #64748b; }
</style>
