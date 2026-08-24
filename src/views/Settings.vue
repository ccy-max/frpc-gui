<script setup lang="ts">
import { useAppStore } from '@/stores/app';
import { useI18n } from 'vue-i18n';
import { Folder } from '@element-plus/icons-vue';

const appStore = useAppStore();
const { t } = useI18n();

function handleConfigChange() {
  appStore.saveConfig();
}
</script>

<template>
  <div class="settings-page">
    <h2 class="page-title">{{ t('settings.title') }}</h2>

    <el-row :gutter="16">
      <el-col :span="12">
        <el-card>
          <template #header>
            <span>{{ t('settings.general') }}</span>
          </template>
          
          <el-form label-width="140px">
            <el-form-item :label="t('settings.language')">
              <el-select v-model="appStore.config.language" @change="handleConfigChange" style="width: 100%">
                <el-option label="简体中文" value="zh-CN" />
                <el-option label="English" value="en-US" />
              </el-select>
            </el-form-item>

            <el-form-item :label="t('settings.theme')">
              <el-select v-model="appStore.config.theme" @change="handleConfigChange" style="width: 100%">
                <el-option :label="t('settings.light')" value="light" />
                <el-option :label="t('settings.dark')" value="dark" />
                <el-option :label="t('settings.auto')" value="auto" />
              </el-select>
            </el-form-item>

            <el-form-item :label="t('settings.startup')">
              <el-switch v-model="appStore.config.autoStart" @change="handleConfigChange" />
            </el-form-item>

            <el-form-item :label="t('settings.minimizeToTray')">
              <el-switch v-model="appStore.config.minimizeToTray" />
            </el-form-item>

            <el-form-item :label="t('settings.closeToTray')">
              <el-switch v-model="appStore.config.closeToTray" />
            </el-form-item>

            <el-form-item :label="t('settings.checkUpdateOnStart')">
              <el-switch v-model="appStore.config.checkUpdateOnStart" />
            </el-form-item>
          </el-form>
        </el-card>
      </el-col>

      <el-col :span="12">
        <el-card>
          <template #header>
            <span>{{ t('settings.advanced') }}</span>
          </template>
          
          <el-form label-width="140px">
            <el-form-item :label="t('settings.frpPath')">
              <el-input v-model="appStore.config.frpBinaryPath" readonly>
                <template #append>
                  <el-button :icon="Folder">
                    {{ t('settings.browse') }}
                  </el-button>
                </template>
              </el-input>
            </el-form-item>

            <el-form-item :label="t('settings.configPath')">
              <el-input v-model="appStore.config.configPath" readonly>
                <template #append>
                  <el-button :icon="Folder">
                    {{ t('settings.browse') }}
                  </el-button>
                </template>
              </el-input>
            </el-form-item>

            <el-form-item :label="t('settings.logPath')">
              <el-input v-model="appStore.config.logPath" readonly>
                <template #append>
                  <el-button :icon="Folder">
                    {{ t('settings.browse') }}
                  </el-button>
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
