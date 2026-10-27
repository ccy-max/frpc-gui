<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { useAppStore } from '@/stores/app';
import { PlusOutlined, AppstoreOutlined, GlobalOutlined, ThunderboltOutlined, CheckCircleOutlined, CloseCircleOutlined, ArrowUpOutlined, ArrowDownOutlined, MoreOutlined, CopyOutlined, BarsOutlined } from '@ant-design/icons-vue';
import { message, Modal } from 'ant-design-vue';

const appStore = useAppStore();
// D4：搜索/筛选状态持久化（切页回来不重置）
const searchKeyword = ref(localStorage.getItem('proxySearch') || '');
const serverFilter = ref<string>(localStorage.getItem('proxyServerFilter') || '');
watch(searchKeyword, v => localStorage.setItem('proxySearch', v));
watch(serverFilter, v => localStorage.setItem('proxyServerFilter', v));
const modalVisible = ref(false);
const editingName = ref<string | null>(null);
const selectedProxies = ref<Set<string>>(new Set()); // 批量选择
const batchActionVisible = ref(false); // 批量操作工具栏

const form = ref<any>({
  name: '', type: 'tcp', local_ip: '127.0.0.1', local_port: 8080,
  remote_port: 8080, server_id: '', enabled: true,
  bandwidth_limit: '', traffic_limit: 0,
  custom_domains: [] as string[], subdomain: '', locations: [] as string[],
  http_user: '', http_password: '', host_header_rewrite: '',
  secret_key: '', allow_users: [] as string[],
  health_check_type: undefined as 'tcp' | 'http' | undefined,
  health_check_interval_s: 10, health_check_timeout_s: 3,
  health_check_max_unhealthy_times: 3, health_check_path: '/health',
  use_encryption: false, use_compression: false,
});

const proxyTypes = [
  { value: 'tcp', label: 'TCP' }, { value: 'udp', label: 'UDP' },
  { value: 'http', label: 'HTTP' }, { value: 'https', label: 'HTTPS' },
  { value: 'stcp', label: 'STCP' }, { value: 'xtcp', label: 'XTCP' },
  { value: 'sudp', label: 'SUDP' }, { value: 'tcpmux', label: 'TCPMUX' },
];

// 获取代理类型对应的颜色
function getProxyTypeColor(type: string): string {
  const colors: Record<string, string> = {
    tcp: 'blue',
    udp: 'cyan',
    http: 'green',
    https: 'purple',
    stcp: 'orange',
    xtcp: 'volcano',
    sudp: 'lime',
    tcpmux: 'pink',
  };
  return colors[type.toLowerCase()] || 'default';
}

// 获取代理类型对应的图标
function getProxyTypeIcon(type: string): string {
  if (['http', 'https'].includes(type)) return 'global';
  if (['stcp', 'xtcp', 'sudp'].includes(type)) return 'thunderbolt';
  return 'appstore';
}

const filteredProxies = computed(() => {
  let result = appStore.proxies;
  
  // 按服务器过滤
  if (serverFilter.value) {
    result = result.filter(p => p.server_id === serverFilter.value);
  }
  
  // 按关键词搜索
  if (searchKeyword.value.trim()) {
    const kw = searchKeyword.value.toLowerCase();
    result = result.filter(p =>
      p.name.toLowerCase().includes(kw) || 
      p.type.toLowerCase().includes(kw)
    );
  }
  
  return result;
});

// A3：渐进渲染——DOM 只挂前 visibleCount 张卡片，批量选择仍作用于全量数据
const PAGE_SIZE = 24;
const visibleCount = ref(PAGE_SIZE);
const visibleProxies = computed(() => filteredProxies.value.slice(0, visibleCount.value));
watch([searchKeyword, serverFilter], () => { visibleCount.value = PAGE_SIZE; });

// N5：卡片/列表双视图切换（持久化到 localStorage）
const viewMode = ref<'card' | 'list'>(
  (localStorage.getItem('proxyViewMode') as 'card' | 'list') || 'card',
);
watch(viewMode, (v) => localStorage.setItem('proxyViewMode', v));

// N5：列表视图列定义
const listColumns = [
  { title: '名称', dataIndex: 'name', key: 'name', width: 160 },
  { title: '类型', dataIndex: 'type', key: 'type', width: 90 },
  { title: '服务器', key: 'server', width: 120 },
  { title: '状态', key: 'state', width: 110 },
  { title: '地址映射', key: 'addr', width: 220 },
  { title: '今日流量', key: 'traffic', width: 160 },
  { title: '操作', key: 'action', width: 240 },
];

