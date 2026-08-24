#!/bin/bash
# 快速设置 GitHub Actions 并推送代码

set -e

echo "========================================"
echo "  FRPC GUI - GitHub Actions 快速设置"
echo "========================================"
echo ""

# 检查是否在 git 仓库中
if [ ! -d ".git" ]; then
    echo "📦 初始化 Git 仓库..."
    git init
fi

# 检查 git 配置
if [ -z "$(git config user.name)" ]; then
    echo "⚙️  配置 Git 用户信息..."
    read -p "请输入 Git 用户名：" git_user
    read -p "请输入 Git 邮箱：" git_email
    git config user.name "$git_user"
    git config user.email "$git_email"
fi

# 添加所有文件
echo "📝 添加文件..."
git add .

# 创建初始提交
if [ -z "$(git log --oneline -1 2>/dev/null)" ]; then
    echo "💾 创建初始提交..."
    git commit -m "Initial commit: FRPC GUI with GitHub Actions"
else
    echo "💾 更新提交..."
    git commit -m "Setup GitHub Actions for Windows EXE build"
fi

# 重命名分支为 main
git branch -M main 2>/dev/null || true

echo ""
echo "========================================"
echo "  ✅ 设置完成！"
echo "========================================"
echo ""
echo "下一步操作："
echo ""
echo "1️⃣  在 GitHub 创建新仓库："
echo "   👉 https://github.com/new"
echo "   - 仓库名：frpc-gui"
echo "   - 不要勾选 'Initialize this repository with a README'"
echo ""
echo "2️⃣  将本地仓库连接到 GitHub："
echo "   git remote add origin https://github.com/YOUR_USERNAME/frpc-gui.git"
echo ""
echo "3️⃣  推送代码："
echo "   git push -u origin main"
echo ""
echo "4️⃣  查看构建进度："
echo "   👉 https://github.com/YOUR_USERNAME/frpc-gui/actions"
echo ""
echo "5️⃣  构建完成后下载 EXE："
echo "   - 点击最近的构建"
echo "   - 在页面底部 'Artifacts' 部分下载"
echo "   - 解压得到 FRPC GUI.exe"
echo ""
echo "📖 详细指南请查看：GITHUB_ACTIONS_GUIDE.md"
echo ""
