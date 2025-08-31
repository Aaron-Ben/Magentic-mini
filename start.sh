#!/bin/bash

# Mini Magentic-UI 启动脚本

echo \"🔮 Welcome to Mini Magentic-UI!\"
echo \"\"

# 检查 .env 文件
if [ ! -f .env ]; then
    echo \"❌ 未找到 .env 文件\"
    echo \"请创建 .env 文件并设置 DASHSCOPE_API_KEY\"
    echo \"示例: echo 'DASHSCOPE_API_KEY=sk-your-api-key' > .env\"
    echo \"\"
    exit 1
fi

# 检查 API 密钥
if ! grep -q \"DASHSCOPE_API_KEY=sk-\" .env; then
    echo \"⚠️  请检查 .env 文件中的 DASHSCOPE_API_KEY 设置\"
    echo \"确保格式为: DASHSCOPE_API_KEY=sk-xxxxxxxxxxxxxxx\"
    echo \"\"
fi

echo \"请选择运行模式:\"
echo \"1) CLI 模式 (命令行界面)\"
echo \"2) Web 模式 (浏览器界面)\"
echo \"3) 帮助\"
echo \"\"
read -p \"请输入选择 (1-3): \" choice

case $choice in
    1)
        echo \"🚀 启动 CLI 模式...\"
        cargo run cli
        ;;
    2)
        echo \"🌐 启动 Web 模式...\"
        echo \"访问 http://localhost:3000 使用 Web 界面\"
        cargo run web --port 3000
        ;;
    3)
        echo \"📖 使用说明:\"
        echo \"\"
        echo \"CLI 模式: 交互式命令行界面，适合快速测试\"
        echo \"Web 模式: 现代化的网页界面，功能更丰富\"
        echo \"\"
        echo \"首次使用请确保:\"
        echo \"1. 已安装 Rust (rustup.rs)\"
        echo \"2. 已配置 .env 文件和 API 密钥\"
        echo \"3. 网络连接正常\"
        echo \"\"
        echo \"获取 API 密钥: https://dashscope.console.aliyun.com/\"
        ;;
    *)
        echo \"❌ 无效选择，退出\"
        exit 1
        ;;
esac