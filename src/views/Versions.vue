<script setup lang="ts">
import { useAppStore } from '@/stores/app';
</script>

<template>
  <div class="versions-page">
    <h2 class="page-title">版本管理</h2>
    <a-card>
      <a-empty description="暂无 FRP 版本，请先下载" />
      <div style="margin-top: 16px">
        <h3>推荐版本</h3>
        <a-table
          :data-source="[{ key: '1', version: 'v0.52.0', platform: '全部', downloaded: false }]"
          :columns="[
            { title: '版本', dataIndex: 'version' },
            { title: '平台', dataIndex: 'platform' },
            { title: '状态', key: 'downloaded' },
            { title: '操作', key: 'action' },
          ]"
          :pagination="false"
          size="small"
        >
          <template #bodyCell="{ column, record }">
            <template v-if="column.key === 'downloaded'">
              <a-tag color="default">未下载</a-tag>
            </template>
            <template v-if="column.key === 'action'">
              <a-button size="small" type="primary">下载</a-button>
            </template>
          </template>
        </a-table>
      </div>
    </a-card>
  </div>
</template>

<style scoped lang="scss">
.versions-page { padding: 24px; height: calc(100vh - 60px); overflow-y: auto; }
.page-title { font-size: 24px; font-weight: 600; margin-bottom: 24px; }
</style>
