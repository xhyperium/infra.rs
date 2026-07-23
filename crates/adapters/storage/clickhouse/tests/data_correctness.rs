//! ClickHouse 数据正确性测试。
//!
//! ```text
//! cargo test -p clickhousex --test data_correctness -- --nocapture
//! ```

use clickhousex::{ClickHouseConfig, ClickHousePool};
use serde_json::Value;

const TEST_PASSWORD: &str = "iCEOuptIx40EduvGOKX73rfY";
const TEST_DATABASE: &str = "infra_draft_data_correctness";

fn default_config() -> ClickHouseConfig {
    ClickHouseConfig {
        host: "127.0.0.1".into(),
        http_port: 8123,
        password: TEST_PASSWORD.into(),
        database: "default".into(),
        ..ClickHouseConfig::default()
    }
}

async fn setup_db() -> ClickHousePool {
    let mut cfg = default_config();
    cfg.database = "default".into();
    let pool = ClickHousePool::connect(cfg).await.expect("连接 default 数据库");
    pool.execute(&format!("CREATE DATABASE IF NOT EXISTS {TEST_DATABASE}"))
        .await
        .expect("创建测试数据库");
    pool.close().await.expect("关闭");

    ClickHousePool::connect(default_config()).await.expect("连接到测试数据库")
}

// ── 测试 1：类型映射正确性 ──────────────────────────────────────

#[tokio::test]
async fn type_mapping_correctness() {
    let pool = setup_db().await;
    let pid = std::process::id();
    let table = format!("types_test_{pid}");

    // 创建包含多种类型的表
    let ddl = format!(
        "CREATE TABLE IF NOT EXISTS {table} (\
           col_int8 Int8,\
           col_int16 Int16,\
           col_int32 Int32,\
           col_int64 Int64,\
           col_uint8 UInt8,\
           col_uint16 UInt16,\
           col_uint32 UInt32,\
           col_uint64 UInt64,\
           col_float32 Float32,\
           col_float64 Float64,\
           col_decimal Decimal64(4),\
           col_str Nullable(String),\
           col_arr_str Array(String),\
           col_arr_int Array(Int64),\
           col_uuid UUID,\
           col_long_str String,\
           col_unicode String,\
           col_special String\
         ) ENGINE = MergeTree ORDER BY col_int32"
    );
    pool.execute(&ddl).await.expect("创建类型映射表");

    // 构造超长字符串（100KB）
    let long_str = "A".repeat(100 * 1024);

    // 构造特殊字符字符串
    let special_chars = "tab\there\nnewline\nquote\"backslash\\null\0end";

    // 插入边界值数据
    let row = serde_json::json!({
        "col_int8": 127i8,
        "col_int16": 32767i16,
        "col_int32": 2147483647i32,
        "col_int64": 9223372036854775807i64,
        "col_uint8": 255u8,
        "col_uint16": 65535u16,
        "col_uint32": 4294967295u32,
        "col_uint64": 18446744073709551615u64,
        "col_float32": std::f32::consts::PI,
        "col_float64": std::f64::consts::E,
        "col_decimal": 1234.5678f64,
        "col_str": "hello nullable",
        "col_arr_str": ["a", "b", "c"],
        "col_arr_int": [1i64, 2, 3],
        "col_uuid": "550e8400-e29b-41d4-a716-446655440000",
        "col_long_str": long_str,
        "col_unicode": "你好世界🌍 emoji test",
        "col_special": special_chars,
    });
    pool.insert_json_each_row(&table, &[row]).await.expect("插入类型测试数据");

    // 查询并验证各字段
    let sql = format!(
        "SELECT \
           col_int8, col_int16, col_int32, col_int64,\
           col_uint8, col_uint16, col_uint32, col_uint64,\
           col_float32, col_float64, col_decimal,\
           col_str, col_arr_str, col_arr_int, col_uuid,\
           col_long_str, col_unicode, col_special\
         FROM {table} FORMAT TabSeparated"
    );
    let rows = pool.query_rows(&sql).await.expect("查询类型测试数据");
    assert_eq!(rows.len(), 1, "应查到 1 行");
    let r = &rows[0];
    assert_eq!(r.len(), 18, "应有 18 列");

    // 验证整型
    assert_eq!(r[0], "127");
    assert_eq!(r[1], "32767");
    assert_eq!(r[2], "2147483647");
    assert_eq!(r[3], "9223372036854775807");
    assert_eq!(r[4], "255");
    assert_eq!(r[5], "65535");
    assert_eq!(r[6], "4294967295");
    assert_eq!(r[7], "18446744073709551615");

    // 验证浮点（TabSeparated 输出可能略有格式差异）
    assert!(r[8].starts_with("3.14"), "Float32: {}", r[8]);
    assert!(r[9].starts_with("2.7182"), "Float64: {}", r[9]);

    // 验证 Decimal
    assert_eq!(r[10], "1234.5678");

    // 验证 Nullable String
    assert_eq!(r[11], "hello nullable");

    // 验证 Array
    assert_eq!(r[12], "['a','b','c']");
    assert_eq!(r[13], "[1,2,3]");

    // 验证 UUID
    assert_eq!(r[14], "550e8400-e29b-41d4-a716-446655440000");

    // 验证超长字符串
    assert_eq!(r[15], "A".repeat(100 * 1024));

    // 验证 Unicode
    assert_eq!(r[16], "你好世界🌍 emoji test");

    // 验证特殊字符
    assert_eq!(r[17], special_chars);

    pool.execute(&format!("DROP TABLE IF EXISTS {table}")).await.expect("清理");
    pool.close().await.expect("关闭");
}

