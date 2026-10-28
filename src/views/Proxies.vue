<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { useAppStore } from '@/stores/app';
import { PlusOutlined, AppstoreOutlined, GlobalOutlined, ThunderboltOutlined, CheckCircleOutlined, CloseCircleOutlined, ArrowUpOutlined, ArrowDownOutlined, MoreOutlined, CopyOutlined, BarsOutlined, EditOutlined, DeleteOutlined, PlayCircleOutlined, PauseCircleOutlined, ApiOutlined, CloudServerOutlined } from '@ant-design/icons-vue';
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
  { title: '映射地址', key: 'traffic', width: 160 },
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

// 获取代理映射的远程地址（frpc admin API 提供，替代不支持的流量显示）
function getProxyRemoteAddr(proxy: any): string {
  const sid = proxy.server_id || appStore.defaultServerId || appStore.servers[0]?.id || '';
  const status = appStore.getProxyStatus(sid, proxy.name);
  return status?.remote_addr || '—';
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
      <div class="empty-card">
        <div class="empty-icon">
          <ApiOutlined />
        </div>
        <h3 class="empty-title">
          {{ searchKeyword || serverFilter ? '没有匹配的代理' : '还没有代理配置' }}
        </h3>
        <p class="empty-desc">
          <template v-if="searchKeyword || serverFilter">
            调整搜索或筛选条件后再试
          </template>
          <template v-else-if="appStore.servers.length === 0">
            请先添加服务器，再创建代理规则
          </template>
          <template v-else>
            把本地端口映射到公网，支持 TCP/UDP/HTTP 等 8 种类型
          </template>
        </p>
        <a-button v-if="appStore.servers.length === 0" type="primary" size="large" @click="$router.push('/servers')">
          <template #icon><CloudServerOutlined /></template>
          先去添加服务器
        </a-button>
        <a-button v-else-if="searchKeyword || serverFilter" @click="searchKeyword = ''; serverFilter = ''">
          清空筛选
        </a-button>
        <a-button v-else type="primary" size="large" @click="openAdd">
          <template #icon><PlusOutlined /></template>
          添加第一个代理
        </a-button>
        <p v-if="appStore.servers.length > 0 && !appStore.defaultServerId" class="empty-hint">
          可在设置中配置默认服务器，新建代理时自动关联
        </p>
      </div>
    </div>

    <!-- 代理卡片网格（N5：card 视图）-->
    <div v-else-if="viewMode === 'card'" class="proxy-grid">
      <div
        v-for="proxy in visibleProxies"
        :key="proxy.name"
        class="proxy-card"
        :class="{
          'is-online': isProxyOnline(proxy),
          'is-selected': selectedProxies.has(proxy.name),
          'is-disabled': !proxy.enabled,
        }"
        @click.stop="toggleSelect(proxy.name)"
      >
        <!-- 选中复选框（hover / 选中时显示） -->
        <a-checkbox
          class="proxy-checkbox"
          :checked="selectedProxies.has(proxy.name)"
          @click.stop="toggleSelect(proxy.name)"
        />

        <!-- 头部：图标 + 名称 + 类型 -->
        <div class="card-head">
          <div class="head-left">
            <span class="type-icon" :class="`type-${proxy.type}`">
              <ThunderboltOutlined v-if="['stcp', 'xtcp', 'sudp'].includes(proxy.type)" />
              <GlobalOutlined v-else-if="['http', 'https'].includes(proxy.type)" />
              <AppstoreOutlined v-else />
            </span>
            <span class="proxy-name" :title="proxy.name">{{ proxy.name }}</span>
          </div>
          <div class="head-right">
            <a-tag :color="getProxyTypeColor(proxy.type)" class="type-tag">
              {{ proxy.type.toUpperCase() }}
            </a-tag>
            <span class="status-led" :class="{ on: isProxyOnline(proxy) }"></span>
          </div>
        </div>

        <!-- 映射关系：本地 → 远程 -->
        <div class="mapping-line">
          <span class="map-side local">{{ proxy.local_ip }}:{{ proxy.local_port }}</span>
          <span class="map-arrow">→</span>
          <span class="map-side remote">:{{ proxy.remote_port }}</span>
        </div>

        <!-- 属性标签 -->
        <div class="meta-row">
          <span class="meta-chip">{{ getServerName(proxy.server_id) }}</span>
          <span class="meta-chip" :class="isProxyOnline(proxy) ? 'chip-online' : 'chip-offline'">
            {{ getProxyStateText(proxy) }}
          </span>
          <span
            v-if="getProxyRemoteAddr(proxy) !== '—'"
            class="meta-chip mono"
            :title="getProxyRemoteAddr(proxy)"
          >{{ getProxyRemoteAddr(proxy) }}</span>
          <span v-if="proxy.bandwidth_limit" class="meta-chip">限速 {{ proxy.bandwidth_limit }}</span>
          <span v-if="proxy.traffic_limit > 0" class="meta-chip">限额 {{ proxy.traffic_limit }}M</span>
        </div>

        <!-- 操作区 -->
        <div class="card-actions" @click.stop>
          <a-button
            size="small"
            :type="proxy.enabled ? 'default' : 'primary'"
            :danger="proxy.enabled"
            class="act-btn"
            @click="toggleProxy(proxy)"
          >
            <template #icon>
              <PauseCircleOutlined v-if="proxy.enabled" />
              <PlayCircleOutlined v-else />
            </template>
            {{ proxy.enabled ? '停止' : '启动' }}
          </a-button>
          <a-button size="small" class="act-btn" @click="openEdit(proxy)">
            <template #icon><EditOutlined /></template>编辑
          </a-button>

          <span class="actions-spacer"></span>

          <a-tooltip title="复制为新代理">
            <a-button size="small" type="text" class="icon-btn" @click="handleCopy(proxy)">
              <template #icon><CopyOutlined /></template>
            </a-button>
          </a-tooltip>
          <a-popconfirm
            :title="`确定删除代理「${proxy.name}」？`"
            ok-text="删除"
            ok-type="danger"
            @confirm="handleDelete(proxy)"
          >
            <a-tooltip title="删除">
              <a-button size="small" type="text" danger class="icon-btn">
                <template #icon><DeleteOutlined /></template>
              </a-button>
            </a-tooltip>
          </a-popconfirm>
        </div>
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
          <span class="mono">{{ getProxyRemoteAddr(record) }}</span>
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

