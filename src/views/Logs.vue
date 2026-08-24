<script setup lang="ts">
import { computed } from 'vue';
import { useAppStore } from '@/stores/app';
import { useI18n } from 'vue-i18n';
import { Refresh, Download, Delete } from '@element-plus/icons-vue';

const appStore = useAppStore();
const { t } = useI18n();

const logs = computed(() => appStore.logs.slice().reverse());

function clearLogs() {
  appStore.clearLogs();
}

function exportLogs() {
  // TODO: 导出日志
}
</script>

<template>
  <div class="logs-page">
    <div class="page-header">
      <h2 class="page-title">{{ t('log.title') }}</h2>
      <div class="actions">
        <el-button :icon="Refresh" @click="() => {}">
          {{ t('common.refresh') }}
        </el-button>
        <el-button :icon="Download" @click="exportLogs">
          {{ t('log.export') }}
        </el-button>
        <el-button type="danger" :icon="Delete" @click="clearLogs">
          {{ t('log.clear') }}
        </el-button>
      </div>
    </div>

    <el-card>
      <el-table :data="logs" :height="600" size="small">
        <el-table-column prop="timestamp" :label="t('log.timestamp')" width="180">
          <template #default="{ row }">
            {{ new Date(row.timestamp).toLocaleString() }}
          </template>
        </el-table-column>
        <el-table-column prop="level" :label="t('log.level')" width="80">
          <template #default="{ row }">
            <el-tag :type="row.level === 'error' ? 'danger' : row.level === 'warn' ? 'warning' : 'info'" size="small">
              {{ row.level.toUpperCase() }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="source" :label="t('log.source')" width="80">
          <template #default="{ row }">
            <el-tag size="small" type="info">{{ row.source }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="message" :label="t('log.message')" />
      </el-table>
      
      <el-empty v-if="logs.length === 0" :description="t('log.title')" />
    </el-card>
  </div>
</template>

<style scoped lang="scss">
.logs-page {
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

.actions {
  display: flex;
  gap: 8px;
}
</style>
