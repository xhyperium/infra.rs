//! 生成仓库根 `STRUCTURE.md`：按架构分层列出 workspace members。
//!
//! # 规则（统一）
//!
//! 1. **分层**：`classify_layer`（与 `lint-deps` 相同）；L0 展示拆为 runtime / test。
//! 2. **Package / Version / Path / Specs 命名对齐**：
//!    - Package = Cargo package 名
//!    - Version = 各 crate `Cargo.toml` `[package].version`（经 `cargo metadata`）
//!    - Path = 物理 crate 路径
//!    - Specs = `.agent/SSOT/` +（去掉 `crates/` 前缀后的 Path）+ `/`
//!      例：`crates/infra/gate` → `.agent/SSOT/infra/gate/`；`tools/evidence` → `.agent/SSOT/tools/evidence/`
//!    - 目录不存在 → `—`（纯文本路径，无 Markdown 链接）
//! 3. **Progress**：`n/3（契约·代码·测试）`（非生产验收）
//! 4. **Quality**：`n/5（读·志·代·述·洁）`（非 code-review / 生产验收）
//! 5. **Notes**：路径/逻辑例外；无则 `—`
//! 6. 权威：`docs/architecture/spec.md` + Approved ADR

use crate::classify::{classify_layer, Layer};
use anyhow::{Context, Result};
use cargo_metadata::MetadataCommand;
use std::{fs, path::Path};

fn section_meta(layer: Layer, rel_path: &str, _pkg_name: &str) -> (u8, &'static str) {
    match layer {
        Layer::Kernel => (0, "L0 Kernel (runtime)"),
        Layer::TestSupport => (1, "T0 Test Support"),
        Layer::Types => (2, "Types"),
        Layer::Contract => (3, "Contracts"),
        Layer::Infra => (4, "L1 Infra"),
        Layer::Storage => (5, "Adapters — Storage"),
        Layer::Exchange => (6, "Adapters — Exchange"),
        Layer::Domain => (7, "L2.5 Domain"),
        Layer::Services => (8, "L2 Services"),
        Layer::Apps => (9, "Apps"),
        Layer::XTask => (10, "Tools"),
        Layer::Legacy => (11, "Legacy"),
        Layer::Unknown => {
            if rel_path.starts_with("tools/") {
                (10, "Tools")
            } else {
                (12, "Unknown")
            }
        }
    }
}

fn package_note(pkg_name: &str, _rel_path: &str, _section: &str) -> &'static str {
    match pkg_name {
        "testkit" => "仅 dev-dependency / 测试图",
        "contract-testkit" => "仅 dev-dependency；contracts trait 契约套件",
        "gate" => "逻辑 L0（非 L1 Infra）",
        "kernel" => "L0 根：error / clock / lifecycle",
        "decimalx" => "package 名 decimalx；路径 types/decimal",
        "domainx" => "package 名 domainx；路径 domain/core",
        "transportx" => "package 名 transportx；路径 infra/transport",
        _ if pkg_name.ends_with('x')
            && matches!(
                pkg_name,
                "redisx"
                    | "kafkax"
                    | "natsx"
                    | "postgresx"
                    | "taosx"
                    | "ossx"
                    | "clickhousex"
                    | "configx"
                    | "observex"
                    | "resiliencx"
                    | "schedulex"
            ) =>
        {
            "package 名带 x 后缀；Path 为物理目录名"
        }
        _ => "",
    }
}

fn package_sort_key(pkg_name: &str) -> (u8, &str) {
    if pkg_name == "kernel" {
        (0, pkg_name)
    } else {
        (1, pkg_name)
    }
}

/// Path → Specs 目录：去掉 `crates/` 前缀后挂到 `.agent/SSOT/`。
fn path_to_spec_dir(crate_rel: &str) -> String {
    let rest = crate_rel.strip_prefix("crates/").unwrap_or(crate_rel);
    format!(".agent/SSOT/{rest}/")
}