/* 空状态 */
.empty-state {
  margin: 40px 0;
  display: flex;
  justify-content: center;
}

.empty-card {
  width: 100%;
  max-width: 400px;
  padding: 40px 32px;
  background: #fff;
  border: 1px dashed #dbe3ee;
  border-radius: 16px;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;

  .empty-icon {
    width: 56px;
    height: 56px;
    border-radius: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 26px;
    color: #7c3aed;
    background: linear-gradient(135deg, rgba(124, 58, 237, 0.12), rgba(37, 99, 235, 0.08));
    margin-bottom: 16px;
  }

  .empty-title {
    font-size: 16px;
    font-weight: 600;
    color: #1e293b;
    margin: 0 0 6px;
  }

  .empty-desc {
    font-size: 13px;
    color: #94a3b8;
    margin: 0 0 20px;
    line-height: 1.6;
  }

  .empty-hint {
    font-size: 12px;
    color: #cbd5e1;
    margin: 14px 0 0;
  }
}

/* ===== 代理卡片（紧凑风格，与服务页一致）===== */
.proxy-grid {
  display: grid;
  // 固定最大宽度，避免单卡片被拉伸铺满整行
  grid-template-columns: repeat(auto-fill, minmax(280px, 340px));
  gap: 14px;
  align-items: start;
}

