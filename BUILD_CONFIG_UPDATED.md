# 📝 构建配置已优化

## ✅ 修改内容

已将 GitHub Actions 配置修改为 **只在打标签时构建**：

**修改文件**: `.github/workflows/build.yml`

**触发条件**:
```yaml
on:
  push:
    tags:
      - 'v*'  # 只有创建 v0.1.0, v1.2.3 等标签时才触发
  workflow_dispatch:  # 可以手动触发
```

**移除**:
- ❌ main/master 分支的自动推送触发
- ❌ pull_request 触发

---

## 🎯 使用方法

### 方法 1：打标签触发构建（推荐）

```bash
# 1. 提交代码
git add .
git commit -m "feat: 新功能或修复"
git push origin main

# 2. 打标签
git tag v0.1.0

# 3. 推送标签（这会触发构建！）
git push origin v0.1.0
```

**标签命名规则**:
- ✅ `v0.1.0` - 正确
- ✅ `v1.2.3` - 正确
- ✅ `v0.1.0-beta.1` - 正确（预发布版本）
- ❌ `0.1.0` - 错误（必须有 v 前缀）
- ❌ `release-0.1.0` - 错误

### 方法 2：手动触发构建

1. 访问 https://github.com/ccy-max/frpc-gui/actions/workflows/build.yml
2. 点击右上角 **"Run workflow"** 按钮
3. 选择分支（通常是 `main`）
4. 点击 **"Run workflow"**
5. 等待构建完成

---

## 📊 配额节省对比

### 修改前
- 每次推送到 main 都触发 ❌
- 每天开发提交 10 次 → 10 次构建
- 每次 15 分钟 → 每天 150 分钟
- **每月约 4500 分钟**（超出配额！）

### 修改后 ✅
- 只有打标签时触发
- 每周发布 1-2 次 → 每周 1-2 次构建
- 每次 15 分钟 → 每周 15-30 分钟
- **每月约 60-120 分钟**（节省 95%+！）

---

## 🚀 当前构建状态

当前正在进行的构建：
- **Run #8**: https://github.com/ccy-max/frpc-gui/actions/runs/32704525004
- **状态**: 构建中
- **预计完成**: 10-15 分钟

**这次构建完成后，你将获得可用的 Windows EXE！**

---

## 📥 下载 EXE 的步骤

构建完成后：

1. 访问 https://github.com/ccy-max/frpc-gui/actions/runs/32704525004
2. 等待所有任务变绿 ✅
3. 滚动到页面底部 **Artifacts** 部分
4. 点击 `FRPC-GUI-Windows-EXE` 下载
5. 解压 ZIP 文件
6. 运行 `frpc-gui.exe`

---

## 🏷️ 下次发布流程

当你需要发布新版本时：

```bash
# 1. 确保代码已提交并推送
git add .
git commit -m "fix: 修复问题"
git push origin main

# 2. 打上新标签
git tag v0.1.1  # 修复版本
# 或
git tag v0.2.0  # 新功能版本

# 3. 推送标签（触发构建）
git push origin v0.1.1
```

然后等待 15-20 分钟，构建完成后就可以下载新的 EXE 了！

---

## ⚠️ 注意事项

1. **标签必须推送到远程**才会触发构建
   - `git tag v0.1.0` - 只在本地创建标签
   - `git push origin v0.1.0` - 推送到 GitHub，触发构建 ✅

2. **可以删除旧标签重新发布**
   ```bash
   git tag -d v0.1.0           # 删除本地标签
   git push origin :refs/tags/v0.1.0  # 删除远程标签
   # 修改代码后重新打标签推送
   ```

3. **预发布版本**
   ```bash
   git tag v0.1.0-beta.1
   git push origin v0.1.0-beta.1
   ```

---

**修改完成时间**: 2026-08-24 16:15  
**配置状态**: ✅ 已优化，等待下次标签推送触发构建
