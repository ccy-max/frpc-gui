// FRP 代理类型定义

export type ProxyType = 
  | 'tcp' 
  | 'udp' 
  | 'http' 
  | 'https' 
  | 'stcp' 
  | 'xtcp' 
  | 'sudp';

export interface ProxyConfig {
  name: string;
  type: ProxyType;
  localIP?: string;
  localPort?: number;
  remotePort?: number;
  customDomains?: string[];
  subdomain?: string;
  locations?: string[];
  httpUser?: string;
  httpPassword?: string;
  useEncryption?: boolean;
  useCompression?: boolean;
  bandwidthLimit?: string;
  metaToken?: string;
  // STCP/XTCP 特有
  secretKey?: string;
  role?: 'visitor' | 'bind';
  serverName?: string;
  // 批量端口
  portRange?: string;
  // 状态
  enabled: boolean;
  createdAt: number;
  updatedAt: number;
}

export interface ServerConfig {
  id: string;
  name: string;
  serverAddr: string;
  serverPort: number;
  authMethod?: 'token' | 'multiuser' | 'oidc';
  token?: string;
  user?: string;
  metaToken?: string;
  tlsEnable?: boolean;
  tlsCertFile?: string;
  tlsKeyFile?: string;
  tlsTrustedCaFile?: string;
  logLevel?: 'debug' | 'info' | 'warn' | 'error';
  logMaxDays?: number;
  adminAddr?: string;
  adminPort?: number;
  adminUser?: string;
  adminPassword?: string;
  // DNS 配置
  dnsServer?: string;
  // QUIC 配置
  quicKeepalivePeriod?: number;
  quicMaxIdleTimeout?: number;
  quicMaxIncomingStreams?: number;
  // 心跳配置
  heartbeatInterval?: number;
  heartbeatTimeout?: number;
  // 状态
  enabled: boolean;
  createdAt: number;
  updatedAt: number;
}

export interface FrpVersion {
  version: string;
  downloadUrl: string;
  platform: string;
  arch: string;
  downloaded: boolean;
  downloadProgress?: number;
}

export interface AppConfig {
  language: 'zh-CN' | 'en-US';
  theme: 'light' | 'dark' | 'auto';
  frpBinaryPath: string;
  configPath: string;
  logPath: string;
  autoStart: boolean;
  minimizeToTray: boolean;
  closeToTray: boolean;
  checkUpdateOnStart: boolean;
  // 下载镜像源
  downloadMirror?: string;
}

export interface LogEntry {
  timestamp: number;
  level: 'debug' | 'info' | 'warn' | 'error';
  message: string;
  source: 'app' | 'frpc';
}

export interface ProcessStatus {
  running: boolean;
  pid?: number;
  startTime?: number;
  exitCode?: number;
  error?: string;
}
