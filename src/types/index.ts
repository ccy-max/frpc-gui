// FRP 配置类型定义

export interface FrpConfig {
  server_addr: string;
  server_port: number;
  auth: AuthConfig;
  user?: string;
  tls: TlsConfig;
  log: LogConfig;
  admin: AdminConfig;
  proxies: ProxyConfig[];
}

export interface AuthConfig {
  method: string;
  token?: string;
  additional?: string;
}

export interface TlsConfig {
  enable: boolean;
  cert_file?: string;
  key_file?: string;
  trusted_ca_file?: string;
}

export interface LogConfig {
  to: string;
  level: string;
  max_days: number;
}

export interface AdminConfig {
  addr: string;
  port: number;
  user?: string;
  password?: string;
}

export interface ProxyConfig {
  name: string;
  type: ProxyType;
  local_ip?: string;
  local_port?: number;
  remote_port?: number;
  custom_domains?: string[];
  subdomain?: string;
  locations?: string[];
  http_user?: string;
  http_password?: string;
  use_encryption?: boolean;
  use_compression?: boolean;
  secret_key?: string;
  role?: 'visitor' | 'bind';
  server_name?: string;
  enabled: boolean;
}

export type ProxyType = 'tcp' | 'udp' | 'http' | 'https' | 'stcp' | 'xtcp' | 'sudp';

export interface ProcessStatus {
  running: boolean;
  pid: number | null;
  state: string;
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
