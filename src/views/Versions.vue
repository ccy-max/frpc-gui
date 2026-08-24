<script setup lang="ts">
import { useAppStore } from '@/stores/app';

const appStore = useAppStore();
</script>

<template>
  <div class="versions-page">
    <h2 class="page-title">版本管理</h2>
    
    <el-card>
      <el-empty description="暂无 FRP 版本，请先下载" />
      
      <div class="download-section">
        <h3>推荐版本</h3>
        <el-table :data="[{version: 'v0.52.0', platform: '全部', downloaded: false}]" style="margin-top: 16px">
          <el-table-column prop="version" label="版本" />
          <el-table-column prop="platform" label="平台" />
          <el-table-column label="状态" width="100">
            <template #default>
              <el-tag type="info">未下载</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="150">
            <template #default>
              <el-button size="small" type="primary">下载</el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>
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
</style>
