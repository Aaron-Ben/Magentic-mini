#!/bin/bash

# Python PDF 服务启动脚本

echo "🚀 启动 Python PDF 加载服务..."

# 检查是否安装了依赖
if [ ! -d "venv" ]; then
    echo "📦 创建虚拟环境..."
    python3 -m venv venv
fi

echo "📦 激活虚拟环境..."
source venv/bin/activate

echo "📦 安装依赖..."
pip install -r requirements.txt

echo "✅ 启动服务在 http://localhost:8001"
uvicorn pdf_loader:app --host 0.0.0.0 --port 8001 --reload

