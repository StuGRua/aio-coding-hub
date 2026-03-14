<div align="center">
  <img src="public/logo.jpg" width="120" alt="AIO Coding Hub Logo" />

# AIO Coding Hub

**本地 AI CLI 统一网关** — 让 Claude Code / Codex / Gemini CLI 请求走同一个入口

[![Release](https://img.shields.io/github/v/release/dyndynjyxa/aio-coding-hub?style=flat-square)](https://github.com/dyndynjyxa/aio-coding-hub/releases)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20macOS%20|%20Linux-lightgrey?style=flat-square)](#安装)

简体中文 | [English](./README_EN.md)

</div>

> **致谢** — 本项目借鉴了 [cc-switch](https://github.com/farion1231/cc-switch)、[claude-code-hub](https://github.com/ding113/claude-code-hub)、[code-switch-R](https://github.com/Rogers-F/code-switch-R) 等优秀开源项目。

---

## 为什么需要它？

| 痛点 | AIO Coding Hub 的解决方案 |
|------|--------------------------|
| 每个 CLI 都要单独配置 API | **统一网关** — 所有 CLI 走 `127.0.0.1` 本机入口 |
| 上游不稳定时请求失败 | **智能 Failover** — 自动切换供应商，熔断保护 |
| 不知道用了多少 Token 和花了多少钱 | **全链路可观测** — Trace 追踪、用量统计、花费估算 |
| 不同项目需要不同的 Prompts / MCP 配置 | **工作区隔离** — 按项目管理 CLI 配置，一键切换 |

---

## 产品截图

### 首页 — 热力图、用量趋势、活跃 Session、请求日志

![首页](public/screenshots/home.png)

### 用量 — Token 统计、缓存命中率、耗时、花费排行

![用量](public/screenshots/usage.png)

### 模型验证 — 多维度渠道鉴别与供应商验证

![模型验证](public/screenshots/modelValidate.png)

---

## 核心功能

### 网关代理

- 单一入口代理 Claude Code / Codex / Gemini CLI 请求
- 自定义模型名称映射
- SSE / JSON 响应自动修复

### 智能路由与容错

- 多供应商优先级排序 + 自动故障转移
- 熔断器模式（可配置阈值与恢复时间）
- Sticky Session 保持会话粘滞

### 用量与可观测

- Token 用量统计（按 CLI / 供应商 / 模型维度）
- 花费估算 + 模型价格自动同步
- 请求 Trace 与实时控制台日志
- 热力图与缓存趋势图

### 工作区管理

- 按项目隔离 Prompts、MCP、Skill 配置
- 工作区对比、克隆、切换与回滚
- 配置自动同步到各 CLI

### Skill 市场

- 从 Git 仓库发现并安装 Skill
- 仓库管理、过滤、排序
- 关联工作区批量管理

### CLI 管理

- Claude Code 设置直接编辑
- Codex config.toml 代码编辑器
- 环境变量冲突检测
- 本地 Session 历史浏览（项目 → 会话 → 消息）

### 模型验证

- 多维度验证模板（Token 截断、Extended Thinking 等）
- 跨供应商签名验证
- 批量验证 + 历史记录

### 其他

- 自动更新、开机自启、单实例
- 数据导入 / 导出 / 清空
- WSL 环境支持

---

## 安装

### 从 Release 下载（推荐）

前往 [Releases](https://github.com/dyndynjyxa/aio-coding-hub/releases) 下载对应平台安装包：

| 平台 | 安装包 |
|------|--------|
| **Windows** | `.exe` (NSIS) 或 `.msi` |
| **macOS** | `.dmg` |
| **Linux** | `.deb` / `.AppImage` |

<details>
<summary>macOS 安全提示</summary>

若遇到"无法打开 / 来源未验证"提示：

```bash
sudo xattr -cr /Applications/"AIO Coding Hub.app"
```

</details>

### 从源码构建

<details>
<summary>前置条件</summary>

**通用要求：** Node.js 18+、pnpm、Rust 1.90+

**Windows：** [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)（勾选"使用 C++ 的桌面开发"）

**macOS：** `xcode-select --install`

**Linux (Ubuntu/Debian)：**
```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

</details>

```bash
git clone https://github.com/dyndynjyxa/aio-coding-hub.git
cd aio-coding-hub
pnpm install

# 开发模式
pnpm tauri:dev

# 构建（当前平台）
pnpm tauri:build

# 指定平台
pnpm tauri:build:mac:arm64       # macOS Apple Silicon
pnpm tauri:build:mac:x64         # macOS Intel
pnpm tauri:build:mac:universal   # macOS Universal
pnpm tauri:build:win:x64         # Windows x64
pnpm tauri:build:win:arm64       # Windows ARM64
pnpm tauri:build:linux:x64       # Linux x64
```

---

## 快速开始

```
1. 供应商页 → 添加上游（官方 API / 自建代理 / 公司网关）
2. 首页 → 打开目标 CLI 的"代理"开关
3. 终端发起请求 → 在控制台 / 用量页查看 Trace 与统计
```

验证网关运行：

```bash
curl http://127.0.0.1:37123/health
# {"status":"ok"}
```

---

## 技术栈

| 层级 | 技术 |
|------|------|
| **前端** | React 19 · TypeScript · Tailwind CSS · Vite |
| **状态管理** | TanStack Query · React Hooks |
| **桌面框架** | Tauri 2 |
| **后端** | Rust · Axum (HTTP Gateway) |
| **数据库** | SQLite (rusqlite) |
| **测试** | Vitest · Testing Library · MSW · Cargo Test |

---

## 质量保证

```bash
pnpm check:precommit       # 快速预提交检查（前端 + Rust check）
pnpm check:precommit:full  # 完整检查（格式 + clippy）
pnpm check:prepush         # 覆盖率 + 后端测试 + clippy
pnpm test:unit              # 前端单元测试
pnpm tauri:test             # 后端测试
```

---

## 不适用场景

- 公网部署 / 远程访问 / 多租户
- 企业级 RBAC 权限管理

> 本项目定位为 **单机桌面工具 + 本地网关**，所有数据保存在本机。

---

## 参与贡献

欢迎提交 Issue 和 PR！采用 [Conventional Commits](https://www.conventionalcommits.org/) 规范。

```bash
feat(ui): add usage heatmap
fix(gateway): handle timeout correctly
docs: update installation guide
```

---

## 许可证

[MIT License](LICENSE)

---

[![Stargazers over time](https://starchart.cc/dyndynjyxa/aio-coding-hub.svg?variant=adaptive)](https://starchart.cc/dyndynjyxa/aio-coding-hub)
