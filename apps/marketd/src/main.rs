//! marketd —— 行情服务二进制入口。
//!
//! 仅负责组装与启动：依赖注入与运行时生命周期见 `composition` 模块。
//! 领域逻辑、交易所协议与存储实现不在本二进制内定义。

mod composition;

#[tokio::main]
async fn main() {
    if let Err(error) = composition::run().await {
        eprintln!("marketd: {error}");
        std::process::exit(1);
    }
}
