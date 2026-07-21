//! 报告写出。

use std::fs;
use std::path::Path;

use crate::types::RunResult;

/// 聚合：当前实现直接返回已有 RunResult（预留多 run 合并）。
#[must_use]
pub fn aggregate_report(run: RunResult) -> RunResult {
    run
}

/// 写 JSON 报告到路径。
pub fn write_report(path: &Path, run: &RunResult) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(run)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(path, json)
}
