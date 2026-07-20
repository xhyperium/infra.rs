# 量化开发领域规范

> 量化金融 Rust 开发专项要求，与 [CONSTITUTION.md](../CONSTITUTION.md) 通用规范互补。

## 数值精度

禁止浮点存储金融数据，使用 `rust_decimal::Decimal`：

```rust
use rust_decimal::Decimal;
let price = Decimal::new(12345, 2); // 123.45
pub struct Price(Decimal);
pub struct Quantity(Decimal);
```

## 时间戳

```rust
pub struct NanoTimestamp(u64);
```

## 数据处理

推荐 [Polars](https://pola.rs/) 惰性求值 + 并行计算。

## 优化

- 有界通道防溢出
- SoA 缓存友好布局
- `bytes::Bytes` 零拷贝

## Python (PyO3)

Rust 核心 + Python 原型验证。

## 基准测试

关键路径 `criterion` benchmark。

## 依赖

- 优先纯 Rust
- `Cargo.lock` 入库
