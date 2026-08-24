// FRP 配置类型定义

export interface FrpConfig {
  // 基础配置
  server_addr: string;
  server_port: number;
  
  // 认证
  auth: AuthConfig;
  user?: string;
  
  // 传输
  transport: TransportConfig;
  
  // TLS
  tls: TlsConfig;
  
  // 日志
  log: LogConfig;
  
  // 管理控制台
  admin: AdminConfig;
  
  // 高级
  dns_server?: string;
  login_fail_exit?: boolean;
  tcp_mux?: boolean;
  udp_packet_size?: number;
  heartbeat_interval?: number;
  heartbeat_timeout?: number;
  
  // 代理
  proxies: ProxyConfig[];
}

export interface AuthConfig {
  method: string;
  token?: string;
  additional?: string;
}

export interface TransportConfig {
  protocol: 'tcp' | 'kcp' | 'quic' | 'websocket';
  dial_server_timeout?: number;
  dial_server_keepalive?: number;
  proxy_url?: string;
  bind_addr?: string;
  bind_port?: number;
}

export interface TlsConfig {
  enable: boolean;
  cert_file?: string;
  key_file?: string;
  trusted_ca_file?: string;
  server_name?: string;
}

export interface LogConfig {
  to: string;
  level: 'trace' | 'debug' | 'info' | 'warn' | 'error';
  max_days: number;
}

export interface AdminConfig {
  addr: string;
  port: number;
  user?: string;
  password?: string;
}

export interface ProxyConfig {
  // 基础
  name: string;
  type: ProxyType;
  enabled: boolean;
  
  // 通用
  local_ip?: string;
  local_port?: number;
  remote_port?: number;
  
  // 带宽/流量
  bandwidth_limit?: string;
  traffic_limit?: number;
  
  // HTTP/HTTPS
  custom_domains?: string[];
  subdomain?: string;
  locations?: string[];
  http_user?: string;
  http_password?: string;
  host_header_rewrite?: string;
  headers?: Record<string, string>;
  
  // STCP/XTCP/SUDP
  secret_key?: string;
  allow_users?: string[];
  role?: 'visitor' | 'bind';
  server_name?: string;
  
  // 插件
  plugin?: string;
  plugin_params?: Record<string, string>;
  
  // 健康检查
  health_check_type?: 'tcp' | 'http';
  health_check_interval_s?: number;
  health_check_timeout_s?: number;
  health_check_max_unhealthy_times?: number;
  health_check_path?: string;
  
  // 其他
  use_encryption?: boolean;
  use_compression?: boolean;
  meta_tokens?: Record<string, string>;
}

export type ProxyType = 'tcp' | 'udp' | 'http' | 'https' | 'stcp' | 'xtcp' | 'sudp' | 'tcpmux';

export interface ProcessStatus {
  running: boolean;
  pid: number | null;
  state: 'stopped' | 'starting' | 'running' | 'stopping' | 'error';
  uptime?: number; // 运行时长（毫秒）
  connection_error?: string;
}

export interface LogEntry {
  timestamp: number;
  level: 'debug' | 'info' | 'warn' | 'error';
  message: string;
  source: 'app' | 'frpc';
}

export interface AppConfig {
  language: 'zh-CN' | 'en-US';
  theme: 'light' | 'dark' | 'auto';
  frpc_path: string;
  config_path: string;
  log_path: string;
  auto_start: boolean;
  minimize_to_tray: boolean;
  close_to_tray: boolean;
}

export interface LocalPort {
  protocol: string;
  ip: string;
  port: number;
}
