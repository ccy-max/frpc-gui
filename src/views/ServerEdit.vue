<script setup lang="ts">
/**
 * 服务器编辑页（替代原弹窗）
 *
 * 通过 props 接收编辑目标，由父页面在列表视图内切换显示。
 * 不新增路由，避免列表筛选/滚动位置丢失。
 */
import { ref, computed, watch, onMounted } from 'vue';
import { useAppStore } from '@/stores/app';
import { message } from 'ant-design-vue';
import type { FormInstance, Rule } from 'ant-design-vue/es/form';
import SubPageHeader from '@/components/SubPageHeader.vue';
import {
  SaveOutlined, UndoOutlined,
} from '@ant-design/icons-vue';

const props = defineProps<{
  /** 传入则为编辑，null 为新建 */
  server: any | null;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'saved'): void;
}>();

const appStore = useAppStore();
const formRef = ref<FormInstance>();
const saving = ref(false);

const isEdit = computed(() => !!props.server?.id);
const editingId = computed(() => props.server?.id ?? null);

const form = ref<any>({
  id: '', name: '', serverAddr: '', serverPort: 7000,
  token: '', tlsEnable: false, enabled: true, user: '', metaToken: '',
});

function resetForm() {
  const s = props.server;
  form.value = {
    id: s?.id ?? '',
    name: s?.name ?? '',
    serverAddr: s?.serverAddr ?? '',
    serverPort: s?.serverPort ?? 7000,
    token: s?.token ?? '',
    tlsEnable: s?.tlsEnable ?? false,
    enabled: s?.enabled ?? true,
    user: s?.user ?? '',
    metaToken: s?.metaToken ?? '',
  };
}

watch(() => props.server, resetForm, { immediate: true });

// A2 表单校验
const rules: Record<string, Rule[]> = {
  name: [
    { required: true, message: '请输入服务器名称', trigger: 'blur' },
    {
      validator: async (_r: Rule, value: string) => {
        const dup = appStore.servers.some(s => s.name === value && s.id !== editingId.value);
        if (dup) throw new Error('该名称已存在');
      },
      trigger: 'blur',
    },
  ],
  serverAddr: [
    { required: true, message: '请输入服务器地址', trigger: 'blur' },
  ],
  serverPort: [
    { required: true, message: '请输入端口', trigger: 'change' },
    {
      validator: async (_r: Rule, value: number) => {
        if (!value || value < 1 || value > 65535) throw new Error('端口范围 1-65535');
      },
      trigger: 'change',
    },
  ],
};

async function handleSave() {
  try {
    await formRef.value?.validate();
  } catch {
    message.warning('请检查表单填写');
    return;
  }
  saving.value = true;
  try {
    const payload = { ...form.value };
    if (editingId.value) {
      await appStore.updateServer(editingId.value, payload);
      message.success('服务器已更新');
    } else {
      payload.id = `srv-${Date.now()}`;
      await appStore.addServer(payload);
      message.success('服务器已添加');
    }
    emit('saved');
  } catch (e) {
    message.error(`保存失败：${String(e)}`);
  } finally {
    saving.value = false;
  }
}

onMounted(resetForm);
</script>

<template>
  <SubPageHeader
    :title="isEdit ? '编辑服务器' : '添加服务器'"
    :subtitle="isEdit ? `修改「${server?.name}」的连接配置` : '填写 frps 服务器地址、端口与令牌'"
    back-text="返回服务器列表"
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
        {{ isEdit ? '保存修改' : '添加服务器' }}
      </a-button>
    </template>

    <div class="form-wrap">
    <a-form ref="formRef" :model="form" :rules="rules" layout="vertical">
      <div class="form-section">
        <div class="section-title">基础信息</div>
        <a-form-item label="名称" name="name">
          <a-input v-model:value="form.name" placeholder="例如：阿里云香港" />
        </a-form-item>
        <a-row :gutter="12">
          <a-col :span="16">
            <a-form-item label="服务器地址" name="serverAddr">
              <a-input v-model:value="form.serverAddr" placeholder="127.0.0.1 或 frp.example.com" />
            </a-form-item>
          </a-col>
          <a-col :span="8">
            <a-form-item label="端口" name="serverPort">
              <a-input-number v-model:value="form.serverPort" :min="1" :max="65535" style="width: 100%" />
            </a-form-item>
          </a-col>
        </a-row>
        <a-form-item label="令牌">
          <a-input-password v-model:value="form.token" placeholder="frps 配置的 token，留空表示无鉴权" />
        </a-form-item>
      </div>

      <div class="form-section">
        <div class="section-title">
          多用户鉴权
          <span class="section-hint">可选 · frps 启用 multiuser 插件时使用</span>
        </div>
        <a-row :gutter="12">
          <a-col :span="12">
            <a-form-item label="用户名 user">
              <a-input v-model:value="form.user" placeholder="留空则不启用" />
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="meta_token">
              <a-input-password v-model:value="form.metaToken" placeholder="与 user 配套使用" />
            </a-form-item>
          </a-col>
        </a-row>
      </div>

      <div class="form-section">
        <div class="section-title">连接选项</div>
        <div class="switch-row">
          <div class="switch-text">
            <div class="switch-label">启用 TLS</div>
            <div class="switch-desc">与 frps 之间的连接使用 TLS 加密</div>
          </div>
          <a-switch v-model:checked="form.tlsEnable" />
        </div>
        <div class="switch-row">
          <div class="switch-text">
            <div class="switch-label">启用此服务器</div>
            <div class="switch-desc">关闭后该服务器不会参与启动</div>
          </div>
          <a-switch v-model:checked="form.enabled" />
        </div>
      </div>
      </a-form>
    </div>
  </SubPageHeader>
</template>

<style scoped>
.form-wrap {
  max-width: 640px;
}

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
html.dark .switch-row { border-top-color: #334155; }
html.dark .switch-label { color: #cbd5e1; }
html.dark .switch-desc { color: #64748b; }
</style>
