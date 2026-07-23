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

    let mut cfg = default_config();
    cfg.database = TEST_DATABASE.into();
    ClickHousePool::connect(cfg)
        .await
        .expect("连接到测试数据库")
}

#[tokio::test]
async fn type_mapping_correctness() {
    let pool = setup_db().await;
    pool.ping().await.expect("ping");
    let pid = std::process::id();
    let table = format!("types_test_{pid}");

    let ddl = format!(
        "CREATE TABLE IF NOT EXISTS {table} (\
           col_int8 Int8, col_int16 Int16, col_int32 Int32, col_int64 Int64,\
           col_uint8 UInt8, col_uint16 UInt16, col_uint32 UInt32, col_uint64 UInt64,\
           col_float32 Float32, col_float64 Float64, col_decimal Decimal64(4),\
           col_str Nullable(String), col_arr_str Array(String), col_arr_int Array(Int64),\
           col_uuid UUID, col_long_str String, col_unicode String, col_special String\
         ) ENGINE = MergeTree ORDER BY col_int32"
    );
    pool.execute(&ddl).await.expect("创建类型映射表");

    let count_check = format!("SELECT count() FROM {table} FORMAT TabSeparated");
    let count_result = pool.query_text(&count_check).await;
    match &count_result {
        Ok(txt) => assert_eq!(txt.trim(), "0", "空表 count 应为 0, got: {}", txt),
        Err(e) => panic!("count check 失败: {e:?}"),
    }

    let long_str = "A".repeat(100 * 1024);
    let special_chars = "tab_here_newline_next_quote_bslash_null_end";

    let row = serde_json::json!({
        "col_int8": 127i8, "col_int16": 32767i16, "col_int32": 2147483647i32,
        "col_int64": 9223372036854775807i64, "col_uint8": 255u8, "col_uint16": 65535u16,
        "col_uint32": 4294967295u32, "col_uint64": 18446744073709551615u64,
        "col_float32": 3.14f32, "col_float64": 2.718281828459045f64,
        "col_decimal": 1234.5678f64, "col_str": "hello nullable",
        "col_arr_str": ["a", "b", "c"], "col_arr_int": [1i64, 2, 3],
        "col_uuid": "550e8400-e29b-41d4-a716-446655440000",
        "col_long_str": long_str, "col_unicode": "ni hao shi jie",
        "col_special": special_chars,
    });
    pool.insert_json_each_row(&table, &[row])
        .await
        .expect("插入类型测试数据");

    let count_check2 = format!("SELECT count() FROM {table} FORMAT TabSeparated");
    let count2 = pool.query_text(&count_check2).await;
    match &count2 {
        Ok(txt) => assert_eq!(txt.trim(), "1", "插入后 count 应为 1, got: {}", txt),
        Err(e) => panic!("插入后 count check 失败: {e:?}"),
    }

    let sql = format!("SELECT col_int8 FROM {table} FORMAT TabSeparated");
    let rows = pool.query_rows(&sql).await.expect("查询 col_int8");
    assert_eq!(rows.len(), 1, "应查到 1 行");
    assert_eq!(rows[0][0], "127");

    let sql_full = format!("SELECT col_int8, col_int16, col_int32, col_int64, col_uint8, col_uint16, col_uint32, col_uint64, col_float32, col_float64, col_decimal, col_str, col_arr_str, col_arr_int, col_uuid, col_long_str, col_unicode, col_special FROM {table} FORMAT TabSeparated");
    let rows_all = pool.query_rows(&sql_full).await.expect("查询所有列");
    assert_eq!(rows_all.len(), 1, "应查到 1 行");
    let r = &rows_all[0];
    assert_eq!(r.len(), 18, "应有 18 列");

    assert_eq!(r[0], "127"); assert_eq!(r[1], "32767");
    assert_eq!(r[2], "2147483647"); assert_eq!(r[3], "9223372036854775807");
    assert_eq!(r[4], "255"); assert_eq!(r[5], "65535");
    assert_eq!(r[6], "4294967295"); assert_eq!(r[7], "18446744073709551615");
    assert!(r[8].starts_with("3.14"), "Float32: {}", r[8]);
    assert!(r[9].starts_with("2.7182"), "Float64: {}", r[9]);
    assert_eq!(r[10], "1234.5678");
    assert_eq!(r[11], "hello nullable");
    assert_eq!(r[12], "['a','b','c']"); assert_eq!(r[13], "[1,2,3]");
    assert_eq!(r[14], "550e8400-e29b-41d4-a716-446655440000");
    assert_eq!(r[15], "A".repeat(100 * 1024));
    assert_eq!(r[16], "ni hao shi jie");
    assert_eq!(r[17], special_chars);

    pool.execute(&format!("DROP TABLE IF EXISTS {table}")).await.expect("清理");
    pool.close().await.expect("关闭");
}

#[tokio::test]
async fn datetime_precision() {
    let pool = setup_db().await;
    let pid = std::process::id();
    let table = format!("dt_test_{pid}");

    pool.execute(&format!(
        "CREATE TABLE IF NOT EXISTS {table} (\
           id UInt32, dt3 DateTime64(3), dt6 DateTime64(6)\
         ) ENGINE = MergeTree ORDER BY id"
    )).await.expect("创建 DateTime 表");

    let row = serde_json::json!({
        "id": 1, "dt3": "2024-01-15 10:30:45.123",
        "dt6": "2024-01-15 10:30:45.123456",
    });
    pool.insert_json_each_row(&table, &[row]).await.expect("插入时间数据");

    let text = pool.query_text(&format!("SELECT dt3, dt6 FROM {table} WHERE id = 1 FORMAT TabSeparated")).await.expect("查询时间数据");
    let parts: Vec<&str> = text.trim().split('\t').collect();
    assert!(parts[0].contains(".123"), "DateTime64(3) 应精确到毫秒: {}", parts[0]);
    assert!(parts[1].contains(".123456"), "DateTime64(6) 应精确到微秒: {}", parts[1]);

    pool.execute(&format!("DROP TABLE IF EXISTS {table}")).await.expect("清理");
    pool.close().await.expect("关闭");
}

#[tokio::test]
async fn json_roundtrip() {
    let pool = setup_db().await;
    let pid = std::process::id();
    let table = format!("json_test_{pid}");

    pool.execute(&format!(
        "CREATE TABLE IF NOT EXISTS {table} (id UInt32, data String) ENGINE = MergeTree ORDER BY id"
    )).await.expect("创建 JSON 测试表");

    let complex_json = serde_json::json!({
        "name": "test", "values": [1, 2, 3],
        "nested": {"inner": true, "count": 42, "arr": [{"key": "value"}, {"key": "another"}],
                   "null_val": null, "float": 3.14159},
        "empty_array": [], "empty_object": {}, "unicode": "konnichiwa",
        "special": "newline_and_slash"
    });
    pool.insert_json_each_row(&table, &[serde_json::json!({"id": 1, "data": complex_json.to_string()})])
        .await.expect("插入 JSON 数据");

    let text = pool.query_text(&format!("SELECT data FROM {table} WHERE id = 1 FORMAT TabSeparated")).await.expect("查询 JSON 数据");
    let parsed: Value = serde_json::from_str(text.trim()).expect("解析 JSON");
    assert_eq!(parsed, complex_json, "JSON 往返应保持精度");

    pool.execute(&format!("DROP TABLE IF EXISTS {table}")).await.expect("清理");
    pool.close().await.expect("关闭");
}
