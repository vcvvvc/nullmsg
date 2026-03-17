# Rust 加密货币新闻聚合系统重写计划

## Context

### 学习理念

从零开始，按需添加依赖。每个阶段只引入必要的库，通过实践理解每个工具的作用。

### 项目目标

将 Python 加密货币新闻聚合推送系统重写为 Rust 版本。

### 核心约束

- 主线程运行 HTTP Server（防止平台休眠）
- 后台异步运行 4 个爬虫
- 配置文件：config.toml

---

## 学习路径（渐进式）

### 阶段 1：Rust 基础 - 纯标准库

**目标**：不添加任何外部依赖，用标准库实现配置读取

**当前依赖**：无（仅标准库）

**任务 1.1：手动解析 config.toml**

- 使用 `std::fs::read_to_string()` 读取文件
- 手动解析简单的 `key=value` 格式
- 定义 `Config` 结构体存储配置

**学到什么**：
- Rust 文件 I/O
- `Result<T, E>` 错误处理
- `?` 操作符
- 结构体定义

**验证**：
```bash
cargo run  # 打印读取到的配置
```

---

### 阶段 2：引入 toml 库

**为什么需要**：手动解析 TOML 太复杂，需要专业库

**添加依赖**：
```toml
[dependencies]
toml = "0.8"
serde = { version = "1", features = ["derive"] }
```

**任务 2.1：用 toml 库重写配置读取**

- 添加 `#[derive(Deserialize)]` 到 `Config`
- 使用 `toml::from_str()` 解析

**学到什么**：
- Cargo 添加依赖
- serde 序列化框架
- derive 宏的作用

---

### 阶段 3：实现加密 - base64

**为什么需要**：推送需要 Base64 编码

**添加依赖**：
```toml
base64 = "0.21"
```

**任务 3.1：实现简单的 Base64 编码**

- 创建 `src/crypto.rs`
- 实现 `encode_base64(text: &str) -> String`

**学到什么**：
- 模块系统（mod）
- 字节数组 `&[u8]`
- 外部 crate 使用

---

### 阶段 4：实现加密 - AES

**为什么需要**：推送需要 AES-CBC 加密

**添加依赖**：
```toml
aes = "0.8"
cbc = "0.1"
```

**任务 4.1：实现 AES-CBC 加密**

- 实现 PKCS7 填充
- 实现 `encrypt(text, key, iv) -> String`

**学到什么**：
- 字节操作
- 加密算法集成
- 错误处理进阶

---

### 阶段 5：异步基础 - tokio

**为什么需要**：爬虫需要异步 I/O，HTTP Server 需要并发

**添加依赖**：
```toml
tokio = { version = "1", features = ["rt-multi-thread", "macros", "time"] }
```

**任务 5.1：改造 main 为异步**

- 添加 `#[tokio::main]`
- 实现简单的异步定时器

**学到什么**：
- async/await 语法
- tokio 运行时
- 异步编程基础

---

### 阶段 6：HTTP 客户端 - reqwest

**为什么需要**：爬虫需要发送 HTTP 请求

**添加依赖**：
```toml
reqwest = { version = "0.11", features = ["json"] }
serde_json = "1"
```

**任务 6.1：实现第一个爬虫（BlockBeats）**

- 定义 JSON 响应结构体
- 使用 `reqwest::get()` 获取数据
- 解析 JSON

**学到什么**：
- HTTP 客户端使用
- JSON 反序列化
- `#[derive(Deserialize)]` 实战

---

### 阶段 7：实现推送逻辑

**添加依赖**：无（复用 reqwest）

**任务 7.1：实现 pusher.rs**

- 使用 reqwest POST 发送加密数据
- 实现重试机制

**学到什么**：
- HTTP POST 请求
- 异步重试逻辑
- 模块组合

---

### 阶段 8：HTTP Server

**添加依赖**：
```toml
tokio = { version = "1", features = ["net"] }  # 添加 net feature
```

**任务 8.1：实现简单 HTTP Server**

- 使用 `tokio::net::TcpListener`
- 返回静态 HTML

**学到什么**：
- TCP 编程
- HTTP 协议基础
- tokio 网络编程

---

### 阶段 9：其他爬虫

**按需添加依赖**：
```toml
feed-rs = "1"  # RSS 解析
regex = "1"    # HTML 清理
```

**任务 9.1-9.3：实现其他爬虫**

- Odaily（需要 regex）
- PanNews（字符串处理）
- BWEnews（需要 feed-rs）

---

### 阶段 10：完善与优化

**按需添加**：
```toml
tracing = "*"              # 日志
tracing-subscriber = "*"   # 日志订阅
thiserror = "*"            # 错误处理
async-trait = "*"          # Trait 抽象
```

---

## 当前进度

- [x] 创建项目结构
- [x] 阶段 1：标准库配置读取
- [ ] 阶段 2：引入 toml 库（待解锁）

---

## 验证方法

每个阶段完成后：

1. `cargo check` - 检查编译
2. `cargo run` - 运行程序
3. 观察输出，确认功能正常
