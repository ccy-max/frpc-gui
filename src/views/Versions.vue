<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useAppStore } from '@/stores/app';
import { message } from 'ant-design-vue';
import { ReloadOutlined, DownloadOutlined, DeleteOutlined } from '@ant-design/icons-vue';
import { h } from 'vue';

const appStore = useAppStore();
const loading = ref(false);
const downloading = ref<string | null>(null);

const columns = [
  { title: '版本', dataIndex: 'version', key: 'version' },
  { title: '发布日期', dataIndex: 'published_at', key: 'published_at',
    customRender: ({ text }: { text: string }) => text ? new Date(text).toLocaleDateString() : '-' },
  { title: '大小', key: 'size',
    customRender: ({ record }: { record: any }) => record.size ? (record.size / 1048576).toFixed(1) + ' MB' : '-' },
  { title: '状态', key: 'downloaded', width: 100 },
  { title: '操作', key: 'action', width: 150 },
];

async function refresh() {
  loading.value = true;
  await appStore.loadVersions();
  loading.value = false;
}

async function download(record: any) {
  downloading.value = record.version;
  message.loading(`正在下载 ${record.version}...`, 0);
  const result = await appStore.downloadVersion(record.version, record.download_url);
  message.destroy();
  if (result.success) {
    message.success(`下载成功！frpc 路径：${result.path}`);
  } else {
    message.error(`下载失败：${result.error}`);
  }
  downloading.value = null;
}

async function remove(record: any) {
  const result = await appStore.deleteVersion(record.version);
  if (result.success) {
    message.success('已删除');
  } else {
    message.error(`删除失败：${result.error}`);
  }
}

onMounted(() => {
  refresh();
});
</script>

<template>
  <div class="versions-page">
    <div class="page-header">
      <h2 class="page-title">版本管理</h2>
      <a-button :icon="h(ReloadOutlined)" @click="refresh" :loading="loading">刷新</a-button>
    </div>

    <a-card>
      <a-empty v-if="!loading && appStore.versions.length === 0" description="点击右上角刷新获取版本列表" />
      <a-table
        v-else
        :data-source="appStore.versions"
        :columns="columns"
        row-key="version"
        :pagination="{ pageSize: 10 }"
        :loading="loading"
        size="middle"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'downloaded'">
            <a-tag :color="record.downloaded ? 'green' : 'default'">
              {{ record.downloaded ? '已下载' : '未下载' }}
            </a-tag>
          </template>
          <template v-if="column.key === 'action'">
            <a-button
              v-if="!record.downloaded"
              size="small"
              type="primary"
              :icon="h(DownloadOutlined)"
              :loading="downloading === record.version"
              @click="download(record)"
            >
              下载
            </a-button>
            <a-button
              v-else
              size="small"
              danger
              :icon="h(DeleteOutlined)"
              @click="remove(record)"
            >
              删除
            </a-button>
          </template>
        </template>
      </a-table>
    </a-card>
  </div>
</template>

<style scoped lang="scss">
.versions-page { padding: 24px; height: calc(100vh - 60px); overflow-y: auto; }
.page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; }
.page-title { font-size: 24px; font-weight: 600; }
</style>
