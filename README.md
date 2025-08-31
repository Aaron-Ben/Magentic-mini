# 🔮 Mini Magentic-UI

智能任务规划和执行系统 - 将自然语言描述转换为可执行的任务计划

## ✨ 功能特性

- 🤖 **智能规划**: 使用阿里云通义千问 AI，将自然语言转换为结构化任务计划
- 🎯 **双模式代理**: 支持 WebSurfer（网络浏览）和 Coder（编程）两种智能代理
- 🖥️ **多界面支持**: 提供 CLI 命令行界面和 Web 可视化界面
- 🚀 **一键执行**: 自动执行生成的任务计划
- 📊 **实时监控**: 跟踪任务执行状态和进度

## 🏗️ 系统架构

### 后端 (Rust)
- **异步架构**: 基于 Tokio 的高性能异步运行时
- **Web API**: 使用 Axum 构建的 RESTful API 服务
- **AI 集成**: 集成阿里云 DashScope（通义千问）API
- **模块化设计**: 清晰的代码结构和组件分离

### 前端 (React + TypeScript)
- **现代化界面**: 响应式设计，支持桌面和移动端
- **实时交互**: 直观的任务创建和执行界面
- **状态可视化**: 清晰的步骤状态和进度显示
- **类型安全**: 完整的 TypeScript 类型定义

## 🚀 快速开始

### 环境要求

- Rust 1.70+
- Node.js 18+ (可选，用于前端开发)
- 阿里云 DashScope API 密钥

### 1. 获取 API 密钥

1. 访问 [阿里云 DashScope 控制台](https://dashscope.console.aliyun.com/)
2. 创建 API 密钥
3. 复制 `sk-` 开头的密钥

### 2. 配置环境

```bash
# 克隆项目
git clone <repository-url>
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

### 4. 前端开发（可选）

如果需要修改前端界面：

```bash
cd frontend

# 安装依赖
npm install

# 启动开发服务器
npm run dev

# 构建生产版本
npm run build
```

## 📖 使用方法

### CLI 界面

1. 启动程序后，输入你想完成的任务
2. 系统会生成详细的执行计划
3. 确认后开始执行计划
4. 查看每个步骤的执行结果

### Web 界面

1. 在输入框中描述你的任务
2. 点击"生成计划"按钮
3. 查看生成的执行步骤
4. 点击"执行计划"开始执行
5. 实时查看执行状态

### 示例任务

- "我想学习 React 框架"
- "帮我制作一个个人网站"
- "我需要分析销售数据"
- "创建一个 API 服务器"

## 🛠️ API 文档

### 创建计划
```http
POST /api/plans
Content-Type: application/json

{
  "user_input": "我想学习 React"
}
```

### 获取计划
```http
GET /api/plans/{plan_id}
```

### 执行计划
```http
POST /api/plans/{plan_id}/execute
```

### 健康检查
```http
GET /api/health
```

## 🏛️ 项目结构

```
.
├── src/
│   ├── agents/           # 智能代理模块
│   │   ├── orchestrator.rs  # 任务编排器
│   │   └── mod.rs
│   ├── cli/              # CLI 界面模块
│   │   ├── interface.rs     # 交互界面
│   │   └── mod.rs
│   ├── llm/              # LLM 客户端模块
│   │   ├── client.rs        # DashScope 客户端
│   │   └── mod.rs
│   ├── routes/           # Web API 路由
│   │   ├── plan.rs          # 计划相关 API
│   │   └── mod.rs
│   ├── types/            # 类型定义模块
│   │   ├── message.rs       # 消息类型
│   │   ├── plan.rs          # 计划类型
│   │   └── mod.rs
│   └── main.rs           # 程序入口
├── frontend/             # React 前端
│   ├── src/
│   │   ├── components/      # React 组件
│   │   ├── App.tsx         # 主应用组件
│   │   ├── api.ts          # API 客户端
│   │   ├── types.ts        # 类型定义
│   │   └── main.tsx        # 入口文件
│   ├── dist/               # 构建产物
│   └── package.json
├── Cargo.toml            # Rust 依赖配置
└── README.md
```

## 🤝 贡献指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 打开 Pull Request

## 📝 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🆘 故障排除

### 常见问题

1. **API 密钥错误**
   - 确保 `DASHSCOPE_API_KEY` 环境变量正确设置
   - 验证密钥格式为 `sk-` 开头

2. **编译错误**
   - 确保 Rust 版本 1.70+
   - 运行 `cargo clean` 清理缓存

3. **前端访问失败**
   - 确保后端服务器正在运行
   - 检查端口是否被占用

4. **网络连接问题**
   - 检查网络连接
   - 确认能访问阿里云 DashScope API

如有其他问题，请在 Issues 中反馈。