// N3：批量端口创建（TCP/UDP 8000-8010 一次建多条）
const batchPortVisible = ref(false);
const batchForm = ref<any>({
  server_id: '', type: 'tcp', namePrefix: 'port',
  localIp: '127.0.0.1', localStart: 8080, remoteStart: 8080, count: 5,
});
function openBatchPorts() {
  batchForm.value.server_id = appStore.defaultServerId || activeServerId.value || '';
  batchPortVisible.value = true;
}
function handleBatchPorts() {
  const f = batchForm.value;
  if (!f.server_id) { message.warning('请选择服务器'); return; }
  if (f.count < 1 || f.count > 100) { message.warning('数量需在 1-100 之间'); return; }
  if (f.remoteStart + f.count - 1 > 65535) { message.error('远程端口区间超出范围'); return; }
  let created = 0, skipped = 0;
  for (let i = 0; i < f.count; i++) {
    const rp = f.remoteStart + i;
    const lp = f.localStart + i;
    const name = `${f.namePrefix}-${rp}`;
    if (appStore.proxies.some(p => p.name === name)) { skipped++; continue; }
    appStore.addProxy({
      name, type: f.type, local_ip: f.localIp, local_port: lp, remote_port: rp,
      server_id: f.server_id, enabled: true,
    });
    created++;
  }
  batchPortVisible.value = false;
  message.success(`批量创建 ${created} 个代理${skipped ? `，跳过重名 ${skipped} 个` : ''}`);
}

// B4x：复制单个代理的 TOML 配置片段
async function copyProxyToml(proxy: any) {
  const lines: string[] = [`[[proxies]]`, `name = "${proxy.name}"`, `type = "${proxy.type}"`];
  if (proxy.local_ip) lines.push(`localIP = "${proxy.local_ip}"`);
  if (proxy.local_port) lines.push(`localPort = ${proxy.local_port}`);
  if (proxy.remote_port) lines.push(`remotePort = ${proxy.remote_port}`);
  if (proxy.custom_domains?.length) lines.push(`customDomains = ${JSON.stringify(proxy.custom_domains)}`);
  if (proxy.subdomain) lines.push(`subdomain = "${proxy.subdomain}"`);
  if (proxy.secret_key) lines.push(`secretKey = "${proxy.secret_key}"`);
  if (proxy.use_encryption) lines.push(`useEncryption = true`);
  if (proxy.use_compression) lines.push(`useCompression = true`);
  const text = lines.join('\n');
  try {
    await navigator.clipboard.writeText(text);
    message.success('已复制 TOML 配置片段');
  } catch {
    message.error('复制失败');
  }
}

// 获取服务器名称
function getServerName(serverId: string): string {
  const server = appStore.servers.find(s => s.id === serverId);
  return server ? server.name : '未关联';
}

// 获取活动服务器 ID
const activeServerId = computed(() => {
  const active = appStore.servers.find(s => s.active);
  return active ? active.id : '';
});

// B1x：命令面板"新建代理"入口
function onOpenAddProxy() { openAdd(); }
onMounted(() => window.addEventListener('open-add-proxy', onOpenAddProxy));
onUnmounted(() => window.removeEventListener('open-add-proxy', onOpenAddProxy));

function openAdd() {
  modalVisible.value = true;
  editingName.value = null;
  // 优先使用默认服务器，其次使用活动服务器
  form.value = {
    name: '', type: 'tcp', local_ip: '127.0.0.1', local_port: 8080,
    remote_port: 8080,
    server_id: appStore.defaultServerId || activeServerId.value || null,
    enabled: true,
    bandwidth_limit: '', traffic_limit: 0,
    custom_domains: [], subdomain: '', locations: [],
    http_user: '', http_password: '', host_header_rewrite: '',
    secret_key: '', allow_users: [], health_check_type: undefined,
    health_check_interval_s: 10, health_check_timeout_s: 3,
    health_check_max_unhealthy_times: 3, health_check_path: '/health',
    use_encryption: false, use_compression: false,
  };
}

function openEdit(proxy: any) {
  modalVisible.value = true;  editingName.value = proxy.name;
  form.value = { ...proxy };
}

