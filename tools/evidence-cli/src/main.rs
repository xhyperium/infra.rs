//! evidence-cli —— SPEC-EVIDENCE-002 §25 只读校验与检查工具。

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use evidence::{
    digest_canonical, genesis_digest, seal_record_v1, verify_chain_records_v1, ChainId, Digest32,
    EventId, EvidenceActor, EvidenceDraft, EvidenceError, EvidenceName, EvidenceOutcome,
    EvidenceReader, OperationId,
};
use evidence_file::FileEvidenceReadOnly;
use kernel::Timestamp;
use serde::Serialize;

/// 退出码（§25.3）。
mod exit {
    pub const OK: u8 = 0;
    pub const BAD_ARGS: u8 = 2;
    pub const CHAIN_INVALID: u8 = 3;
    pub const CHECKPOINT: u8 = 4;
    pub const STORAGE: u8 = 5;
    pub const UNSUPPORTED: u8 = 6;
    pub const REPAIR: u8 = 7;
}

#[derive(Parser, Debug)]
#[command(
    name = "evidence-cli",
    about = "Tamper-evident audit chain tools (read-only by default)"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 验证一条或全部链
    Verify {
        /// evidence 根目录（含 chains/）
        #[arg(long)]
        root: PathBuf,
        /// 可选 chain_id hex（64 字符）
        #[arg(long)]
        chain: Option<String>,
        /// JSON 输出（非 canonical）
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// 打印链头
    Head {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        chain: Option<String>,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// 检查记录（range）
    Inspect {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        chain: String,
        #[arg(long, default_value_t = 1)]
        from: u64,
        #[arg(long, default_value_t = 100)]
        limit: u32,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// 导出摘要列表
    Export {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        chain: String,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// golden / 内置向量
    Vectors {
        #[command(subcommand)]
        cmd: VectorCmd,
    },
    /// 修复不完整尾帧（显式确认）
    RepairTail {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        chain: String,
        /// 必须显式确认
        #[arg(long)]
        confirm: bool,
    },
}

#[derive(Subcommand, Debug)]
enum VectorCmd {
    /// 复算内置 golden 向量
    Verify,
}

#[derive(Serialize)]
struct HeadOut {
    chain_id: String,
    sequence: Option<u64>,
    digest: Option<String>,
    incomplete_tail: bool,
    records: u64,
}

#[derive(Serialize)]
struct InspectRec {
    sequence: u64,
    event_id: String,
    record_digest: String,
    previous_digest: String,
    outcome_tag: u8,
}

fn map_err(e: EvidenceError) -> u8 {
    match e {
        EvidenceError::InvalidName { .. }
        | EvidenceError::InvalidDraft { .. }
        | EvidenceError::InvalidEncoding { .. } => exit::BAD_ARGS,
        EvidenceError::UnsupportedVersion => exit::UNSUPPORTED,
        EvidenceError::StorageUnavailable | EvidenceError::DurabilityFailure => exit::STORAGE,
        EvidenceError::CheckpointMismatch | EvidenceError::SignatureInvalid => exit::CHECKPOINT,
        EvidenceError::SequenceGap
        | EvidenceError::RecordDigestMismatch
        | EvidenceError::PreviousDigestMismatch
        | EvidenceError::ForkDetected
        | EvidenceError::ChainIdMismatch
        | EvidenceError::DuplicateEventId
        | EvidenceError::DuplicateSequence
        | EvidenceError::CorruptStorage
        | EvidenceError::TailTruncated => exit::CHAIN_INVALID,
        _ => exit::CHAIN_INVALID,
    }
}

fn parse_chain(hex: &str) -> Result<ChainId, u8> {
    let d = Digest32::from_hex(hex).map_err(|_| exit::BAD_ARGS)?;
    Ok(ChainId::from_bytes(*d.as_bytes()))
}

fn open_ro(root: &PathBuf) -> Result<FileEvidenceReadOnly, u8> {
    FileEvidenceReadOnly::open(root).map_err(|e| {
        eprintln!("error: {e}");
        map_err(e)
    })
}

fn cmd_verify(root: PathBuf, chain: Option<String>, json: bool) -> u8 {
    let ro = match open_ro(&root) {
        Ok(v) => v,
        Err(c) => return c,
    };
    if ro.chains().is_empty() {
        eprintln!("error: no chains under {}", root.display());
        return exit::STORAGE;
    }
    let mut any_repair = false;
    let targets: Vec<_> = if let Some(h) = chain {
        let cid = match parse_chain(&h) {
            Ok(c) => c,
            Err(c) => return c,
        };
        match ro.get(cid) {
            Some(ch) => vec![ch],
            None => {
                eprintln!("error: chain not found");
                return exit::BAD_ARGS;
            }
        }
    } else {
        ro.chains().iter().collect()
    };

    for ch in targets {
        if ch.incomplete_tail() {
            any_repair = true;
        }
        match verify_chain_records_v1(ch.chain_id(), ch.records(), true, None, None) {
            Ok(rep) => {
                if json {
                    let v = serde_json::json!({
                        "chain_id": Digest32::from_bytes(*ch.chain_id().as_bytes()).to_hex(),
                        "valid": rep.valid(),
                        "records_checked": rep.records_checked(),
                        "incomplete_tail": ch.incomplete_tail(),
                    });
                    println!("{v}");
                } else {
                    println!(
                        "OK chain={} records={} incomplete_tail={}",
                        Digest32::from_bytes(*ch.chain_id().as_bytes()).to_hex(),
                        rep.records_checked(),
                        ch.incomplete_tail()
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "INVALID chain={}: {e}",
                    Digest32::from_bytes(*ch.chain_id().as_bytes()).to_hex()
                );
                return map_err(e);
            }
        }
    }
    if any_repair {
        eprintln!("warning: incomplete tail present; run repair-tail --confirm");
        return exit::REPAIR;
    }
    exit::OK
}

fn cmd_head(root: PathBuf, chain: Option<String>, json: bool) -> u8 {
    let ro = match open_ro(&root) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let list: Vec<_> = if let Some(h) = chain {
        let cid = match parse_chain(&h) {
            Ok(c) => c,
            Err(c) => return c,
        };
        match ro.get(cid) {
            Some(ch) => vec![ch],
            None => {
                eprintln!("error: chain not found");
                return exit::BAD_ARGS;
            }
        }
    } else {
        ro.chains().iter().collect()
    };
    for ch in list {
        let out = HeadOut {
            chain_id: Digest32::from_bytes(*ch.chain_id().as_bytes()).to_hex(),
            sequence: ch.head().map(|h| h.sequence()),
            digest: ch.head().map(|h| h.digest().to_hex()),
            incomplete_tail: ch.incomplete_tail(),
            records: ch.records().len() as u64,
        };
        if json {
            println!("{}", serde_json::to_string(&out).unwrap_or_default());
        } else {
            println!(
                "chain={} seq={:?} digest={:?} records={} incomplete_tail={}",
                out.chain_id, out.sequence, out.digest, out.records, out.incomplete_tail
            );
        }
    }
    exit::OK
}

fn cmd_inspect(root: PathBuf, chain: String, from: u64, limit: u32, json: bool) -> u8 {
    if from < 1 || limit == 0 || limit > 10_000 {
        eprintln!("error: from>=1 and 1<=limit<=10000");
        return exit::BAD_ARGS;
    }
    let ro = match open_ro(&root) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let cid = match parse_chain(&chain) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let Some(ch) = ro.get(cid) else {
        eprintln!("error: chain not found");
        return exit::BAD_ARGS;
    };
    let end = from.saturating_add(u64::from(limit));
    let rows: Vec<InspectRec> = ch
        .records()
        .iter()
        .filter(|r| r.sequence() >= from && r.sequence() < end)
        .map(|r| InspectRec {
            sequence: r.sequence(),
            event_id: r.event_id().as_digest().to_hex(),
            record_digest: r.record_digest().to_hex(),
            previous_digest: r.previous_digest().to_hex(),
            outcome_tag: r.outcome().tag(),
        })
        .collect();
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&rows).unwrap_or_default()
        );
    } else {
        for r in &rows {
            println!(
                "seq={} event={} digest={} prev={} outcome={}",
                r.sequence, r.event_id, r.record_digest, r.previous_digest, r.outcome_tag
            );
        }
    }
    exit::OK
}

fn cmd_export(root: PathBuf, chain: String, json: bool) -> u8 {
    cmd_inspect(root, chain, 1, 10_000, json)
}

fn cmd_vectors_verify() -> u8 {
    // 内置稳定向量：genesis + 一条 Attempted
    let chain = ChainId::from_bytes([0x42; 32]);
    let g = genesis_digest(chain);
    if *g.as_bytes() == [0u8; 32] {
        eprintln!("genesis must not be zero");
        return exit::CHAIN_INVALID;
    }
    let draft = EvidenceDraft::new(
        EventId::from_bytes([0x01; 32]),
        OperationId::from_bytes([0x02; 32]),
        EvidenceName::new("domain_macro").unwrap(),
        EvidenceActor::new(
            EvidenceName::new("service").unwrap(),
            Digest32::from_bytes([0x03; 32]),
        ),
        Digest32::from_bytes([0x04; 32]),
        EvidenceName::new("advance").unwrap(),
        digest_canonical(
            &EvidenceName::new("domain_macro.point.v1").unwrap(),
            b"point",
        ),
        EvidenceOutcome::Attempted,
    );
    let rec = match seal_record_v1(
        chain,
        1,
        Timestamp::from_unix_nanos(1_700_000_000_000_000_000),
        g,
        draft,
    ) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("seal failed: {e}");
            return exit::CHAIN_INVALID;
        }
    };
    match verify_chain_records_v1(chain, &[rec], true, None, None) {
        Ok(rep) if rep.valid() => {
            println!("vectors ok: genesis+attempted");
            exit::OK
        }
        Ok(_) => {
            eprintln!("vectors invalid");
            exit::CHAIN_INVALID
        }
        Err(e) => {
            eprintln!("vectors failed: {e}");
            map_err(e)
        }
    }
}

