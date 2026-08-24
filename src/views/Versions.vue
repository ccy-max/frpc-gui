<script setup lang="ts">
import { useAppStore } from '@/stores/app';
import { useI18n } from 'vue-i18n';

const appStore = useAppStore();
const { t } = useI18n();

function checkUpdate() {
  // TODO: 检查更新
}
</script>

<template>
  <div class="versions-page">
    <h2 class="page-title">{{ t('version.title') }}</h2>
    
    <el-card>
      <div class="version-header">
        <span>{{ t('version.currentVersion') }}: v0.11.0</span>
        <el-button type="primary" size="small" @click="checkUpdate">
          {{ t('version.checkUpdate') }}
        </el-button>
      </div>
      
      <el-table :data="appStore.versions" style="margin-top: 16px">
        <el-table-column prop="version" :label="t('version.title')" />
        <el-table-column prop="platform" label="平台" />
        <el-table-column prop="downloaded" label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="row.downloaded ? 'success' : 'info'" size="small">
              {{ row.downloaded ? t('version.downloaded') : t('version.download') }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="150">
          <template #default="{ row }">
            <el-button size="small" type="primary" v-if="!row.downloaded">
              {{ t('version.download') }}
            </el-button>
            <el-button size="small" type="danger" v-else>
              {{ t('version.deleteVersion') }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>
      
      <el-empty v-if="appStore.versions.length === 0" description="暂无 FRP 版本" />
    </el-card>
  </div>
</template>

<style scoped lang="scss">
.versions-page {
  padding: 24px;
  height: calc(100vh - 60px);
  overflow-y: auto;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 24px;
}

.version-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>