async function handleSave() {
  // 验证名称
  if (!form.value.name) { 
    message.warning('请输入代理名称'); 
    return; 
  }
  
  // 验证名称是否重复（编辑时排除自己）
  const nameExists = appStore.proxies.some(p => 
    p.name === form.value.name && p.name !== editingName.value
  );
  if (nameExists) {
    message.error(`代理名称 "${form.value.name}" 已存在，请使用其他名称`);
    return;
  }
  
  // 验证端口范围
  if (form.value.local_port < 1 || form.value.local_port > 65535) {
    message.error('本地端口必须在 1-65535 之间');
    return;
  }
  if (form.value.remote_port < 1 || form.value.remote_port > 65535) {
    message.error('远程端口必须在 1-65535 之间');
    return;
  }
  
  // 验证 IP 地址格式
  const ipRegex = /^(\d{1,3}\.){3}\d{1,3}$/;
  if (!ipRegex.test(form.value.local_ip)) {
    message.error('请输入有效的 IP 地址格式');
    return;
  }
  
  // 验证 HTTP 代理的域名
  if (isHttpType.value && form.value.custom_domains && form.value.custom_domains.length > 0) {
    const domainRegex = /^[a-zA-Z0-9][-a-zA-Z0-9]*(\.[a-zA-Z0-9][-a-zA-Z0-9]*)*$/;
    for (const domain of form.value.custom_domains) {
      if (!domainRegex.test(domain)) {
        message.error(`无效的域名格式：${domain}`);
        return;
      }
    }
  }
  
  // 验证 STCP 的密钥
  if (isStcpType.value && !form.value.secret_key) {
    message.warning('STCP/XTCP 类型建议设置共享密钥以提高安全性');
  }
  
  if (editingName.value) {
    await appStore.updateProxy(editingName.value, form.value);
    message.success('保存成功');
  } else {
    await appStore.addProxy(form.value);
    message.success('添加成功');
  }
  modalVisible.value = false;
}

function handleDelete(proxy: any) {
  // B3x：确认文案带对象名
  Modal.confirm({
    title: '删除代理',
    content: `确定删除代理「${proxy.name}」？删除后不可恢复。`,
    okText: '删除',
    okType: 'danger',
    cancelText: '取消',
    onOk: () => {
      appStore.deleteProxy(proxy.name);
      message.success(`代理「${proxy.name}」已删除`);
    },
  });
}

// B2x：复制代理——预填表单，名称自动加 -copy 后缀
function handleCopy(proxy: any) {
  editingName.value = null;
  let copyName = `${proxy.name}-copy`;
  let i = 2;
  while (appStore.proxies.some(p => p.name === copyName)) {
    copyName = `${proxy.name}-copy${i++}`;
  }
  form.value = { ...JSON.parse(JSON.stringify(proxy)), name: copyName };
  modalVisible.value = true;
}

async function toggleProxy(proxy: any) {
  const target = !proxy.enabled;
  try {
    // 1. 更新并持久化代理启用状态
    await appStore.updateProxy(proxy.name, { enabled: target });
    // 2. 若所属服务器进程运行中，热重启使配置立即生效
    const restarted = proxy.server_id
      ? await appStore.restartServerIfRunning(proxy.server_id)
      : false;
    if (restarted) {
      message.success(`已${target ? '启用' : '停用'} ${proxy.name}，服务器已热重启生效`);
    } else {
      message.info(`已${target ? '启用' : '停用'} ${proxy.name}（服务器下次启动时生效）`);
    }
  } catch (e) {
    message.error(`操作失败：${typeof e === 'string' ? e : (e as Error)?.message ?? String(e)}`);
  }
}

// ===== 批量操作 =====
function toggleSelect(proxyName: string) {
  if (selectedProxies.value.has(proxyName)) {
    selectedProxies.value.delete(proxyName);
  } else {
    selectedProxies.value.add(proxyName);
  }
  batchActionVisible.value = selectedProxies.value.size > 0;
}

function selectAll() {
  if (selectedProxies.value.size === filteredProxies.value.length) {
    // 取消全选
    selectedProxies.value.clear();
    batchActionVisible.value = false;
  } else {
    // 全选
    filteredProxies.value.forEach(p => selectedProxies.value.add(p.name));
    batchActionVisible.value = true;
  }
}

function clearSelection() {
  selectedProxies.value.clear();
  batchActionVisible.value = false;
}

// 批量修改服务器
const batchServerModalVisible = ref(false);
const batchServerId = ref<string>('');

function openBatchServerModal() {
  batchServerId.value = appStore.defaultServerId || '';
  batchServerModalVisible.value = true;
}

async function handleBatchSetServer() {
  if (!batchServerId.value) {
    message.warning('请选择要关联的服务器');
    return;
  }
  
  let count = 0;
  for (const proxyName of selectedProxies.value) {
    await appStore.updateProxy(proxyName, { server_id: batchServerId.value });
    count++;
  }
  
  message.success(`已批量修改 ${count} 个代理的服务器关联`);
  batchServerModalVisible.value = false;
  clearSelection();
}

