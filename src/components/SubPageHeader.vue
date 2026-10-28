<script setup lang="ts">
/**
 * 页内子页面头部（替代弹窗）
 *
 * 统一"返回 + 标题 + 副标题 + 右侧操作"布局，
 * 供所有"新建/编辑/详情"页内页面复用。
 */
import { useRouter } from 'vue-router';
import { ArrowLeftOutlined } from '@ant-design/icons-vue';

const props = withDefaults(defineProps<{
  title: string;
  subtitle?: string;
  /** 返回目标路径；留空则 router.back() */
  backTo?: string;
  /** 返回按钮文案 */
  backText?: string;
  /**
   * 是否由组件自己执行返回跳转。
   * 设为 false 时只派发 back 事件，完全交给父组件
   * （用于列表页内切换视图，避免路由跳转丢掉筛选状态）。
   */
  autoBack?: boolean;
}>(), {
  subtitle: '',
  backTo: '',
  backText: '返回',
  autoBack: true,
});

const emit = defineEmits<{
  (e: 'back'): void;
}>();

const router = useRouter();

function goBack() {
  emit('back');
  if (!props.autoBack) return;
  if (props.backTo) {
    router.push(props.backTo);
  } else if (window.history.length > 1) {
    router.back();
  } else {
    router.push('/');
  }
}
</script>

<template>
  <div class="subpage">
    <div class="subpage-head">
      <a-button type="text" class="back-btn" @click="goBack">
        <template #icon><ArrowLeftOutlined /></template>
        {{ backText }}
      </a-button>
      <div class="head-divider"></div>
      <div class="head-titles">
        <h1 class="head-title">{{ title }}</h1>
        <p v-if="subtitle" class="head-subtitle">{{ subtitle }}</p>
      </div>
      <div class="head-actions">
        <slot name="actions" />
      </div>
    </div>

    <div class="subpage-body">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.subpage {
  padding: 20px 24px 32px;
  min-height: 100%;
}

.subpage-head {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-bottom: 20px;
  padding-bottom: 14px;
  border-bottom: 1px solid rgba(15, 23, 42, 0.06);
}

.back-btn {
  font-size: 13px;
  color: #64748b;
  padding: 0 10px 0 6px;
  height: 30px;
  border-radius: 7px;
  flex-shrink: 0;

  &:hover {
    color: #2563eb;
    background: rgba(37, 99, 235, 0.08);
  }
}

.head-divider {
  width: 1px;
  height: 16px;
  background: #e2e8f0;
  margin: 0 10px 0 4px;
  flex-shrink: 0;
}

.head-titles {
  min-width: 0;
  flex: 1;
}

.head-title {
  font-size: 18px;
  font-weight: 600;
  color: #1e293b;
  margin: 0;
  line-height: 1.35;
}

.head-subtitle {
  font-size: 12px;
  color: #94a3b8;
  margin: 2px 0 0;
  line-height: 1.4;
}

.head-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.subpage-body {
  max-width: 720px;
}

/* 深色模式 */
html.dark .subpage-head {
  border-bottom-color: rgba(255, 255, 255, 0.07);
}
html.dark .head-title { color: #e2e8f0; }
html.dark .head-subtitle { color: #64748b; }
html.dark .head-divider { background: #334155; }
html.dark .back-btn {
  color: #94a3b8;
  &:hover { color: #60a5fa; background: rgba(96, 165, 250, 0.12); }
}
</style>
