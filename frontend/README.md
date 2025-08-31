# Mini Magentic-UI Frontend

这是 Mini Magentic-UI 的 React + TypeScript 前端界面。

## 功能特性

- 🎯 **智能任务规划**: 输入自然语言描述，AI 自动生成执行计划
- 🚀 **一键执行**: 可视化执行计划中的每个步骤
- 🔄 **实时状态**: 显示每个步骤的执行状态
- 🎨 **现代化界面**: 响应式设计，支持移动端
- 🌐 **双模式代理**: 支持 WebSurfer（网络浏览）和 Coder（编程）代理

## 技术栈

- **React 18** - 用户界面框架
- **TypeScript** - 类型安全的 JavaScript
- **Vite** - 快速的构建工具
- **Axios** - HTTP 客户端
- **CSS-in-JS** - 内联样式组件

## 开发环境要求

- Node.js 18+
- npm 或 yarn

## 安装和运行

### 1. 安装依赖

```bash
npm install
# 或
yarn install
```

### 2. 启动开发服务器

```bash
npm run dev
# 或
yarn dev
```

前端将运行在 `http://localhost:5173`

### 3. 构建生产版本

```bash
npm run build
# 或
yarn build
```

构建产物将生成在 `dist` 目录中。

## 与后端集成

确保后端服务器运行在 `http://localhost:3000`，前端会自动代理 API 请求。

启动完整应用：

1. 启动后端：
   ```bash
   cd ..
   cargo run web --port 3000
   ```

2. 启动前端：
   ```bash
   cd frontend
   npm run dev
   ```

3. 访问 `http://localhost:5173` 使用应用

## API 接口

前端与以下后端 API 交互：

- `POST /api/plans` - 创建新计划
- `GET /api/plans/:id` - 获取计划详情
- `POST /api/plans/:id/execute` - 执行计划
- `GET /api/health` - 健康检查

## 组件结构

```
src/
├── App.tsx           # 主应用组件
├── main.tsx          # 应用入口
├── api.ts            # API 客户端
├── types.ts          # TypeScript 类型定义
└── components/
    └── PlanStep.tsx  # 计划步骤组件
```

## 样式设计

- 使用渐变背景和现代化的卡片设计
- 响应式布局，适配各种屏幕尺寸
- 直观的状态指示器和图标
- 流畅的交互动画和过渡效果"