// 批量删除
async function handleBatchDelete() {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除选中的 ${selectedProxies.value.size} 个代理吗？`,
    okText: '确认删除',
    okType: 'danger',
    cancelText: '取消',
    onOk: async () => {
      let count = 0;
      for (const proxyName of selectedProxies.value) {
        await appStore.deleteProxy(proxyName);
        count++;
      }
      message.success(`已删除 ${count} 个代理`);
      clearSelection();
    },
  });
}

const isHttpType = computed(() => ['http', 'https'].includes(form.value.type));
const isStcpType = computed(() => ['stcp', 'xtcp', 'sudp'].includes(form.value.type));

// ===== 监控功能 =====
// 获取代理状态
function getProxyState(proxy: any): string {
  if (!proxy.enabled) return 'disabled';
  // 优先用代理自己的 server_id（兼容旧数据 null/'' 的情况）
  const sid = proxy.server_id || appStore.defaultServerId || appStore.servers[0]?.id || '';
  const status = appStore.getProxyStatus(sid, proxy.name);
  if (!status) return proxy.enabled ? 'online' : 'offline';
  return status.state || (proxy.enabled ? 'online' : 'offline');
}

// 判断是否在线
function isProxyOnline(proxy: any): boolean {
  return getProxyState(proxy) === 'online';
}

// 状态文本
function getProxyStateText(proxy: any): string {
  const state = getProxyState(proxy);
  if (!proxy.enabled) return '已禁用';
  if (state === 'online') return '在线';
  if (state === 'offline') return '离线';
  if (state === 'starting') return '启动中';
  return state;
}

// 状态颜色
function getProxyStateColor(proxy: any): string {
  const state = getProxyState(proxy);
  if (!proxy.enabled) return 'default';
  if (state === 'online') return 'green';
  if (state === 'offline') return 'red';
  if (state === 'starting') return 'blue';
  return 'orange';
}

// 获取今日上传流量
function getProxyTodayTrafficOut(proxy: any): number {
  const sid = proxy.server_id || appStore.defaultServerId || appStore.servers[0]?.id || '';
  const status = appStore.getProxyStatus(sid, proxy.name);
  return status?.today_traffic_out || 0;
}

// 获取今日下载流量
function getProxyTodayTrafficIn(proxy: any): number {
  const sid = proxy.server_id || appStore.defaultServerId || appStore.servers[0]?.id || '';
  const status = appStore.getProxyStatus(sid, proxy.name);
  return status?.today_traffic_in || 0;
}

// 格式化流量显示
function formatTraffic(bytes: number): string {
  return appStore.formatTraffic(bytes);
}

// 全选复选框状态
const selectAllChecked = computed({
  get: () => filteredProxies.value.length > 0 && 
             selectedProxies.value.size === filteredProxies.value.length,
  set: (val) => {
    if (val) {
      filteredProxies.value.forEach(p => selectedProxies.value.add(p.name));
      batchActionVisible.value = true;
    } else {
      selectedProxies.value.clear();
      batchActionVisible.value = false;
    }
  }
});
</script>

<template>
  <div class="page-container">
    <div class="page-header">
      <h1 class="page-title">
        <AppstoreOutlined class="header-icon" />
        代理管理
      </h1>
      <a-space>
        <a-radio-group v-model:value="viewMode" button-style="solid" size="small">
          <a-radio-button value="card"><AppstoreOutlined /></a-radio-button>
          <a-radio-button value="list"><BarsOutlined /></a-radio-button>
        </a-radio-group>
        <a-button @click="openBatchPorts">
          <template #icon><ThunderboltOutlined /></template>
          批量端口
        </a-button>
        <a-button type="primary" @click="openAdd">
          <template #icon><PlusOutlined /></template>
          添加代理
        </a-button>
      </a-space>
    </div>

    <!-- 搜索和过滤 -->
    <div class="filter-bar">
      <a-checkbox 
        v-model:checked="selectAllChecked" 
        @change="selectAll"
        style="margin-right: 16px;"
      >
        全选
      </a-checkbox>
      
      <a-input
        v-model:value="searchKeyword"
        placeholder="搜索代理名称或类型"
        class="search-input"
        allow-clear
        style="flex: 1; max-width: 320px;"
      />
      
      <a-select
        v-model:value="serverFilter"
        placeholder="选择服务器"
        class="server-filter"
        allow-clear
        style="width: 200px; margin-left: 16px;"
      >
        <a-select-option value="">全部服务器</a-select-option>
        <a-select-option
          v-for="server in appStore.servers"
          :key="server.id"
          :value="server.id"
        >
          {{ server.name }}
        </a-select-option>
      </a-select>
    </div>

    <!-- 批量操作工具栏 -->
    <div v-if="batchActionVisible" class="batch-action-bar">
      <a-space>
        <span class="selected-count">已选择 {{ selectedProxies.size }} 个代理</span>
        <a-button size="small" @click="openBatchServerModal">批量修改服务器</a-button>
        <a-button size="small" danger @click="handleBatchDelete">批量删除</a-button>
        <a-button size="small" @click="clearSelection">取消选择</a-button>
      </a-space>
    </div>

    <!-- 空状态 -->
    <div v-if="filteredProxies.length === 0" class="empty-state">
      <a-empty description="暂无代理配置">
        <template #description>
          <div class="empty-guide">
            <p style="margin-bottom: 16px; font-size: 14px; color: #64748b;">
              暂无代理配置
            </p>
            <a-button type="primary" @click="openAdd">
              <template #icon><PlusOutlined /></template>
              添加第一个代理
            </a-button>
            <p v-if="appStore.servers.length === 0" class="help-text" style="margin-top: 12px; font-size: 13px;">
              💡 提示：建议先添加服务器，再添加代理
            </p>
            <p v-else-if="!appStore.defaultServerId" class="help-text" style="margin-top: 12px; font-size: 13px;">
              💡 提示：可在设置中配置默认服务器，新建代理时自动关联
            </p>
          </div>
        </template>
      </a-empty>
    </div>

    <!-- 代理卡片网格（N5：card 视图）-->
    <div v-else-if="viewMode === 'card'" class="proxy-grid">
      <div 
        v-for="proxy in visibleProxies" 
        :key="proxy.name"
        class="proxy-card-wrapper"
        :class="{ 'proxy-selected': selectedProxies.has(proxy.name) }"
      >
        <a-checkbox 
          class="proxy-checkbox"
          :checked="selectedProxies.has(proxy.name)"
          @click.stop="toggleSelect(proxy.name)"
        />
        <a-card
          class="proxy-card"
          :class="{ 'proxy-card-enabled': proxy.enabled }"
          hoverable
          @click.stop="toggleSelect(proxy.name)"
        >
        <template #title>
          <div class="proxy-card-title">
            <span class="proxy-name">
              <ThunderboltOutlined v-if="['stcp', 'xtcp', 'sudp'].includes(proxy.type)" />
              <GlobalOutlined v-else-if="['http', 'https'].includes(proxy.type)" />
              <AppstoreOutlined v-else />
              {{ proxy.name }}
            </span>
            <a-tag :color="getProxyTypeColor(proxy.type)">
              {{ proxy.type.toUpperCase() }}
            </a-tag>
          </div>
        </template>

        <div class="proxy-card-content">
          <!-- 关联服务器 -->
          <div class="proxy-info-row">
            <span class="info-label">服务器：</span>
            <span class="info-value server-name">
              <a-tag :color="proxy.server_id ? 'blue' : 'default'" size="small">
                {{ getServerName(proxy.server_id) }}
              </a-tag>
            </span>
          </div>
          
          <!-- 连接状态 -->
          <div class="proxy-info-row">
            <span class="info-label">状态：</span>
            <span class="info-value">
              <a-tag :color="getProxyStateColor(proxy)" size="small">
                <template #icon>
                  <CheckCircleOutlined v-if="isProxyOnline(proxy)" />
                  <CloseCircleOutlined v-else />
                </template>
                {{ getProxyStateText(proxy) }}
              </a-tag>
            </span>
          </div>
          
          <!-- 本地地址 -->
          <div class="proxy-info-row">
            <span class="info-label">本地：</span>
            <span class="info-value">{{ proxy.local_ip }}:{{ proxy.local_port }}</span>
          </div>
          
          <!-- 远程端口 -->
          <div class="proxy-info-row">
            <span class="info-label">远程：</span>
            <span class="info-value">:{{ proxy.remote_port }}</span>
          </div>
          
          <!-- 今日流量 -->
          <div class="proxy-info-row">
            <span class="info-label">流量：</span>
            <span class="info-value traffic-text">
              <ArrowUpOutlined style="color: #52c41a; font-size: 12px;" />
              {{ formatTraffic(getProxyTodayTrafficOut(proxy)) }}
              &nbsp;
              <ArrowDownOutlined style="color: #1890ff; font-size: 12px;" />
              {{ formatTraffic(getProxyTodayTrafficIn(proxy)) }}
            </span>
          </div>
          
          <!-- 其他信息 -->
          <div v-if="proxy.bandwidth_limit" class="proxy-info-row">
            <span class="info-label">带宽：</span>
            <span class="info-value">{{ proxy.bandwidth_limit }}</span>
          </div>
          
          <div v-if="proxy.traffic_limit > 0" class="proxy-info-row">
            <span class="info-label">流量：</span>
            <span class="info-value">{{ proxy.traffic_limit }} MB</span>
          </div>
        </div>

        <template #actions>
          <a @click="toggleProxy(proxy)">{{ proxy.enabled ? '停止' : '启动' }}</a>
          <a @click="openEdit(proxy)">编辑</a>
          <a-dropdown>
            <a @click="(e: MouseEvent) => e.preventDefault()"><MoreOutlined /> 更多</a>
            <template #overlay>
              <a-menu>
                <a-menu-item key="copy" @click="handleCopy(proxy)">
                  <CopyOutlined /> 复制为新代理
                </a-menu-item>
                <a-menu-item key="delete" @click="handleDelete(proxy)">
                  <span style="color: #ef4444;">删除「{{ proxy.name }}」</span>
                </a-menu-item>
              </a-menu>
            </template>
          </a-dropdown>
        </template>
      </a-card>
      </div>
    </div>

    <!-- A3：渐进渲染"显示更多" -->
    <div v-if="viewMode === 'card' && visibleCount < filteredProxies.length" style="text-align: center; margin: 16px 0;">
      <a-button @click="visibleCount += PAGE_SIZE">
        显示更多（还有 {{ filteredProxies.length - visibleCount }} 个）
      </a-button>
    </div>

    <!-- N5：列表视图 -->
    <a-table
      v-else-if="viewMode === 'list'"
      :data-source="filteredProxies"
      :columns="listColumns"
      row-key="name"
      size="middle"
      :pagination="{ pageSize: 20, hideOnSinglePage: true }"
      class="proxy-table"
    >
      <template #bodyCell="{ column, record }">
        <template v-if="column.key === 'type'">
          <a-tag :color="getProxyTypeColor(record.type)">{{ record.type.toUpperCase() }}</a-tag>
        </template>
        <template v-else-if="column.key === 'server'">
          <a-tag :color="record.server_id ? 'blue' : 'default'">{{ getServerName(record.server_id) }}</a-tag>
        </template>
        <template v-else-if="column.key === 'state'">
          <a-tag :color="getProxyStateColor(record)">
            <template #icon><CheckCircleOutlined v-if="isProxyOnline(record)" /><CloseCircleOutlined v-else /></template>
            {{ getProxyStateText(record) }}
          </a-tag>
        </template>
        <template v-else-if="column.key === 'addr'">
          <span class="mono">{{ record.local_ip }}:{{ record.local_port }} → :{{ record.remote_port }}</span>
        </template>
        <template v-else-if="column.key === 'traffic'">
          <span class="mono">↑{{ formatTraffic(getProxyTodayTrafficOut(record)) }} ↓{{ formatTraffic(getProxyTodayTrafficIn(record)) }}</span>
        </template>
        <template v-else-if="column.key === 'action'">
          <a-space>
            <a @click="toggleProxy(record)">{{ record.enabled ? '停止' : '启动' }}</a>
            <a @click="openEdit(record)">编辑</a>
            <a @click="copyProxyToml(record)">复制TOML</a>
            <a style="color:#ef4444;" @click="handleDelete(record)">删除</a>
          </a-space>
        </template>
      </template>
    </a-table>

    <!-- N3：批量端口创建弹窗 -->
    <a-modal v-model:open="batchPortVisible" title="批量端口创建" @ok="handleBatchPorts" width="520px">
      <a-form layout="vertical">
        <a-form-item label="目标服务器" required>
          <a-select v-model:value="batchForm.server_id" placeholder="选择服务器">
            <a-select-option v-for="s in appStore.servers" :key="s.id" :value="s.id">{{ s.name }}</a-select-option>
          </a-select>
        </a-form-item>
        <a-row :gutter="12">
          <a-col :span="12">
            <a-form-item label="类型">
              <a-select v-model:value="batchForm.type">
                <a-select-option value="tcp">TCP</a-select-option>
                <a-select-option value="udp">UDP</a-select-option>
              </a-select>
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="名称前缀">
              <a-input v-model:value="batchForm.namePrefix" placeholder="port" />
            </a-form-item>
          </a-col>
        </a-row>
        <a-form-item label="本地 IP">
          <a-input v-model:value="batchForm.localIp" placeholder="127.0.0.1" />
        </a-form-item>
        <a-row :gutter="12">
          <a-col :span="8"><a-form-item label="本地起始端口"><a-input-number v-model:value="batchForm.localStart" :min="1" :max="65535" style="width:100%" /></a-form-item></a-col>
          <a-col :span="8"><a-form-item label="远程起始端口"><a-input-number v-model:value="batchForm.remoteStart" :min="1" :max="65535" style="width:100%" /></a-form-item></a-col>
          <a-col :span="8"><a-form-item label="数量"><a-input-number v-model:value="batchForm.count" :min="1" :max="100" style="width:100%" /></a-form-item></a-col>
        </a-row>
        <div class="form-help">将创建 {{ batchForm.count }} 个代理：远程端口 {{ batchForm.remoteStart }}-{{ batchForm.remoteStart + batchForm.count - 1 }}，命名为 前缀-远程端口</div>
      </a-form>
    </a-modal>

    <!-- 编辑/添加对话框 -->
    <a-modal
      v-model:open="modalVisible"
      :title="editingName ? '编辑代理' : '添加代理'"
      @ok="handleSave"
      width="900px"
    >
      <a-form :model="form" layout="vertical">
        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item label="名称" required>
              <a-input v-model:value="form.name" placeholder="请输入代理名称" />
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="类型" required>
              <a-select v-model:value="form.type">
                <a-select-option v-for="t in proxyTypes" :key="t.value" :value="t.value">
                  {{ t.label }}
                </a-select-option>
              </a-select>
            </a-form-item>
          </a-col>
        </a-row>

        <!-- 服务器关联 -->
        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item label="所属服务器">
              <a-select v-model:value="form.server_id" placeholder="选择关联的服务器" allow-clear>
                <a-select-option
                  v-for="server in appStore.servers"
                  :key="server.id"
                  :value="server.id"
                >
                  {{ server.name }} ({{ server.serverAddr }})
                </a-select-option>
              </a-select>
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="启用">
              <a-switch v-model:checked="form.enabled" />
            </a-form-item>
          </a-col>
        </a-row>

        <a-row :gutter="16">
          <a-col :span="8">
            <a-form-item label="本地 IP">
              <a-input v-model:value="form.local_ip" placeholder="127.0.0.1" />
            </a-form-item>
          </a-col>
          <a-col :span="8">
            <a-form-item label="本地端口">
              <a-input-number v-model:value="form.local_port" :min="1" :max="65535" style="width: 100%" />
            </a-form-item>
          </a-col>
          <a-col :span="8">
            <a-form-item label="远程端口">
              <a-input-number v-model:value="form.remote_port" :min="1" :max="65535" style="width: 100%" />
            </a-form-item>
          </a-col>
        </a-row>

        <template v-if="isHttpType">
          <a-divider orientation="left">HTTP/HTTPS 配置</a-divider>
          <a-form-item label="自定义域名">
            <a-select v-model:value="form.custom_domains" mode="tags" style="width: 100%" />
          </a-form-item>
          <a-row :gutter="16">
            <a-col :span="12">
              <a-form-item label="子域名">
                <a-input v-model:value="form.subdomain" />
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="Host 重写">
                <a-input v-model:value="form.host_header_rewrite" />
              </a-form-item>
            </a-col>
          </a-row>
          <a-row :gutter="16">
            <a-col :span="12">
              <a-form-item label="HTTP 用户">
                <a-input v-model:value="form.http_user" />
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="HTTP 密码">
                <a-input-password v-model:value="form.http_password" />
              </a-form-item>
            </a-col>
          </a-row>
        </template>

        <template v-if="isStcpType">
          <a-divider orientation="left">STCP/XTCP 配置</a-divider>
          <a-form-item label="共享密钥">
            <a-input-password v-model:value="form.secret_key" />
          </a-form-item>
          <a-form-item label="允许访问者">
            <a-select v-model:value="form.allow_users" mode="tags" style="width: 100%" />
          </a-form-item>
        </template>

        <a-divider orientation="left">健康检查</a-divider>
        <a-row :gutter="16">
          <a-col :span="8">
            <a-form-item label="检查类型">
              <a-select v-model:value="form.health_check_type" allow-clear>
                <a-select-option value="tcp">TCP</a-select-option>
                <a-select-option value="http">HTTP</a-select-option>
              </a-select>
            </a-form-item>
          </a-col>
          <a-col :span="8" v-if="form.health_check_type === 'http'">
            <a-form-item label="检查路径">
              <a-input v-model:value="form.health_check_path" />
            </a-form-item>
          </a-col>
        </a-row>
        <a-row :gutter="16" v-if="form.health_check_type">
          <a-col :span="6">
            <a-form-item label="间隔 (秒)">
              <a-input-number v-model:value="form.health_check_interval_s" :min="1" style="width: 100%" />
            </a-form-item>
          </a-col>
          <a-col :span="6">
            <a-form-item label="超时 (秒)">
              <a-input-number v-model:value="form.health_check_timeout_s" :min="1" style="width: 100%" />
            </a-form-item>
          </a-col>
          <a-col :span="6">
            <a-form-item label="最大失败次数">
              <a-input-number v-model:value="form.health_check_max_unhealthy_times" :min="1" style="width: 100%" />
            </a-form-item>
          </a-col>
        </a-row>

        <a-divider orientation="left">其他选项</a-divider>
        <a-row :gutter="16">
          <a-col :span="8">
            <a-form-item label="带宽限制">
              <a-input v-model:value="form.bandwidth_limit" placeholder="例：1MB" />
            </a-form-item>
          </a-col>
          <a-col :span="8">
            <a-form-item label="流量限制 (MB)">
              <a-input-number v-model:value="form.traffic_limit" :min="0" style="width: 100%" />
            </a-form-item>
          </a-col>
          <a-col :span="8">
            <a-space>
              <a-checkbox v-model:checked="form.use_encryption">启用加密</a-checkbox>
              <a-checkbox v-model:checked="form.use_compression">启用压缩</a-checkbox>
            </a-space>
          </a-col>
        </a-row>
      </a-form>
    </a-modal>

    <!-- 批量修改服务器对话框 -->
    <a-modal
      v-model:open="batchServerModalVisible"
      title="批量修改服务器"
      @ok="handleBatchSetServer"
      width="500px"
    >
      <p>已选择 <strong>{{ selectedProxies.size }}</strong> 个代理</p>
      <a-form layout="vertical">
        <a-form-item label="选择要关联的服务器" required>
          <a-select v-model:value="batchServerId" placeholder="请选择服务器">
            <a-select-option
              v-for="server in appStore.servers"
              :key="server.id"
              :value="server.id"
            >
              {{ server.name }} ({{ server.serverAddr }}:{{ server.serverPort }})
            </a-select-option>
          </a-select>
        </a-form-item>
      </a-form>
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
    display: flex;
    align-items: center;
    gap: 12px;

    .header-icon {
      font-size: 28px;
      color: #8b5cf6;
    }
  }

  .ant-btn {
    border-radius: 8px;
    font-weight: 500;
    transition: all 0.2s ease;

    &:hover {
      transform: translateY(-1px);
      box-shadow: 0 2px 8px rgba(139, 92, 246, 0.3);
    }

    &:active {
      transform: scale(0.98);
    }
  }
}

.filter-bar {
  display: flex;
  align-items: center;
  margin-bottom: 24px;
  gap: 16px;

  .search-input {
    border-radius: 8px;
  }

  .server-filter {
    border-radius: 8px;
  }
}

.batch-action-bar {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 12px 20px;
  border-radius: 8px;
  margin-bottom: 24px;
  color: white;
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
  animation: slideDown 0.3s ease;

  .selected-count {
    font-weight: 600;
    margin-right: 16px;
  }

  :deep(.ant-btn) {
    background: rgba(255, 255, 255, 0.9);
    border: none;
    color: #667eea;
    
    &:hover {
      background: white;
      color: #764ba2;
    }
  }
}

@keyframes slideDown {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.proxy-card-wrapper {
  position: relative;
  padding-left: 36px;
  transition: all 0.3s ease;
  
  &.proxy-selected {
    .proxy-card {
      border: 2px solid #667eea;
      box-shadow: 0 4px 16px rgba(102, 126, 234, 0.3);
    }
  }
  
  .proxy-checkbox {
    position: absolute;
    left: 8px;
    top: 16px;
    z-index: 10;
  }
}

.empty-state {
  margin: 48px 0;
}

.proxy-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}

.proxy-card {
  border-radius: 12px;
  transition: all 0.3s ease;
  position: relative;
  overflow: hidden;

  &:hover {
    transform: translateY(-4px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
  }

  &.proxy-card-enabled {
    border: 2px solid #52c41a;
    box-shadow: 0 4px 16px rgba(82, 196, 26, 0.2);
  }

  .proxy-card-title {
    display: flex;
    justify-content: space-between;
    align-items: center;

    .proxy-name {
      font-size: 16px;
      font-weight: 600;
      color: #1e293b;
      display: flex;
      align-items: center;
      gap: 8px;
    }
  }

  .proxy-card-content {
    .proxy-info-row {
      margin-bottom: 10px;
      display: flex;
      align-items: center;
      gap: 8px;

      .info-label {
        font-weight: 500;
        color: #64748b;
        min-width: 50px;
      }

      .info-value {
        color: #1e293b;
        
        &.server-name {
          flex: 1;
        }
        
        &.traffic-text {
          font-size: 13px;
          font-family: 'Courier New', monospace;
          color: #475569;
        }
      }
    }
  }

  :deep(.ant-card-actions) {
    li {
      a {
        color: #8b5cf6;
        transition: all 0.2s;

        &:hover {
          color: #7c3aed;
        }

        &.text-danger {
          color: #ef4444;

          &:hover {
            color: #dc2626;
          }
        }
      }
    }
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

  :deep(.ant-divider-with-text) {
    &::before {
      border-top-color: #e2e8f0;
    }
    &::after {
      border-top-color: #e2e8f0;
    }
  }

  .ant-btn-primary {
    border-radius: 8px;

    &:active {
      transform: scale(0.98);
    }
  }
}
</style>