// ── 测试 2：DateTime64 精度 ─────────────────────────────────────

#[tokio::test]
async fn datetime_precision() {
    let pool = setup_db().await;
    let pid = std::process::id();
    let table = format!("dt_test_{pid}");

    // 创建具有不同精度的 DateTime64 列的表
    let ddl = format!(
        "CREATE TABLE IF NOT EXISTS {table} (\
           id UInt32,\
           dt3 DateTime64(3),\
           dt6 DateTime64(6)\
         ) ENGINE = MergeTree ORDER BY id"
    );
    pool.execute(&ddl).await.expect("创建 DateTime 表");

    // 插入带毫秒和微秒的时间
    let ts3 = "2024-01-15 10:30:45.123";
    let ts6 = "2024-01-15 10:30:45.123456";
    let row = serde_json::json!({
        "id": 1,
        "dt3": ts3,
        "dt6": ts6,
    });
    pool.insert_json_each_row(&table, &[row]).await.expect("插入时间数据");

    // 查询
    let sql = format!("SELECT dt3, dt6 FROM {table} WHERE id = 1 FORMAT TabSeparated");
    let text = pool.query_text(&sql).await.expect("查询时间数据");
    let parts: Vec<&str> = text.trim().split('\t').collect();

    // DateTime64(3): 预期 2024-01-15 10:30:45.123
    assert!(parts[0].contains(".123"), "DateTime64(3) 应精确到毫秒: {}", parts[0]);

    // DateTime64(6): 预期 2024-01-15 10:30:45.123456
    assert!(parts[1].contains(".123456"), "DateTime64(6) 应精确到微秒: {}", parts[1]);

    pool.execute(&format!("DROP TABLE IF EXISTS {table}")).await.expect("清理");
    pool.close().await.expect("关闭");
}

// ── 测试 3：JSON 往返 ───────────────────────────────────────────

#[tokio::test]
async fn json_roundtrip() {
    let pool = setup_db().await;
    let pid = std::process::id();
    let table = format!("json_test_{pid}");

    let ddl = format!(
        "CREATE TABLE IF NOT EXISTS {table} (\
           id UInt32,\
           data String\
         ) ENGINE = MergeTree ORDER BY id"
    );
    pool.execute(&ddl).await.expect("创建 JSON 测试表");

    // 构造复杂 JSON
    let complex_json = serde_json::json!({
        "name": "测试",
        "values": [1, 2, 3],
        "nested": {
            "inner": true,
            "count": 42,
            "arr": [{"key": "value"}, {"key": "another"}],
            "null_val": null,
            "float": std::f64::consts::PI
        },
        "empty_array": [],
        "empty_object": {},
        "unicode": "こんにちは",
        "special": "tab\tnewline\nquote\"slash\\"
    });

    let row = serde_json::json!({
        "id": 1,
        "data": complex_json.to_string(),
    });
    pool.insert_json_each_row(&table, &[row]).await.expect("插入 JSON 数据");

    // 读回
    let sql = format!("SELECT data FROM {table} WHERE id = 1 FORMAT TabSeparated");
    let text = pool.query_text(&sql).await.expect("查询 JSON 数据");
    let parsed: Value = serde_json::from_str(text.trim()).expect("解析 JSON");

    assert_eq!(parsed, complex_json, "JSON 往返应保持精度");

    pool.execute(&format!("DROP TABLE IF EXISTS {table}")).await.expect("清理");
    pool.close().await.expect("关闭");
}
