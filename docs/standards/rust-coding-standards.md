# Rust 编码规范(完整版)

> 版本:1.0 | 适用:Rust 2021 Edition 及以上
> 本规范涵盖:基础编码规范、异步编程规范、Unsafe 使用准则

---

## 目录

- [第一部分:基础编码规范](#第一部分基础编码规范)
  - [1. 命名规范](#1-命名规范)
  - [2. 代码格式化](#2-代码格式化)
  - [3. 错误处理](#3-错误处理)
  - [4. 所有权与借用](#4-所有权与借用)
  - [5. 类型与抽象](#5-类型与抽象)
  - [6. 代码组织](#6-代码组织)
  - [7. Clippy 静态检查](#7-clippy-静态检查)
  - [8. 注释与文档](#8-注释与文档)
  - [9. 测试规范](#9-测试规范)
  - [10. 其他最佳实践](#10-其他最佳实践)
- [第二部分:异步编程规范](#第二部分异步编程规范)
- [第三部分:Unsafe 使用准则](#第三部分unsafe-使用准则)
- [附录:CI 检查清单](#附录ci-检查清单)

---

# 第一部分:基础编码规范

## 1. 命名规范

遵循官方命名约定(RFC 430):

| 项目 | 规范 | 示例 |
|------|------|------|
| crate | `snake_case` | `serde_json` |
| 模块 (module) | `snake_case` | `mod file_utils` |
| 类型/结构体/枚举 | `UpperCamelCase` | `struct HttpClient` |
| trait | `UpperCamelCase` | `trait Serialize` |
| 枚举变体 | `UpperCamelCase` | `Color::DarkRed` |
| 函数/方法 | `snake_case` | `fn parse_config()` |
| 变量 | `snake_case` | `let user_name` |
| 常量 | `SCREAMING_SNAKE_CASE` | `const MAX_SIZE: u32` |
| 静态变量 | `SCREAMING_SNAKE_CASE` | `static GLOBAL_COUNT` |
| 泛型参数 | 简短大写字母 | `T`, `K`, `V` |
| 生命周期 | 简短小写 | `'a`, `'ctx` |
| 宏 | `snake_case!` | `println!` |

```rust
const MAX_RETRY_COUNT: u32 = 3;

struct UserProfile {
    display_name: String,
    email_address: String,
}

trait DataProcessor {
    fn process_batch(&self, items: &[Item]) -> Result<(), ProcessError>;
}
```

### 命名细节

- 转换方法命名:`as_`(廉价引用转换)、`to_`(昂贵转换)、`into_`(所有权转移)
- getter 不加 `get_` 前缀:用 `fn name()` 而非 `fn get_name()`
- 迭代器方法:`iter()`、`iter_mut()`、`into_iter()`

## 2. 代码格式化

使用 rustfmt 统一格式:

```bash
cargo fmt              # 格式化整个项目
cargo fmt -- --check   # 检查格式(CI 中使用)
```

基本约定(rustfmt 默认):
- 缩进使用 **4 个空格**
- 每行最大宽度 **100 字符**
- 使用 Unix 换行符(LF)

可通过 `rustfmt.toml` 自定义配置:

```toml
max_width = 100
edition = "2021"
```

## 3. 错误处理

### 3.1 优先使用 Result,避免 panic

```rust
// 不好:随意 panic
fn read_config(path: &str) -> Config {
    let content = std::fs::read_to_string(path).unwrap();  // ❌
    parse(&content).expect("parse failed")                  // ❌
}

// 好:返回 Result 让调用者决定
fn read_config(path: &str) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    parse(&content)
}
```

### 3.2 合理使用 `?` 操作符

```rust
fn process() -> Result<Data, MyError> {
    let file = File::open("data.txt")?;
    let parsed = parse_file(file)?;
    Ok(transform(parsed))
}
```

### 3.3 自定义错误类型

- **库代码**:使用 `thiserror`
- **应用代码**:使用 `anyhow`

```rust
// 库中使用 thiserror
#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("解析失败,第 {line} 行")]
    Parse { line: usize },
}

// 应用中使用 anyhow
use anyhow::{Context, Result};

fn load() -> Result<Config> {
    let content = std::fs::read_to_string("config.toml")
        .context("无法读取配置文件")?;
    Ok(toml::from_str(&content)?)
}
```

### 3.4 panic 的合理场景

- 测试代码中的断言
- 程序不变量被违反(逻辑 bug)
- 原型/示例代码(需注释说明)

## 4. 所有权与借用

### 4.1 函数参数优先使用借用

```rust
// 不好:不必要地获取所有权
fn print_name(name: String) { /* ... */ }

// 好:借用即可
fn print_name(name: &str) { /* ... */ }
```

### 4.2 参数类型选择

| 需求 | 推荐类型 |
|------|---------|
| 只读字符串 | `&str` |
| 只读切片 | `&[T]` |
| 需要所有权 | `String` / `Vec<T>` |
| 可能需要所有权 | `impl Into<String>` 或 `Cow<'_, str>` |

### 4.3 避免不必要的 clone

```rust
// 不好:clone 掩盖借用问题
let name = user.name.clone();
process(name);

// 好:先思考是否能借用
process(&user.name);
```

## 5. 类型与抽象

### 5.1 善用类型系统表达约束(newtype 模式)

```rust
// 不好:用基本类型,容易传错参数
fn transfer(from: u64, to: u64, amount: u64) { /* ... */ }

// 好:newtype 模式
struct AccountId(u64);
struct Amount(u64);

fn transfer(from: AccountId, to: AccountId, amount: Amount) { /* ... */ }
```

### 5.2 用枚举替代布尔标志,让无效状态无法表示

```rust
// 不好
struct Connection {
    is_connected: bool,
    is_connecting: bool,  // 可能出现矛盾状态
}

// 好
enum ConnectionState {
    Disconnected,
    Connecting { attempt: u32 },
    Connected { session_id: String },
}
```

### 5.3 实现常用 trait

为公开类型尽量派生:`Debug`、`Clone`、`PartialEq`,按需添加 `Default`、`Hash`、`Serialize` 等。

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(u64);
```

## 6. 代码组织

### 6.1 模块结构

```
src/
├── main.rs          # 入口,保持精简
├── lib.rs           # 库入口,声明模块
├── config.rs        # 单文件模块
├── db/
│   ├── mod.rs       # 或使用 db.rs + db/ 目录
│   ├── models.rs
│   └── queries.rs
└── error.rs         # 统一错误定义
```

### 6.2 可见性原则

- 默认私有,按需 `pub`
- 使用 `pub(crate)` 限制 crate 内可见
- 库的公开 API 尽量精简

## 7. Clippy 静态检查

```bash
cargo clippy                 # 运行 clippy
cargo clippy -- -D warnings  # CI 中严格模式
```

```rust
// 不好
if x == true { /* ... */ }
let v = vec.iter().map(|x| x * 2).collect::<Vec<i32>>();

// 好
if x { /* ... */ }
let v: Vec<i32> = vec.iter().map(|x| x * 2).collect();
```

## 8. 注释与文档

```rust
/// 计算两点之间的欧几里得距离。
///
/// # Examples
///
/// ```
/// let d = distance(&Point::new(0.0, 0.0), &Point::new(3.0, 4.0));
/// assert_eq!(d, 5.0);
/// ```
///
/// # Panics
///
/// 当坐标包含 NaN 时 panic。
pub fn distance(a: &Point, b: &Point) -> f64 { /* ... */ }
```

- `///` 用于文档注释,`//!` 用于模块级文档
- 公开 API 必须有文档,包含 `Examples`、`Errors`、`Panics`、`Safety` 等章节
- 文档中的示例代码会被 `cargo test` 执行(doctest)

## 9. 测试规范

```rust
// 单元测试:与代码同文件
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_input_returns_ok() {
        assert!(parse("valid").is_ok());
    }

    #[test]
    fn parse_empty_input_returns_error() {
        assert!(parse("").is_err());
    }
}
```

- 测试函数名描述场景与预期:`场景_条件_预期结果`
- 集成测试放在 `tests/` 目录
- 使用 `#[should_panic(expected = "...")]` 测试 panic 场景

## 10. 其他最佳实践

### 10.1 迭代器优于手动循环

```rust
let sum: i32 = items.iter().filter(|x| x.active).map(|x| x.value).sum();
```

### 10.2 使用 `#[must_use]` 防止忽略重要返回值

```rust
#[must_use]
pub fn build(self) -> Config { /* ... */ }
```

### 10.3 依赖管理

- 应用程序提交 `Cargo.lock`
- 定期运行 `cargo audit` 检查安全漏洞
- 避免引入功能重复的依赖

---

# 第二部分:异步编程规范

## 1. 运行时选择与约定

### 1.1 统一运行时

一个项目只使用**一个**异步运行时,避免混用:

| 运行时 | 适用场景 |
|--------|---------|
| `tokio` | 事实标准,生态最全,网络服务首选 |
| `async-std` | 已基本停止维护,不推荐新项目 |
| `smol` | 轻量级场景 |
| `embassy` | 嵌入式 |

```toml
# 应用:直接依赖 tokio 全家桶
tokio = { version = "1", features = ["full"] }

# 库:只启用需要的 feature
tokio = { version = "1", features = ["rt", "net", "time"] }
```

### 1.2 库代码尽量保持运行时无关

```rust
// 不好:库代码内部绑死 tokio::spawn
pub async fn process(&self) {
    tokio::spawn(async { /* ... */ });  // ❌ 强迫用户使用 tokio
}

// 好:只写纯 async 逻辑,让调用者决定如何调度
pub async fn process(&self) -> Result<Output, Error> { /* ... */ }
```

## 2. async 函数设计

### 2.1 避免锁跨越 await 点

```rust
// 不好:std MutexGuard 跨越 await,导致 Future 非 Send
async fn bad(data: &std::sync::Mutex<Vec<u32>>) {
    let guard = data.lock().unwrap();
    do_something().await;  // ❌
    guard.push(1);
}

// 好:缩小锁作用域
async fn good(data: &std::sync::Mutex<Vec<u32>>) {
    let value = compute().await;
    data.lock().unwrap().push(value);
}
```

### 2.2 锁的选择

| 场景 | 选择 |
|------|------|
| 临界区短、不跨 await | `std::sync::Mutex`(更快) |
| 必须跨 await 持有锁 | `tokio::sync::Mutex` |
| 读多写少 | `RwLock`(对应版本) |
| 单值广播/更新 | `tokio::sync::watch` |

> **经验法则**:能用 `std::sync::Mutex` 就不用异步锁。

### 2.3 不要在 async 上下文中执行阻塞操作

```rust
// 不好
async fn bad() {
    std::thread::sleep(Duration::from_secs(1));        // ❌
    let data = std::fs::read("big.bin").unwrap();      // ❌
    let result = heavy_cpu_computation();              // ❌
}

// 好
async fn good() -> Result<()> {
    tokio::time::sleep(Duration::from_secs(1)).await;
    let data = tokio::fs::read("big.bin").await?;
    let result = tokio::task::spawn_blocking(|| heavy_cpu_computation()).await?;
    Ok(())
}
```

**判断标准**:可能阻塞超过 10~100 微秒的同步操作,应使用 `spawn_blocking` 或异步替代品。

## 3. 任务管理

### 3.1 避免"野任务"

```rust
// 不好:fire-and-forget,错误被吞掉
tokio::spawn(async { do_work().await; });

// 好:保存 JoinHandle,处理结果
let handle = tokio::spawn(async { do_work().await });
match handle.await {
    Ok(Ok(result)) => { /* ... */ }
    Ok(Err(e)) => tracing::error!("任务失败: {e}"),
    Err(e) if e.is_panic() => tracing::error!("任务 panic"),
    _ => {}
}
```

### 3.2 使用 JoinSet 管理一组任务

```rust
use tokio::task::JoinSet;

let mut set = JoinSet::new();
for item in items {
    set.spawn(process(item));
}
while let Some(res) = set.join_next().await {
    let output = res??;
    // JoinSet drop 时自动 abort 所有剩余任务
}
```

### 3.3 优雅关闭

```rust
use tokio_util::sync::CancellationToken;

let token = CancellationToken::new();
let child = token.child_token();

tokio::spawn(async move {
    loop {
        tokio::select! {
            _ = child.cancelled() => { cleanup().await; break; }
            work = queue.recv() => handle(work).await,
        }
    }
});

tokio::signal::ctrl_c().await?;
token.cancel();
```

## 4. 取消安全(Cancellation Safety)⚠️

**Future 在任何 await 点都可能被 drop(取消)**,这是异步 Rust 最易踩的坑。

### 4.1 `select!` 中的取消陷阱

```rust
// 危险:read_exact 不是取消安全的,取消时已读数据丢失
loop {
    tokio::select! {
        res = socket.read_exact(&mut buf) => { /* ❌ */ }
        _ = interval.tick() => { /* ... */ }
    }
}
```

### 4.2 编写取消安全代码的原则

- 优先使用文档标注 "cancel safe" 的 API
- 需要"取消时必须清理"的逻辑,用 `Drop` 实现清理
- 关键状态变更尽量放在 await 之后,或确保 drop 时能正确回滚

---

# 第三部分:Unsafe 使用准则

## 1. 最小必要原则

- 默认禁止 `unsafe`
- 仅在 safe API 无法表达、且有明确收益时使用
- `unsafe` 块尽量小,把安全检查留在 safe 代码中

## 2. 优先 safe 替代

在写 `unsafe` 前,先评估:

- `bytemuck` / `zerocopy` / `safe_transmute`
- 标准库安全 API
- 成熟 crate 的安全封装

## 3. 文档与注释要求

### 3.1 使用 unsafe 时:论证为什么安全

```rust
// SAFETY: `index` 已在上方通过 `index < self.len` 检查,
// 且 self.data 在 self 生命周期内始终有效。
let value = unsafe { self.data.get_unchecked(index) };
```

注释必须回答:**违反了什么前提会导致 UB,以及为什么这里的前提成立**。

### 3.2 定义 unsafe fn 时:写明调用者义务

```rust
/// 从裸指针构造切片。
///
/// # Safety
///
/// 调用者必须保证:
/// - `ptr` 非空且对齐到 `T`
/// - `ptr` 指向的 `len` 个连续元素已初始化
/// - 在返回的切片生命周期内,内存不被修改或释放
pub unsafe fn slice_from_parts<'a, T>(ptr: *const T, len: usize) -> &'a [T] {
    unsafe { std::slice::from_raw_parts(ptr, len) }
}
```

强制 clippy 检查:

```toml
[lints.clippy]
undocumented_unsafe_blocks = "deny"   # unsafe 块必须有 SAFETY 注释
missing_safety_doc = "deny"           # unsafe fn 必须有 # Safety 文档
```

## 4. 封装原则:安全抽象边界

### 4.1 unsafe 不应泄漏到 API 边界

```rust
// 好:内部使用 unsafe,对外提供安全 API,不变量由私有性保护
pub struct RingBuffer<T> {
    buf: Box<[MaybeUninit<T>]>,
    head: usize,   // 不变量:head, tail 始终 < buf.len()
    tail: usize,
    len: usize,    // 不变量:[tail, tail+len) 范围内元素已初始化
}

impl<T> RingBuffer<T> {
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 { return None; }
        // SAFETY: len > 0 保证 tail 位置元素已初始化(见结构体不变量),
        // 读取后立即更新 tail 与 len,该位置不会被再次读取。
        let value = unsafe { self.buf[self.tail].assume_init_read() };
        self.tail = (self.tail + 1) % self.buf.len();
        self.len -= 1;
        Some(value)
    }
}
```

### 4.2 不变量依赖的字段必须私有

```rust
// 不好:len 是 pub,safe 代码可随意改,导致 unsafe 读取 UB
pub struct Bad<T> {
    pub len: usize,  // ❌
    ptr: *mut T,
}
```

## 5. 常见 UB 场景清单

| 禁止行为 | 说明 |
|---------|------|
| 解引用空指针/悬垂指针 | 包括已释放内存 |
| 数据竞争 | 无同步的并发读写 |
| 违反别名规则 | `&mut` 存在期间通过其他路径访问 |
| 读取未初始化内存 | 用 `MaybeUninit` |
| 构造无效值 | 如 `bool` 为 3、无效 enum 判别值、非 UTF-8 的 `str` |
| 未对齐访问 | 用 `read_unaligned` 处理未对齐指针 |
| `transmute` 滥用 | 优先用 `as`、`from_ne_bytes`、`bytemuck` |

```rust
// 即使"不使用"结果,构造无效值本身就是 UB
let b: bool = unsafe { std::mem::transmute(3u8) };  // ❌ 立即 UB
```

### 裸指针与引用的转换纪律

```rust
// 不好:从 &self 得到 *mut 再写入,违反别名规则
fn bad(&self) {
    let p = self as *const Self as *mut Self;
    unsafe { (*p).field = 1; }  // ❌ UB
}

// 需要内部可变性时用 UnsafeCell
struct Counter { count: UnsafeCell<u64> }
```

## 6. Send / Sync 手动实现准则

```rust
struct MyPtr {
    ptr: *mut Data,  // 裸指针导致自动 !Send !Sync
}

// SAFETY: MyPtr 独占 ptr 指向的堆内存(类似 Box),
// 无内部共享状态,跨线程转移所有权是安全的。
unsafe impl Send for MyPtr {}

// SAFETY: 所有通过 &MyPtr 的操作只读,不存在数据竞争。
unsafe impl Sync for MyPtr {}
```

论证要点:
- `Send`:值移动到另一个线程后使用是否安全?
- `Sync`:多个线程同时持有 `&T` 是否安全?
- 检查所有字段和方法,而不只是"看起来没问题"

## 7. FFI 规范

```rust
use std::os::raw::{c_char, c_int};

extern "C" {
    fn lib_process(data: *const u8, len: usize) -> c_int;
}

// 立即封装为安全 API
pub fn process(data: &[u8]) -> Result<(), LibError> {
    // SAFETY: data.as_ptr() 在调用期间有效,len 与指针匹配;
    // 根据 C 库文档,lib_process 不保留指针、不修改数据。
    let code = unsafe { lib_process(data.as_ptr(), data.len()) };
    if code == 0 { Ok(()) } else { Err(LibError::from_code(code)) }
}
```

FFI 检查清单:
- [ ] 结构体加 `#[repr(C)]`
- [ ] 字符串用 `CString`/`CStr`,注意 NUL 终止符和内部 NUL
- [ ] 明确内存所有权:谁分配谁释放
- [ ] C 回调中的 panic 是 UB,用 `catch_unwind` 拦截
- [ ] 跨 FFI 边界的错误用错误码,不用 `Result`

```rust
extern "C" fn callback(ctx: *mut c_void) {
    let result = std::panic::catch_unwind(|| {
        // 实际逻辑
    });
    if result.is_err() {
        // 记录错误,绝不让 panic 穿越 FFI 边界
    }
}
```

## 8. 验证工具链

unsafe 代码必须用工具验证:

```bash
cargo +nightly miri test                              # Miri:检测 UB(必备)
RUSTFLAGS="-Zsanitizer=address" cargo +nightly test   # ASan:内存错误
RUSTFLAGS="-Zsanitizer=thread" cargo +nightly test    # TSan:数据竞争
cargo geiger                                          # 审计依赖树中的 unsafe
cargo fuzz run parse_target                           # 模糊测试
```

**CI 要求**:包含 unsafe 的 crate,CI 中必须跑 Miri。

## 9. unsafe 审查清单(Code Review 用)

- [ ] 是否真的需要 unsafe?有无 safe 替代(`bytemuck`、`zerocopy`)?
- [ ] unsafe 块是否最小化?
- [ ] 每个 unsafe 块是否有具体的 SAFETY 论证?
- [ ] 每个 unsafe fn 是否有 `# Safety` 文档?
- [ ] 不变量依赖的字段是否全部私有?
- [ ] 是否考虑了 panic 路径(drop guard)?
- [ ] 泛型代码:用户提供的 `T` 的方法 panic 时是否仍安全?
- [ ] 是否通过 Miri 测试?

---

# 附录:CI 检查清单

```bash
cargo fmt -- --check          # 格式检查
cargo clippy -- -D warnings   # lint 检查
cargo test                    # 运行测试(含 doctest)
cargo doc --no-deps           # 文档构建
cargo audit                   # 依赖安全审计
cargo +nightly miri test      # 含 unsafe 的 crate 必须
```

# 附录:推荐阅读

- 基础:《The Rust Programming Language》、Rust API Guidelines
- 异步:《Tokio Tutorial》、Alice Ryhl 博客(actor 模式、shared state)
- unsafe:《The Rustonomicon》、`std` 源码中的 SAFETY 注释
