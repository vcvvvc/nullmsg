# Rust 加密货币新闻聚合系统重写计划

## Context

### 学习理念

从零开始，按需添加依赖。每个阶段只引入必要的库，通过实践理解每个工具的作用。

### 项目目标

将 Python 加密货币新闻聚合推送系统重写为 Rust 版本。

- `main` 分支：Python 基线实现（行为对齐基准）
- `msg_rust` 分支：Rust 重写实现（当前开发分支）
- 不实现心跳保活逻辑（heartbeat 不在本次重写范围）

### 核心约束

- 主线程运行 HTTP Server（防止平台休眠）
- 后台异步运行 4 个爬虫
  ('Odaily', oda.get_news),  # 新闻源 1
  ('PanNews', Pan.get_news),  # 新闻源 2
  ('BWENews', bwe.get_news),  # 新闻源 3
  ('Beats', beat.get_news),
- 配置文件：config.toml
- 以 `main` 行为为验收基准，不做无来源扩展功能