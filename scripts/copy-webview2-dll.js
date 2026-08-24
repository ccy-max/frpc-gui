#!/usr/bin/env node
// 拷贝 WebView2Loader.dll 到 bundle 目录
// 用于交叉编译 Windows 版本

import { copyFileSync, existsSync, mkdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const projectRoot = join(__dirname, '..');
const srcTauriDir = join(projectRoot, 'src-tauri');

// 检测目标架构
const target = process.env.TARGET || 'x86_64-pc-windows-gnu';
const arch = target.includes('aarch64') ? 'arm64' : 
             target.includes('i686') ? 'x86' : 'x64';

const profile = process.env.PROFILE || 'release';
const targetDir = join(srcTauriDir, 'target', target, profile);
const bundleDir = join(targetDir, 'bundle');

// WebView2Loader.dll 源路径（由 webview2-com-sys 构建生成）
const webview2Dir = join(targetDir, `build/webview2-com-sys-*/out/${arch}`);
const glob = await import('glob');
const dllFiles = glob.sync(webview2Dir + '/WebView2Loader.dll');

if (dllFiles.length === 0) {
  console.error('❌ WebView2Loader.dll not found!');
  console.error(`Searched: ${webview2Dir}`);
  process.exit(1);
}

const dllSource = dllFiles[0];
const dllDest = join(bundleDir, 'WebView2Loader.dll');

// 确保 bundle 目录存在
mkdirSync(bundleDir, { recursive: true });

// 拷贝 DLL
copyFileSync(dllSource, dllDest);
console.log(`✅ Copied WebView2Loader.dll to ${dllDest}`);

// 也拷贝到 release 目录（如果还没有）
const releaseDest = join(targetDir, 'WebView2Loader.dll');
if (!existsSync(releaseDest)) {
  copyFileSync(dllSource, releaseDest);
  console.log(`✅ Copied WebView2Loader.dll to ${releaseDest}`);
}

console.log('✅ WebView2Loader.dll deployment complete');
