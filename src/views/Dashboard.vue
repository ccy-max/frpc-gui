<script setup lang="ts">
import { computed } from 'vue';
import { useAppStore } from '@/stores/app';
import { useI18n } from 'vue-i18n';
import { Plus, VideoPlay, Document, Setting } from '@element-plus/icons-vue';

const appStore = useAppStore();
const { t } = useI18n();

const stats = computed(() => [
  {
    title: '运行中的服务器',
    value: appStore.runningServersCount,
    icon: 'Server',
    color: 'var(--el-color-success)',
  },
  {
    title: '代理总数',
    value: appStore.proxies.length,
    icon: 'Connection',
    color: 'var(--el-color-primary)',
  },
  {
    title: '活跃代理',
    value: appStore.activeProxiesCount,
    icon: 'CircleCheck',
    color: 'var(--el-color-warning)',
  },
  {
    title: 'FRP 版本',
    value: appStore.versions.length,
    icon: 'Download',
    color: 'var(--el-color-info)',
  },
]);

const recentLogs = computed(() => 
  appStore.logs.slice(-5).reverse()
);
</script>

<template>
  <div class="dashboard">
    <h2 class="page-title">概览</h2>
    
    <!-- 统计卡片 -->
    <el-row :gutter="16" class="stats-row">
      <el-col :span="6" v-for="(stat, index) in stats" :key="index">
        <el-card class="stat-card" shadow="hover">
          <div class="stat-content">
            <div class="stat-icon" :style="{ backgroundColor: stat.color + '20' }">
              <el-icon :size="24" :color="stat.color">
                <component :is="stat.icon" />
              </el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-value">{{ stat.value }}</div>
              <div class="stat-label">{{ stat.title }}</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 快速操作 -->
    <el-card class="quick-start-card" shadow="hover">
      <template #header>
        <div class="card-header">
          <span>快速启动</span>
        </div>
      </template>
      <div class="quick-actions">
        <el-button type="primary" :icon="Plus">
          添加服务器
        </el-button>
        <el-button type="success" :icon="VideoPlay" @click="appStore.startFRP()">
          启动 FRP
        </el-button>
        <el-button type="warning" :icon="Document" @click="$router.push('/logs')">
          查看日志
        </el-button>
        <el-button type="info" :icon="Setting" @click="$router.push('/settings')">
          设置
        </el-button>
      </div>
    </el-card>

    <!-- 最近日志 -->
    <el-card class="logs-card" shadow="hover">
      <template #header>
        <div class="card-header flex-between">
          <span>最近日志</span>
          <el-button text type="primary" size="small" @click="$router.push('/logs')">
            查看全部
          </el-button>
        </div>
      </template>
      <el-empty v-if="recentLogs.length === 0" description="暂无日志" />
      <el-table v-else :data="recentLogs" :height="200" size="small">
        <el-table-column prop="timestamp" label="时间" width="180">
          <template #default="{ row }">
            {{ new Date(row.timestamp).toLocaleString() }}
          </template>
        </el-table-column>
        <el-table-column prop="level" label="级别" width="80">
          <template #default="{ row }">
            <el-tag :type="row.level === 'error' ? 'danger' : row.level === 'warn' ? 'warning' : 'info'" size="small">
              {{ row.level.toUpperCase() }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="message" label="消息" />
      </el-table>
    </el-card>
  </div>
</template>

<style scoped lang="scss">
.dashboard {
  padding: 24px;
  height: calc(100vh - 60px);
  overflow-y: auto;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin-bottom: 24px;
}

.stats-row {
  margin-bottom: 24px;
}

.stat-card {
  :deep(.el-card__body) {
    padding: 20px;
  }
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 16px;
}

.stat-icon {
  width: 56px;
  height: 56px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.stat-info {
  flex: 1;
}

.stat-value {
  font-size: 28px;
  font-weight: 700;
  color: var(--el-text-color-primary);
  line-height: 1;
  margin-bottom: 4px;
}

.stat-label {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.quick-start-card,
.logs-card {
  margin-bottom: 16px;
}

.card-header {
  font-weight: 600;
  font-size: 16px;
}

.quick-actions {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}
</style>
