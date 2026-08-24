#!/usr/bin/env node
// 交叉编译后处理脚本 - 拷贝必要的 DLL 文件

import { copyFileSync, mkdirSync, existsSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const projectRoot = join(__dirname, '..');

// 获取目标架构
const target = process.argv[2] || 'x86_64-pc-windows-gnu';
const isCrossCompile = target.includes('windows-gnu') || target.includes('windows-msvc');

if (!isCrossCompile) {
  console.log('Native build, skipping post-processing');
  process.exit(0);
}

const arch = target.startsWith('aarch64') ? 'arm64' : 
             target.startsWith('i686') ? 'x86' : 'x64';

const releaseDir = join(projectRoot, 'src-tauri', 'target', target, 'release');
const bundleDir = join(releaseDir, 'bundle');

// 需要拷贝的文件
const filesToCopy = [
  'WebView2Loader.dll'
];

console.log(`Cross-compile post-processing for ${arch}...`);

// 确保 bundle 目录存在
mkdirSync(bundleDir, { recursive: true });

// 拷贝 DLL 到 bundle 目录
let copied = 0;
for (const file of filesToCopy) {
  const src = join(releaseDir, file);
  const dst = join(bundleDir, file);
  
  if (existsSync(src)) {
    copyFileSync(src, dst);
    console.log(`  ✓ Copied ${file}`);
    copied++;
  } else {
    console.log(`  ✗ Not found: ${file}`);
  }
}

// 也拷贝主 EXE 到 bundle 目录（方便分发）
const exeName = 'frpc-gui.exe';
const exeSrc = join(releaseDir, exeName);
const exeDst = join(bundleDir, exeName);

if (existsSync(exeSrc)) {
  copyFileSync(exeSrc, exeDst);
  console.log(`  ✓ Copied ${exeName}`);
  copied++;
}

console.log(`Post-processing complete. ${copied} files copied to bundle/`);
console.log(`Distribution ready at: ${bundleDir}`);
