# Rust 重写计划

## Context

### 项目目标
将 Python 加密货币新闻聚合推送系统重写为 Rust 版本

### 核心约束
- **主线程必须运行 HTTP Server**（伪装为普通网站，防止平台休眠）
- **后台异步运行爬虫**（不阻塞 HTTP 响应）
- **配置文件**：config.toml（已创建）

### 功能模块
1. **配置读取**：server_host, aes_key, aes_iv
2. **加密推送**：AES-CBC + PKCS7 Padding + Base64 → HTTP POST
3. **4个爬虫**：
   - Odaily (JSON, topid: String, 600s)
   - PanNews (JSON, topid: String, 600s)
   - BWEnews (RSS, topid: String, 60s)
   - BlockBeats (JSON, topid: String, 600s)
4. **HTTP Server**：轻量级静态页面服务

### 技术栈
- 异步运行时：tokio
- HTTP 客户端：reqwest
- 加密：aes + base64
- HTTP 服务器：tokio::net::TcpListener（标准库，返回单个 HTML）
- RSS 解析：feed-rs
- 配置：toml

---

## Checklist

### 基础设施
- [ ] 创建 Cargo.toml 和项目结构
- [ ] 实现 config.rs（读取 config.toml）
- [ ] 实现 crypto.rs（AES-CBC 加密 + Base64）
- [ ] 实现 pusher.rs（HTTP POST 推送逻辑）

### 爬虫模块
- [ ] 定义 Crawler trait（统一接口）
- [ ] 实现 odaily.rs 爬虫
- [ ] 实现 pannews.rs 爬虫
- [ ] 实现 bwenews.rs 爬虫
- [ ] 实现 blockbeats.rs 爬虫

### 调度与服务
- [ ] 实现 scheduler.rs（tokio 异步调度）
- [ ] 实现 HTTP Server（main.rs 主线程）
- [ ] 集成测试与验证
