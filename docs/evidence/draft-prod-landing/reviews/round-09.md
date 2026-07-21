# Review Round 09 — Coverage and benches

## Methodology

Public API inventory is produced by scanning `src/**/*.rs` for:

- `pub struct|enum|fn|type|trait|const`
- `pub use { ... }` export names

Scaffold-only modules (`adapter.rs` / `mock.rs` / feature-gated scaffold types) are marked **N/A scaffold** and excluded from the production surface denominator.

Coverage criterion: each production public name appears at least twice across `src/` + `tests/` (definition + use).

Artifact: implementer scratch `coverage-public-api.txt` (per-name OK/MISS list + PRODUCTION TOTAL).

## Unit tests on production pools

- `redisx::pool`: connect refused (real connect path) + close flag when env live
- `postgresx::pool`: TLS require rejected + connect refused
- `kafkax::pool`: connect refused (timeout-bounded)
- `natsx::pool`: connect refused (timeout-bounded)
- `clickhousex`/`taosx` clients: refused endpoint ping path

## Benches

Core ops (when services up): redis set/get, postgres SELECT, kafka produce, nats publish, CH SELECT 1, taos ping, oss sign+config.

Disposition: ACCEPT P0 evidence pack with machine inventory.
