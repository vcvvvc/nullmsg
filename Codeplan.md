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

---

## 下一步：M4 - AES-CBC 加密

### 概念
- PKCS7 填充
- AES-CBC 加密模式
- 字节数组操作 `&[u8]`

### 依赖
```toml
aes = "0.8"
cbc = "0.1"
```

### 任务
在 `src/crypto.rs` 里实现：
```rust
pub fn encrypt(text: &str, key: &str, iv: &str) -> String
```

返回 base64 编码的加密结果。