fn cmd_repair_tail(root: PathBuf, chain: String, confirm: bool) -> u8 {
    if !confirm {
        eprintln!("error: repair-tail requires --confirm (destructive; backs up segment first)");
        return exit::BAD_ARGS;
    }
    let cid = match parse_chain(&chain) {
        Ok(c) => c,
        Err(c) => return c,
    };
    // 只读探测
    let ro = match open_ro(&root) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let Some(ch) = ro.get(cid) else {
        eprintln!("error: chain not found");
        return exit::BAD_ARGS;
    };
    if !ch.incomplete_tail() {
        println!("no incomplete tail; nothing to repair");
        return exit::OK;
    }
    // 写路径：打开 FileEvidenceStore 会在 open_existing 时自动截断尾帧
    // 先备份 segment
    let hex = Digest32::from_bytes(*cid.as_bytes()).to_hex();
    let seg = root.join("chains").join(&hex).join("seg-00000001.xhev");
    let bak = root
        .join("chains")
        .join(&hex)
        .join(format!("seg-00000001.xhev.bak-{}", chrono_like_stamp()));
    if let Err(e) = std::fs::copy(&seg, &bak) {
        eprintln!("error: backup failed: {e}");
        return exit::STORAGE;
    }
    println!("backup: {}", bak.display());
    // 通过写打开触发恢复截断（并占用 writer.lock）
    match evidence_file::FileEvidenceStore::open(&root) {
        Ok(store) => {
            // 触达 chain 以打开 segment
            match store.head(cid) {
                Ok(h) => {
                    println!("repaired; head_seq={:?}", h.map(|x| x.sequence()));
                    // 再只读校验
                    drop(store);
                    let ro2 = match open_ro(&root) {
                        Ok(v) => v,
                        Err(c) => return c,
                    };
                    if let Some(ch2) = ro2.get(cid) {
                        if ch2.incomplete_tail() {
                            eprintln!("error: incomplete tail still present");
                            return exit::REPAIR;
                        }
                        if let Err(e) =
                            verify_chain_records_v1(cid, ch2.records(), true, None, None)
                        {
                            eprintln!("error after repair: {e}");
                            return map_err(e);
                        }
                    }
                    exit::OK
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    map_err(e)
                }
            }
        }
        Err(e) => {
            eprintln!("error opening store (lock?): {e}");
            map_err(e)
        }
    }
}

fn chrono_like_stamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let s = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{s}")
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let code = match cli.cmd {
        Commands::Verify { root, chain, json } => cmd_verify(root, chain, json),
        Commands::Head { root, chain, json } => cmd_head(root, chain, json),
        Commands::Inspect {
            root,
            chain,
            from,
            limit,
            json,
        } => cmd_inspect(root, chain, from, limit, json),
        Commands::Export { root, chain, json } => cmd_export(root, chain, json),
        Commands::Vectors { cmd } => match cmd {
            VectorCmd::Verify => cmd_vectors_verify(),
        },
        Commands::RepairTail {
            root,
            chain,
            confirm,
        } => cmd_repair_tail(root, chain, confirm),
    };
    ExitCode::from(code)
}
