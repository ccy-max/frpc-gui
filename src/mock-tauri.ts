// Tauri API Mock - 用于浏览器预览
// 在浏览器中运行时拦截所有 invoke 调用

const isBrowser = typeof window !== 'undefined' && !('__TAURI_INTERNALS__' in window);

if (isBrowser) {
  console.log('🎭 Running in browser mode with mocked Tauri APIs');

  // Mock @tauri-apps/api/core
  window.__TAURI_MOCK__ = {
    // 模拟 invoke 调用
    invoke: async (cmd, args = {}) => {
      console.log(`[Mock] invoke: ${cmd}`, args);
      
      // 模拟延迟
      await new Promise(resolve => setTimeout(resolve, 100));
      
      switch (cmd) {
        // ===== 设置 =====
        case 'load_settings':
          return {
            language: 'zh-CN',
            theme: 'auto',
            frpc_path: '',
            config_path: '',
            log_path: '',
            auto_start: false,
            minimize_to_tray: true,
            close_to_tray: true,
          };
        
        case 'save_settings':
          return true;
        
        // ===== 配置 =====
        case 'load_config':
          return {
            success: true,
            config: {
              servers: [
                {
                  id: '1',
                  name: '测试服务器',
                  serverAddr: '127.0.0.1',
                  serverPort: 7000,
                  token: 'test123',
                  tlsEnable: false,
                  enabled: true,
                }
              ],
              proxies: [
                {
                  name: 'web-proxy',
                  type: 'tcp',
                  local_ip: '127.0.0.1',
                  local_port: '8080',
                  remote_port: '8080',
                  enabled: true,
                },
                {
                  name: 'ssh-proxy',
                  type: 'tcp',
                  local_ip: '127.0.0.1',
                  local_port: '22',
                  remote_port: '6000',
                  enabled: false,
                }
              ]
            }
          };
        
        case 'save_config':
          return { success: true, config: args.config };
        
        case 'reset_all_config':
          return { success: true };
        
        case 'import_toml_config':
          return { success: true, config: null };
        
        // ===== 进程控制 =====
        case 'start_frp':
          return { success: true };
        
        case 'stop_frp':
          return { success: true };
        
        case 'restart_frp':
          return { success: true };
        
        case 'reload_frp':
          return { success: true };
        
        case 'get_process_status':
          return {
            running: false,
            pid: null,
            state: 'stopped',
          };
        
        case 'detect_frpc_process':
          return false;
        
        case 'modify_proxy_status':
          return true;
        
        // ===== 日志 =====
        case 'get_logs':
          return [
            { timestamp: Date.now() - 60000, level: 'info', message: '应用启动', source: 'app' },
            { timestamp: Date.now() - 30000, level: 'info', message: '配置加载成功', source: 'app' },
          ];
        
        case 'get_frpc_log_content':
          return '';
        
        case 'get_app_log_content':
          return '';
        
        case 'open_frpc_log_file':
          return true;
        
        // ===== 版本管理 =====
        case 'list_frp_versions':
          return [
            {
              version: 'v0.53.0',
              name: 'v0.53.0',
              published_at: '2024-01-15T10:00:00Z',
              download_url: 'https://github.com/fatedier/frp/releases/download/v0.53.0/frp_0.53.0_windows_amd64.zip',
              mirror_url: null,
              size: 15000000,
              download_count: 1000,
              downloaded: false,
            },
            {
              version: 'v0.52.0',
              name: 'v0.52.0',
              published_at: '2023-12-01T10:00:00Z',
              download_url: 'https://github.com/fatedier/frp/releases/download/v0.52.0/frp_0.52.0_windows_amd64.zip',
              mirror_url: null,
              size: 14500000,
              download_count: 2000,
              downloaded: true,
              local_path: 'C:\\frpc\\frpc.exe',
            }
          ];
        
        case 'get_downloaded_versions':
          return [];
        
        case 'download_frp_version':
          return 'C:\\frpc\\frpc.exe';
        
        case 'delete_frp_version':
          return true;
        
        case 'get_mirrors':
          return [
            { name: 'GitHub', url: 'https://github.com' },
            { name: '镜像 1', url: 'https://mirror1.com' },
          ];
        
        case 'import_local_frpc':
          return 'C:\\frpc\\frpc.exe';
        
        // ===== 系统 =====
        case 'check_frpc_exists':
          return false;
        
        case 'get_frpc_version':
          return 'v0.53.0';
        
        case 'open_url':
          return true;
        
        case 'relaunch_app':
          alert('模拟：应用重启');
          return true;
        
        case 'open_app_data':
          alert('模拟：打开数据目录');
          return true;
        
        case 'select_local_file':
          return null;
        
        case 'check_app_update':
          return 'v0.1.0';
        
        case 'get_local_ports':
          return [
            { protocol: 'TCP', ip: '0.0.0.0', port: 80 },
            { protocol: 'TCP', ip: '0.0.0.0', port: 443 },
            { protocol: 'TCP', ip: '127.0.0.1', port: 3306 },
          ];
        
        default:
          console.warn(`[Mock] Unknown command: ${cmd}`);
          return null;
      }
    },
  };

  // 拦截 @tauri-apps/api/core 的导入
  const originalDefine = window.define;
  window.define = function(...args) {
    if (args[0] === '@tauri-apps/api/core') {
      args[1] = function() {
        return { invoke: window.__TAURI_MOCK__.invoke };
      };
    }
    return originalDefine?.apply(this, args);
  };

  // 全局替换 invoke
  window.invoke = window.__TAURI_MOCK__.invoke;
}
