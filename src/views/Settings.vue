<script setup lang="ts">
import { useAppStore } from '@/stores/app';
import { Folder } from '@element-plus/icons-vue';

const appStore = useAppStore();

function handleConfigChange() {
  appStore.saveConfig(appStore.frpConfig || {} as any);
}
</script>

<template>
  <div class="settings-page">
    <h2 class="page-title">设置</h2>

    <el-row :gutter="16">
      <el-col :span="12">
        <el-card>
          <template #header>
            <span>通用设置</span>
          </template>
          
          <el-form label-width="120px">
            <el-form-item label="语言">
              <el-select v-model="appStore.language" @change="handleConfigChange" style="width: 100%">
                <el-option label="简体中文" value="zh-CN" />
                <el-option label="English" value="en-US" />
              </el-select>
            </el-form-item>

            <el-form-item label="主题">
              <el-select v-model="appStore.theme" @change="handleConfigChange" style="width: 100%">
                <el-option label="浅色" value="light" />
                <el-option label="深色" value="dark" />
                <el-option label="跟随系统" value="auto" />
              </el-select>
            </el-form-item>

            <el-form-item label="开机自启">
              <el-switch v-model="appStore.processStatus.running" />
            </el-form-item>
          </el-form>
        </el-card>
      </el-col>

      <el-col :span="12">
        <el-card>
          <template #header>
            <span>路径设置</span>
          </template>
          
          <el-form label-width="120px">
            <el-form-item label="FRP 路径">
              <el-input readonly placeholder="请选择 frpc 可执行文件路径">
                <template #append>
                  <el-button :icon="Folder">浏览</el-button>
                </template>
              </el-input>
            </el-form-item>

            <el-form-item label="配置路径">
              <el-input readonly placeholder="frpc.toml 保存路径">
                <template #append>
                  <el-button :icon="Folder">浏览</el-button>
                </template>
              </el-input>
            </el-form-item>
          </el-form>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<style scoped lang="scss">
.settings-page {
  padding: 24px;
  height: calc(100vh - 60px);
  overflow-y: auto;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 24px;
}

.el-card {
  margin-bottom: 16px;
}
</style>