fn resolve_spec(root: &Path, crate_rel: &str) -> Option<String> {
    let dir = path_to_spec_dir(crate_rel);
    if root.join(&dir).is_dir() {
        Some(dir)
    } else {
        None
    }
}

fn format_spec_cell(spec: Option<&str>) -> String {
    match spec {
        Some(path) => format!("`{path}`"),
        None => "—".to_string(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Progress {
    has_spec: bool,
    has_code: bool,
    has_test: bool,
}

impl Progress {
    fn score(self) -> u8 {
        u8::from(self.has_spec) + u8::from(self.has_code) + u8::from(self.has_test)
    }

    fn format(self) -> String {
        let mark = |ok: bool| if ok { "✓" } else { "—" };
        format!(
            "{}/3（{}·{}·{}）",
            self.score(),
            mark(self.has_spec),
            mark(self.has_code),
            mark(self.has_test)
        )
    }
}

fn has_source_entry(crate_root: &Path) -> bool {
    crate_root.join("src/lib.rs").is_file() || crate_root.join("src/main.rs").is_file()
}

fn has_tests_dir(crate_root: &Path) -> bool {
    crate_root.join("tests").is_dir()
}

fn has_unit_test_attr(crate_root: &Path) -> bool {
    let src = crate_root.join("src");
    if !src.is_dir() {
        return false;
    }
    walk_rs_for_cfg_test(&src)
}

fn walk_rs_for_cfg_test(dir: &Path) -> bool {
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if walk_rs_for_cfg_test(&path) {
                return true;
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            if let Ok(text) = fs::read_to_string(&path) {
                if text.contains("#[cfg(test)]") {
                    return true;
                }
            }
        }
    }
    false
}

fn package_progress(root: &Path, crate_rel: &str, has_spec: bool) -> Progress {
    let crate_root = root.join(crate_rel);
    Progress {
        has_spec,
        has_code: has_source_entry(&crate_root),
        has_test: has_tests_dir(&crate_root) || has_unit_test_attr(&crate_root),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Quality {
    has_readme: bool,
    has_changelog: bool,
    has_agents: bool,
    has_description: bool,
    is_clean: bool,
}

impl Quality {
    fn score(self) -> u8 {
        u8::from(self.has_readme)
            + u8::from(self.has_changelog)
            + u8::from(self.has_agents)
            + u8::from(self.has_description)
            + u8::from(self.is_clean)
    }

    fn format(self) -> String {
        let mark = |ok: bool| if ok { "✓" } else { "—" };
        format!(
            "{}/5（{}·{}·{}·{}·{}）",
            self.score(),
            mark(self.has_readme),
            mark(self.has_changelog),
            mark(self.has_agents),
            mark(self.has_description),
            mark(self.is_clean)
        )
    }
}

fn cargo_has_description(crate_root: &Path) -> bool {
    let cargo = crate_root.join("Cargo.toml");
    let Ok(text) = fs::read_to_string(cargo) else {
        return false;
    };
    let mut in_package = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_package = t == "[package]";
            continue;
        }
        if in_package && t.starts_with("description") {
            if let Some(rest) = t.split_once('=') {
                let v = rest.1.trim().trim_matches('"').trim_matches('\'');
                return !v.is_empty();
            }
        }
    }
    false
}

fn src_is_clean_of_stubs(crate_root: &Path) -> bool {
    let src = crate_root.join("src");
    if !src.is_dir() {
        return false;
    }
    !walk_rs_for_stub_macros(&src)
}

fn walk_rs_for_stub_macros(dir: &Path) -> bool {
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if walk_rs_for_stub_macros(&path) {
                return true;
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            if let Ok(text) = fs::read_to_string(&path) {
                if text.contains("todo!()")
                    || text.contains("unimplemented!()")
                    || text.contains("todo!(")
                    || text.contains("unimplemented!(")
                    || text.contains("fixme!(")
                {
                    return true;
                }
            }
        }
    }
    false
}

fn package_quality(root: &Path, crate_rel: &str) -> Quality {
    let crate_root = root.join(crate_rel);
    Quality {
        has_readme: crate_root.join("README.md").is_file(),
        has_changelog: crate_root.join("CHANGELOG.md").is_file(),
        has_agents: crate_root.join("AGENTS.md").is_file(),
        has_description: cargo_has_description(&crate_root),
        is_clean: src_is_clean_of_stubs(&crate_root),
    }
}

/// 单行 STRUCTURE 包记录（降低 `type_complexity`）。
struct PackageRow {
    order: u8,
    title: &'static str,
    name: String,
    version: String,
    path: String,
    note: &'static str,
    spec: Option<String>,
    progress: Progress,
    quality: Quality,
}

pub fn render(root: &Path) -> Result<String> {
    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .context("cargo metadata failed")?;

    let mut packages: Vec<PackageRow> = metadata
        .packages
        .iter()
        .map(|p| {
            let abs = p.manifest_path.as_str();
            let path = p
                .manifest_path
                .parent()
                .unwrap()
                .as_std_path()
                .strip_prefix(root)
                .unwrap_or(p.manifest_path.parent().unwrap().as_std_path());
            let rel = path.to_string_lossy().replace('\\', "/");
            let layer = classify_layer(abs);
            let (order, title) = section_meta(layer, &rel, p.name.as_str());
            let note = package_note(p.name.as_str(), &rel, title);
            let spec = resolve_spec(root, &rel);
            let progress = package_progress(root, &rel, spec.is_some());
            let quality = package_quality(root, &rel);
            PackageRow {
                order,
                title,
                name: p.name.to_string(),
                version: p.version.to_string(),
                path: rel,
                note,
                spec,
                progress,
                quality,
            }
        })
        .collect();

    packages.sort_by(|a, b| {
        a.order
            .cmp(&b.order)
            .then_with(|| package_sort_key(&a.name).cmp(&package_sort_key(&b.name)))
            .then_with(|| a.path.cmp(&b.path))
    });

    let total = packages.len();
    let mut out = String::from(
        "# Repository structure\n\n\
         > GENERATED FILE — DO NOT EDIT.\n\
         > Regenerate with `cargo run -p xhyper-xtask -- gen-structure`.\n\
         >\n\
         > **Rules**\n\
         > 1. Layer = `xtask` path classification (same as `lint-deps`); L0 shown as runtime / test.\n\
         > 2. **Package / Version / Path / Specs naming**:\n\
         >    - Package = Cargo package name\n\
         >    - Version = `[package].version` from each crate `Cargo.toml` (via cargo metadata)\n\
         >    - Path = on-disk crate path\n\
         >    - Specs = `.agent/SSOT/` + Path with leading `crates/` stripped + `/`\n\
         >      (plain text path, no markdown link); missing dir → `—`\n\
         > 3. **Progress** = `n/3（契约·代码·测试）` — specs dir, `src/lib.rs|main.rs`, `tests/` or `#[cfg(test)]`.\n\
         > 4. **Quality** = `n/5（读·志·代·述·洁）` — README, CHANGELOG, AGENTS, Cargo description, no stub macros.\n\
         > 5. Not production readiness. Authority: `docs/architecture/spec.md` + Approved ADR.\n\
         > 6. Notes = package-name vs path mismatches / layer exceptions.\n\n",
    );
    out.push_str(&format!("**Workspace members:** {total}\n\n"));

    out.push_str("## Layer summary\n\n");
    out.push_str("| Layer | Count |\n|---|---:|\n");
    {
        let mut i = 0;
        while i < packages.len() {
            let title = packages[i].title;
            let mut j = i + 1;
            while j < packages.len() && packages[j].title == title {
                j += 1;
            }
            out.push_str(&format!("| {title} | {} |\n", j - i));
            i = j;
        }
    }
    out.push('\n');

    let mut i = 0;
    while i < packages.len() {
        let title = packages[i].title;
        let mut j = i + 1;
        while j < packages.len() && packages[j].title == title {
            j += 1;
        }
        out.push_str(&format!("## {title}\n\n"));
        out.push_str(
            "| Package | Version | Path | Specs | Progress | Quality | Notes |\n|---|---|---|---|---|---|---|\n",
        );
        for row in &packages[i..j] {
            let note_cell = if row.note.is_empty() { "—" } else { row.note };
            let spec_cell = format_spec_cell(row.spec.as_deref());
            out.push_str(&format!(
                "| `{}` | `{}` | `{}` | {spec_cell} | {} | {} | {note_cell} |\n",
                row.name,
                row.version,
                row.path,
                row.progress.format(),
                row.quality.format()
            ));
        }
        out.push('\n');
        i = j;
    }

    Ok(out)
}

pub fn run(check: bool) -> Result<()> {
    let root = std::env::current_dir()?;
    let target = root.join("STRUCTURE.md");
    let rendered = render(&root)?;
    if check {
        anyhow::ensure!(
            fs::read_to_string(&target).unwrap_or_default() == rendered,
            "STRUCTURE.md is stale; run gen-structure"
        );
    } else {
        fs::write(target, rendered)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_to_spec_dir_strips_crates_prefix() {
        assert_eq!(
            path_to_spec_dir("crates/infra/gate"),
            ".agent/SSOT/infra/gate/"
        );
        assert_eq!(
            path_to_spec_dir("tools/evidence"),
            ".agent/SSOT/tools/evidence/"
        );
        assert_eq!(path_to_spec_dir("crates/kernel"), ".agent/SSOT/kernel/");
        assert_eq!(
            path_to_spec_dir("crates/adapters/storage/redis"),
            ".agent/SSOT/adapters/storage/redis/"
        );
        assert_eq!(
            path_to_spec_dir("apps/marketd"),
            ".agent/SSOT/apps/marketd/"
        );
    }

    #[test]
    fn resolve_spec_aligned_paths_exist() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root");
        assert_eq!(
            resolve_spec(&root, "crates/kernel").as_deref(),
            Some(".agent/SSOT/kernel/")
        );
        assert_eq!(
            resolve_spec(&root, "crates/infra/gate").as_deref(),
            Some(".agent/SSOT/infra/gate/")
        );
        assert_eq!(
            resolve_spec(&root, "tools/evidence").as_deref(),
            Some(".agent/SSOT/tools/evidence/")
        );
        assert_eq!(
            resolve_spec(&root, "crates/testkit").as_deref(),
            Some(".agent/SSOT/testkit/")
        );
        assert_eq!(resolve_spec(&root, "apps/marketd"), None);
    }

    #[test]
    fn progress_and_quality_format() {
        let p = Progress {
            has_spec: true,
            has_code: true,
            has_test: false,
        };
        assert_eq!(p.format(), "2/3（✓·✓·—）");
        let q = Quality {
            has_readme: true,
            has_changelog: true,
            has_agents: true,
            has_description: true,
            is_clean: true,
        };
        assert_eq!(q.format(), "5/5（✓·✓·✓·✓·✓）");
    }

    #[test]
    fn render_includes_version_column() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root");
        let rendered = render(&root).expect("render STRUCTURE");
        assert!(
            rendered.contains("| Package | Version | Path | Specs | Progress | Quality | Notes |"),
            "missing Version column header"
        );
        assert!(
            rendered.contains("| `xhyper-testkit` | `0.1.1` |"),
            "expected xhyper-testkit version cell from Cargo.toml"
        );
        assert!(
            rendered.contains("| `xhyper-kernel` | `0.1.1` |"),
            "expected xhyper-kernel version cell from Cargo.toml"
        );
        assert!(
            rendered.contains("| `xhyper-contracts` | `0.1.0` |"),
            "expected xhyper-contracts version cell from Cargo.toml"
        );
    }
}
