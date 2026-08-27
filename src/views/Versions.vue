<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useAppStore } from '@/stores/app';
import { message, Modal } from 'ant-design-vue';
import { ReloadOutlined, DownloadOutlined, DeleteOutlined, ImportOutlined, GlobalOutlined, CheckCircleOutlined } from '@ant-design/icons-vue';

const appStore = useAppStore();
const loading = ref(false);
const downloading = ref<string | null>(null);
const mirrorsVisible = ref(false);
const selectedMirror = ref('github');

const columns = [
  { title: '版本', dataIndex: 'version', key: 'version', ellipsis: true, width: 120 },
  { title: '发布日期', dataIndex: 'published_at', key: 'published_at', width: 120,
    customRender: ({ text }: { text: string }) => text ? new Date(text).toLocaleDateString() : '-' },
  { title: '大小', key: 'size', width: 100,
    customRender: ({ record }: { record: any }) => record.size ? (record.size / 1048576).toFixed(1) + ' MB' : '-' },
  { title: '状态', key: 'downloaded', width: 90 },
  { title: '操作', key: 'action', width: 220, fixed: 'right' as const },
];

async function refresh() {
  loading.value = true;
  await appStore.loadVersions();
  loading.value = false;
}

async function download(record: any) {
  downloading.value = record.version;
  // 始终传原始 GitHub URL，镜像回退链由后端 build_download_candidates 统一构造。
  // 历史 bug：此前优先用 mirror_url（已含镜像前缀），后端再拼一层前缀
  // → 生成 ghproxy.net/ghproxy.net/原始 这种坏 URL → 下载全失败。
  let url = record.download_url;
  
  message.loading(`正在下载 ${record.version}...`, 0);
  
  try {
    const result = await appStore.downloadVersion(record.version, url);
    message.destroy();
    if (result.success) {
      message.success('下载成功！');
      await refresh();
    } else {
      message.error(`下载失败：${result.error}`);
      if (url.includes('github.com') && result.error?.includes('404')) {
        Modal.confirm({
          title: '下载失败',
          content: 'GitHub 下载失败，是否尝试使用镜像源？',
          okText: '使用镜像下载',
          cancelText: '取消',
          onOk: () => { mirrorsVisible.value = true; }
        });
      }
    }
  } catch (e: any) {
    message.destroy();
    message.error(`下载失败：${e.message || '网络错误'}`);
  }
  
  downloading.value = null;
}

