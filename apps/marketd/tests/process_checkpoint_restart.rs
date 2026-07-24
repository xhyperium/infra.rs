//! 进程级 FileCheckpoint 重启回归（无外部 provider、无真实 sink）。
//!
//! 证明：完整 OS 进程跑完 fixture 后，同路径 checkpoint 再开进程不会重复 emit。
//! 不覆盖：中途 SIGTERM 竞态、外部真实 sink 幂等、provider testnet。

use std::{
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn run_marketd(checkpoint: &Path) -> (i32, String, String) {
    let bin = env!("CARGO_BIN_EXE_marketd");
    let output =
        Command::new(bin).env("MARKETD_CHECKPOINT", checkpoint).output().expect("spawn marketd");
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (code, stdout, stderr)
}

#[test]
fn process_restart_reopens_checkpoint_without_reemitting() {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    let dir = std::env::temp_dir().join(format!("marketd-process-ckpt-{stamp}"));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let checkpoint = dir.join("marketd.checkpoint");

    let (code1, out1, err1) = run_marketd(&checkpoint);
    assert_eq!(code1, 0, "first process failed: stdout={out1} stderr={err1}");
    assert!(out1.contains("emitted=[1, 3, 4]"), "first process expected unique emits, got: {out1}");
    assert!(out1.contains("committed=4"), "first process expected committed=4, got: {out1}");
    assert!(
        checkpoint.is_file(),
        "checkpoint file missing after first run: {}",
        checkpoint.display()
    );
    let committed = std::fs::read_to_string(&checkpoint)
        .expect("read checkpoint")
        .trim()
        .parse::<u64>()
        .expect("checkpoint u64");
    assert_eq!(committed, 4);

    let (code2, out2, err2) = run_marketd(&checkpoint);
    assert_eq!(code2, 0, "second process failed: stdout={out2} stderr={err2}");
    assert!(out2.contains("emitted=[]"), "restart must not re-emit fixture events, got: {out2}");
    assert!(out2.contains("committed=4"), "restart must retain committed=4, got: {out2}");

    let _ = std::fs::remove_dir_all(&dir);
}