.proxy-card {
  position: relative;
  background: #fff;
  border-radius: 12px;
  border: 1px solid #eef2f7;
  padding: 14px 16px 12px;
  cursor: pointer;
  transition: border-color 0.18s ease, box-shadow 0.18s ease, transform 0.18s ease;
  display: flex;
  flex-direction: column;
  gap: 10px;

  &:hover {
    border-color: #dbeafe;
    box-shadow: 0 4px 16px rgba(15, 23, 42, 0.08);
    transform: translateY(-2px);

    .proxy-checkbox { opacity: 1; }
  }

  // 在线：绿色描边 + 浅绿渐变
  &.is-online {
    border-color: rgba(34, 197, 94, 0.35);
    background: linear-gradient(180deg, rgba(240, 253, 244, 0.65) 0%, #fff 44%);
  }

  // 选中：蓝色描边
  &.is-selected {
    border-color: #6366f1;
    box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.16);
  }

  // 已禁用代理：整体降饱和
  &.is-disabled {
    .card-head .proxy-name,
    .mapping-line { opacity: 0.55; }
  }

  // 选中复选框
  .proxy-checkbox {
    position: absolute;
    top: 10px;
    right: 10px;
    opacity: 0;
    transition: opacity 0.15s ease;
    z-index: 2;
  }
  &.is-selected .proxy-checkbox { opacity: 1; }

  /* 头部 */
  .card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding-right: 22px; // 给复选框留位

    .head-left {
      display: flex;
      align-items: center;
      gap: 8px;
      min-width: 0;
    }

    .type-icon {
      width: 26px;
      height: 26px;
      flex-shrink: 0;
      border-radius: 7px;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 13px;
      background: rgba(37, 99, 235, 0.09);
      color: #2563eb;

      &.type-stcp, &.type-xtcp, &.type-sudp {
        background: rgba(124, 58, 237, 0.1);
        color: #7c3aed;
      }
      &.type-http, &.type-https {
        background: rgba(16, 185, 129, 0.1);
        color: #10b981;
      }
    }

    .proxy-name {
      font-size: 14px;
      font-weight: 600;
      color: #0f172a;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .head-right {
      display: flex;
      align-items: center;
      gap: 6px;
      flex-shrink: 0;
    }

    .type-tag {
      margin: 0;
      font-size: 10px;
      line-height: 17px;
      padding: 0 6px;
      border-radius: 5px;
      font-weight: 600;
      letter-spacing: 0.3px;
    }

    .status-led {
      width: 7px;
      height: 7px;
      border-radius: 50%;
      background: #cbd5e1;

      &.on {
        background: #22c55e;
        box-shadow: 0 0 0 3px rgba(34, 197, 94, 0.16);
      }
    }
  }

  /* 映射关系：本地 → 远程 */
  .mapping-line {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 7px 10px;
    background: #f8fafc;
    border-radius: 8px;
    font-family: ui-monospace, 'SF Mono', Menlo, monospace;
    font-size: 12px;
    overflow: hidden;

    .map-side {
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;

      &.local { color: #475569; flex: 1; min-width: 0; }
      &.remote { color: #2563eb; font-weight: 600; flex-shrink: 0; }
    }

    .map-arrow {
      color: #cbd5e1;
      flex-shrink: 0;
      font-size: 11px;
    }
  }

  /* 属性标签 */
  .meta-row {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;

    .meta-chip {
      font-size: 11px;
      line-height: 20px;
      padding: 0 7px;
      border-radius: 5px;
      background: #f1f5f9;
      color: #475569;
      white-space: nowrap;
      max-width: 100%;
      overflow: hidden;
      text-overflow: ellipsis;

      &.chip-online {
        background: rgba(34, 197, 94, 0.1);
        color: #16a34a;
      }
      &.chip-offline {
        background: #f8fafc;
        color: #94a3b8;
      }
      &.mono {
        font-family: ui-monospace, monospace;
        color: #7c3aed;
        background: rgba(124, 58, 237, 0.07);
      }
    }
  }

  /* 操作区 */
  .card-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    padding-top: 9px;
    border-top: 1px solid #f1f5f9;

    .actions-spacer { flex: 1; }

    .act-btn {
      border-radius: 7px;
      font-size: 12px;
      height: 27px;
      padding: 0 10px;
    }

    .icon-btn {
      width: 27px;
      height: 27px;
      padding: 0;
      border-radius: 7px;
      color: #94a3b8;

      &:hover {
        color: #2563eb;
        background: rgba(37, 99, 235, 0.08);
      }
      &.ant-btn-dangerous:hover {
        color: #ef4444;
        background: rgba(239, 68, 68, 0.08);
      }
    }
  }
}

/* 深色模式 */
html.dark .empty-card {
  background: #1e293b;
  border-color: #334155;

  .empty-title { color: #e2e8f0; }
  .empty-desc { color: #64748b; }
  .empty-hint { color: #475569; }
}

html.dark {
  .proxy-card {
    background: #1e293b;
    border-color: #334155;

    &:hover {
      border-color: #3b82f6;
      box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    }
    &.is-online {
      border-color: rgba(34, 197, 94, 0.3);
      background: linear-gradient(180deg, rgba(34, 197, 94, 0.07) 0%, #1e293b 44%);
    }
    &.is-selected {
      border-color: #818cf8;
      box-shadow: 0 0 0 2px rgba(129, 140, 248, 0.2);
    }

    .card-head .proxy-name { color: #e2e8f0; }
    .mapping-line { background: #0f172a; }
    .mapping-line .map-side.local { color: #cbd5e1; }
    .mapping-line .map-side.remote { color: #60a5fa; }
    .meta-row .meta-chip { background: #0f172a; color: #94a3b8; }
    .meta-row .meta-chip.chip-online {
      background: rgba(34, 197, 94, 0.14);
      color: #4ade80;
    }
    .card-actions { border-top-color: #334155; }
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