async function remove(record: any) {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除版本 "${record.version}" 吗？`,
    onOk: async () => {
      const result = await appStore.deleteVersion(record.version);
      if (result.success) {
        message.success('已删除');
        await refresh();
      } else {
        message.error(`删除失败：${result.error}`);
      }
    }
  });
}

async function importLocal() {
  const result = await appStore.importLocalFrpc();
  if (result.success) {
    message.success('导入成功！');
    await refresh();
  } else if (result.error !== 'canceled') {
    message.error(`导入失败：${result.error}`);
  }
}

/// 切换当前使用的 frpc 版本
async function useVersion(record: any) {
  try {
    const result = await appStore.setActiveVersion(record.version);
    if (result.success) {
      message.success(`已切换为使用 ${record.version}，启动 FRP 时将使用此版本`);
      await refresh();
    } else {
      message.error(`切换失败：${result.error}`);
    }
  } catch (e) {
    message.error(`切换失败：${String(e)}`);
  }
}

function handleMirrorDownload(record: any) {
  const mirror = appStore.mirrors.find((m: any) => m.id === selectedMirror.value);
  let url = record.download_url;
  if (mirror && mirror.prefix) {
    url = mirror.prefix + url;
  }
  download({ ...record, mirror_url: url });
  mirrorsVisible.value = false;
}

onMounted(() => { refresh(); });
</script>

<template>
  <div class="page-container">
    <div class="page-header">
      <h1 class="page-title">版本管理</h1>
      <a-space>
        <a-button @click="mirrorsVisible = true">
          <template #icon><GlobalOutlined /></template>
          镜像源
        </a-button>
        <a-button @click="importLocal">
          <template #icon><ImportOutlined /></template>
          导入本地
        </a-button>
        <a-button @click="refresh" :loading="loading">
          <template #icon><ReloadOutlined /></template>
          刷新
        </a-button>
      </a-space>
    </div>

    <a-card class="content-card">
      <a-alert
        v-if="appStore.versions.length === 0 && !loading"
        message="暂无版本数据"
        description="点击右上角刷新按钮从 GitHub 获取最新版本列表"
        type="info"
        show-icon
        style="margin-bottom: 16px"
      />
      
      <a-table
        v-else
        :data-source="appStore.versions"
        :columns="columns"
        row-key="version"
        :pagination="{ pageSize: 10, showSizeChanger: true, showQuickJumper: true }"
        :loading="loading"
        :scroll="{ x: 620 }"
        size="middle"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'version'">
            <span>{{ record.version }}</span>
            <a-tag v-if="record.is_active" color="processing" style="margin-left: 8px">
              使用中
            </a-tag>
          </template>
          <template v-if="column.key === 'downloaded'">
            <a-tag :color="record.downloaded ? 'green' : 'default'">
              {{ record.downloaded ? '已下载' : '未下载' }}
            </a-tag>
          </template>
          <template v-if="column.key === 'action'">
            <a-space>
              <a-button
                v-if="!record.downloaded"
                size="small"
                type="primary"
                :loading="downloading === record.version"
                @click="download(record)"
              >
                <template #icon><DownloadOutlined /></template>
                下载
              </a-button>
              <a-button
                v-if="record.downloaded && !record.is_active"
                size="small"
                type="primary"
                ghost
                @click="useVersion(record)"
              >
                <CheckCircleOutlined />
                使用此版本
              </a-button>
              <a-button
                v-if="record.downloaded"
                size="small"
                danger
                @click="remove(record)"
              >
                <template #icon><DeleteOutlined /></template>
                删除
              </a-button>
            </a-space>
          </template>
        </template>
      </a-table>
    </a-card>

    <a-modal
      v-model:open="mirrorsVisible"
      title="选择下载镜像源"
      width="500px"
      :footer="null"
    >
      <p style="margin-bottom: 16px; color: #64748b;">
        如果 GitHub 下载失败，可以选择镜像源加速下载：
      </p>
      <a-radio-group v-model:value="selectedMirror" style="width: 100%">
        <a-space direction="vertical" style="width: 100%">
          <a-radio
            v-for="mirror in appStore.mirrors"
            :key="mirror.id"
            :value="mirror.id"
            class="mirror-radio"
          >
            <strong>{{ mirror.name }}</strong>
            <div v-if="mirror.prefix" class="mirror-url">
              {{ mirror.prefix }}
            </div>
          </a-radio>
        </a-space>
      </a-radio-group>
      <div style="margin-top: 24px; text-align: right">
        <a-button @click="mirrorsVisible = false">取消</a-button>
        <a-button type="primary" @click="handleMirrorDownload(appStore.versions.find(v => !v.downloaded) || appStore.versions[0])">
          使用此镜像下载
        </a-button>
      </div>
    </a-modal>
  </div>
</template>

<style scoped lang="scss">
.page-container {
  padding: 24px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;

  .page-title {
    font-size: 24px;
    font-weight: 700;
    color: #1e293b;
    margin: 0;
  }

  .ant-btn {
    border-radius: 8px;
    font-weight: 500;
    transition: all 0.2s ease;

    &:hover {
      transform: translateY(-1px);
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
    }

    &:active {
      transform: scale(0.98);
    }
  }
}

.content-card {
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);

  :deep(.ant-table) {
    .ant-table-thead > tr > th {
      background: #f8fafc;
      font-weight: 600;
      color: #475569;
      border-radius: 6px;
    }

    .ant-table-tbody > tr:hover > td {
      background: #f8fafc;
    }
  }

  .ant-btn {
    border-radius: 6px;
    font-weight: 500;
    transition: all 0.2s ease;

    &:hover {
      transform: translateY(-1px);
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
    }

    &:active {
      transform: scale(0.98);
    }
  }
}

.mirror-radio {
  display: block !important;
  padding: 12px 16px !important;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  margin-bottom: 8px;
  transition: all 0.2s ease;

  &:hover {
    border-color: #2563eb;
    background: #f8fafc;
  }

  :deep(.ant-radio) {
    margin-right: 12px;
  }

  .mirror-url {
    color: #94a3b8;
    font-size: 12px;
    margin-top: 4px;
    font-family: monospace;
  }
}

.ant-modal {
  .ant-modal-content {
    border-radius: 12px;
  }

  .ant-modal-header {
    border-radius: 12px 12px 0 0;
    font-weight: 600;
  }

  .ant-btn-primary {
    border-radius: 8px;

    &:active {
      transform: scale(0.98);
    }
  }
}
</style>
