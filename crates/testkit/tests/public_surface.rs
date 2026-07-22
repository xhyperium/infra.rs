//! 公开面与生产图轻量守卫（SPEC-TESTKIT-002 §3.1 / §13.5 / §24.2 / §24.5 core）。
//!
//! 用 `include_str!` 嵌入清单与源码，避免运行时 FS（Miri isolation 友好）。

/// 去掉 `//` 行注释与 `/* */` 块注释，避免文档叙述触发假阳性。
fn strip_rust_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let bytes = src.as_bytes();
    let mut i = 0;
    let mut in_block = false;
    let mut in_line = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_line {
            if b == b'\n' {
                in_line = false;
                out.push('\n');
            }
            i += 1;
            continue;
        }
        if in_block {
            if b == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                in_block = false;
                i += 2;
                continue;
            }
            if b == b'\n' {
                out.push('\n');
            }
            i += 1;
            continue;
        }
        if b == b'/' && i + 1 < bytes.len() {
            match bytes[i + 1] {
                b'/' => {
                    in_line = true;
                    i += 2;
                    continue;
                }
                b'*' => {
                    in_block = true;
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }
        out.push(b as char);
        i += 1;
    }
    out
}

const LIB_RS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"));
const CLOCK_RS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/clock.rs"));
const HARNESS_RS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/harness.rs"));
const CARGO_TOML: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));

#[test]
fn lib_reexports_only_manual_clock_family() {
    let code = strip_rust_comments(LIB_RS);
    assert!(
        code.contains("ManualClock")
            && code.contains("ManualClockError")
            && code.contains("ManualClockFault")
            && code.contains("ManualClockSnapshot"),
        "lib.rs must re-export ManualClock family"
    );
    assert!(
        code.contains("IntegrationHarness") && code.contains("StepRecord"),
        "lib.rs must re-export IntegrationHarness family"
    );
    for banned in [
        "macro_rules! xlib_test",
        "macro_rules! mock",
        "struct FixtureBuilder",
        "provider_capability_contract_tests!",
    ] {
        assert!(
            !code.contains(banned),
            "lib.rs code must not contain retired definition `{banned}`"
        );
    }
    let pub_uses: Vec<&str> =
        code.lines().map(str::trim).filter(|l| l.starts_with("pub use")).collect();
    assert_eq!(pub_uses.len(), 2, "expected clock + harness pub use lines, got {pub_uses:?}");
    let clock_line = pub_uses.iter().find(|l| l.contains("clock::")).expect("clock pub use");
    for name in ["ManualClock", "ManualClockError", "ManualClockFault", "ManualClockSnapshot"] {
        assert!(clock_line.contains(name), "pub use missing {name}: {clock_line}");
    }
    let harness_line = pub_uses.iter().find(|l| l.contains("harness::")).expect("harness pub use");
    assert!(
        harness_line.contains("IntegrationHarness") && harness_line.contains("StepRecord"),
        "harness pub use missing symbols: {harness_line}"
    );
}

#[test]
fn source_tree_has_no_retired_macros_or_fixture_builder() {
    let sources = [("lib.rs", LIB_RS), ("clock.rs", CLOCK_RS), ("harness.rs", HARNESS_RS)];
    let mut saw_manual_clock_struct = false;
    for (name, text) in sources {
        let code = strip_rust_comments(text);
        for banned in [
            "macro_rules! xlib_test",
            "macro_rules! mock",
            "struct FixtureBuilder",
            "provider_capability_contract_tests!",
            "impl Default for ManualClock",
            "impl Clone for ManualClock",
        ] {
            assert!(!code.contains(banned), "{name} must not reintroduce `{banned}`");
        }
        let lines: Vec<&str> = code.lines().map(str::trim).collect();
        for (idx, line) in lines.iter().enumerate() {
            if line.starts_with("pub struct ManualClock {") {
                saw_manual_clock_struct = true;
                for prev in lines[..idx].iter().rev() {
                    if prev.is_empty() {
                        continue;
                    }
                    if prev.starts_with("#[derive") {
                        assert!(
                            !prev.contains("Clone") && !prev.contains("Default"),
                            "ManualClock derive must not include Clone/Default: {prev}"
                        );
                        break;
                    }
                    if !prev.starts_with('#') {
                        break;
                    }
                }
            }
        }
    }
    assert!(saw_manual_clock_struct, "did not find `pub struct ManualClock` in source tree");
}

#[test]
fn cargo_toml_production_deps_only_kernel() {
    let deps_section = CARGO_TOML
        .split("[dependencies]")
        .nth(1)
        .expect("[dependencies] section")
        .split('[')
        .next()
        .expect("section body");
    let mut prod_deps: Vec<&str> = deps_section
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split('=').next().map(str::trim))
        .filter(|k| !k.is_empty())
        .collect();
    prod_deps.sort_unstable();
    assert_eq!(prod_deps, vec!["kernel"], "production deps must be only kernel, got {prod_deps:?}");
    assert!(
        CARGO_TOML.contains("publish = false"),
        "testkit must remain publish=false (test-support plane)"
    );
    assert!(
        CARGO_TOML.contains("default = []") || CARGO_TOML.contains("default=[]"),
        "features.default must be empty"
    );
}

#[test]
fn consumer_can_import_manual_clock_family() {
    use std::time::Duration;
    use testkit::{
        IntegrationHarness, ManualClock, ManualClockError, ManualClockFault, ManualClockSnapshot,
        StepRecord,
    };

    let c = ManualClock::new(kernel::Timestamp::from_unix_nanos(7));
    let _: ManualClockError = ManualClockError::WallOverflow;
    let _: ManualClockFault = ManualClockFault::Unavailable;
    let snap: ManualClockSnapshot = c.snapshot().expect("snap");
    assert_eq!(snap.wall().as_unix_nanos(), 7);

    let mut h = IntegrationHarness::with_wall(kernel::Timestamp::from_unix_nanos(0));
    h.step_advance_wall("t", Duration::from_nanos(1));
    let rec: &[StepRecord] = h.run().expect("run");
    assert_eq!(rec.len(), 1);
}
