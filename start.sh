# Mini Magentic-UI 启动脚本

echo "🔮 Welcome to Mini Magentic-UI!"
echo ""

export LANG="zh_CN.UTF-8"
export LC_ALL="zh_CN.UTF-8"

# 检查 .env 文件
if [ ! -f .env ]; then
    echo "未找到 .env 文件"
    echo "请创建 .env 文件并设置 DASHSCOPE_API_KEY"
    echo "示例: echo 'DASHSCOPE_API_KEY=sk-your-api-key' > .env"
    echo ""
    exit 1
fi

# 检查 API 密钥
if ! grep -q "DASHSCOPE_API_KEY=sk-" .env; then
    echo "请检查 .env 文件中的 DASHSCOPE_API_KEY 设置"
    echo "确保格式为: DASHSCOPE_API_KEY=sk-xxxxxxxxxxxxxxx"
    echo ""
fi

echo "启动 CLI 模式..."
echo ""
echo "使用说明:"
echo "CLI 模式: 交互式命令行界面，用于浏览器自动化任务"
echo ""
echo "首次使用请确保:"
echo "1. 已安装 Rust (rustup.rs)"
echo "2. 已配置 .env 文件和 API 密钥"
echo "3. 网络连接正常"
echo ""
echo "获取 API 密钥: https://dashscope.console.aliyun.com/"
echo ""

cargo run