# 🔮 Mini Magentic-UI

智能任务规划和执行系统 - 将自然语言描述转换为可执行的任务计划

## ✨ 功能特性

- 🤖 **智能规划**: 使用阿里云通义千问 AI，将自然语言转换为结构化任务计划
- 🎯 **双模式代理**: 支持 WebSurfer（网络浏览）智能代理
- 🖥️ **多界面支持**: 提供 CLI 命令行界面和 Web 可视化界面
- 🚀 **一键执行**: 自动执行生成的任务计划
- 📊 **实时监控**: 跟踪任务执行状态和进度

## 🏗️ 系统架构

### 后端 (Rust)
- **异步架构**: 基于 Tokio 的高性能异步运行时
- **Web API**: 使用 Axum 构建的 RESTful API 服务
- **AI 集成**: 集成阿里云 DashScope（通义千问）API
- **模块化设计**: 清晰的代码结构和组件分离

## 🚀 快速开始

### 环境要求

- Rust 1.70+
- Node.js 22.19.0
- 阿里云 DashScope API 密钥

### 1. 获取 API 密钥

1. 访问 [阿里云 DashScope 控制台](https://dashscope.console.aliyun.com/)
2. 创建 API 密钥
3. 复制 `sk-` 开头的密钥

### 2. 配置环境

```bash
# 克隆项目
git clone xxx
cd Magentic-mini

# 配置环境变量
echo "DASHSCOPE_API_KEY=sk-your-api-key-here" > .env
```

### 3. 运行应用

#### CLI 模式

```bash
# 构建并运行 CLI
cargo run cli
# 或直接运行（默认为 CLI 模式）
cargo run
```

#### Web 模式

```bash
# 启动 Web 服务器
cargo run web --port 3000
```

然后访问 `http://localhost:3000` 使用 Web 界面。