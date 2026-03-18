# M1 引导式学习笔记

> 教练模式：每步一个概念，你写代码，我审查反馈。

---

## Step 1：结构体定义

### 概念

Rust 用 `struct` 定义数据结构：

```rust
struct 名字 {
    字段名: 类型,
    字段名: 类型,
}
```

拥有所有权的字符串类型是 `String`（不是 `&str`，区别后续讲）。

### 任务

在 `src/main.rs` 的 `fn main()` 上方，定义 `Config` 结构体，包含 4 个 `String` 字段：
- `server_host`
- `server_key`
- `aes_key`
- `aes_iv`

### 验证

```bash
cargo check
```

---

## Step 2：文件 I/O

### 概念

```rust
std::fs::read_to_string("路径")
```

返回 `Result<String, io::Error>`，先用 `.unwrap()` 跑通（后面会改掉）。

### 任务

在 `main()` 里读取 `config.toml`，打印文件内容。

### 验证

```bash
cargo run  # 应该打印出 config.toml 的内容
```

---

## Step 3：Result 和 ? 操作符

### 概念

`Result<T, E>` 是 Rust 的错误处理核心：
- `Ok(value)` — 成功
- `Err(e)` — 失败

`?` 操作符：在返回 `Result` 的函数里，遇到 `Err` 自动提前返回，省去手动 `match`。

函数签名需要改为 `-> Result<Config, String>`。

### 任务

1. 把 `unwrap()` 改成 `?`
2. 把读取逻辑提取到 `fn parse_config(path: &str) -> Result<Config, String>`
3. `main()` 里用 `match` 处理结果

### 验证

```bash
cargo run
```

---

## Step 4：字符串解析

### 概念

- `.lines()` — 按行迭代
- `.trim()` — 去首尾空白
- `.starts_with('#')` — 判断注释行
- `.find('=')` — 找第一个 `=` 的位置
- `&line[..pos]` / `&line[pos+1..]` — 字符串切片

### 任务

在 `parse_config()` 里逐行解析，填入 `Config` 的 4 个字段。

### 验证

```bash
cargo run  # 打印出 4 个字段的值
```

---

## 当前进度

- [x] M1 Step 1：定义 Config 结构体
- [x] M1 Step 2：读取文件（std::fs::read_to_string）
- [x] M1 Step 3：Result + ? 操作符
- [x] M1 Step 4：手动解析 config.toml
- [x] M2：引入 toml + serde，一行替换手动解析
- [x] M3：创建 crypto 模块，实现 encode_base64
- [x] M4：实现 AES-128-CBC 加密（encrypt 函数）

---

## 下一步：M5 - tokio 异步基础

### 为什么需要
爬虫需要异步 I/O，HTTP Server 需要并发处理请求。

### 依赖
```toml
tokio = { version = "1", features = ["rt-multi-thread", "macros", "time"] }
```

### 任务
1. 改造 `main` 为异步：`#[tokio::main]`
2. 实现简单的异步定时器（`tokio::time::sleep`）

### 学到什么
- `async/await` 语法
- tokio 运行时
- 异步编程基础
