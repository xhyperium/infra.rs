# infra.rs Makefile — 快捷命令入口

.PHONY: help
help: ## 显示帮助信息
	@echo "infra.rs — 可用命令："
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| sort \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-22s\033[0m %s\n", $$1, $$2}'

# ── 宪章合规性 ──────────────────────────

.PHONY: check check-quick check-json
check: ## 运行完整宪章合规性验证
	@./scripts/check-constitution.sh

check-quick: ## 快速验证（仅格式 + lint）
	@./scripts/check-constitution.sh --quick

check-json: ## JSON 输出验证结果（供 AI 解析）
	@./scripts/check-constitution.sh --json

# ── Rust 工具链 ──────────────────────────

.PHONY: build test fmt lint doc clean
build: ## 编译 workspace（--all-features）
	@cargo build --workspace --all-features

test: ## 运行全部测试（含 doc-test）
	@cargo test --workspace

fmt: ## 格式化全部代码
	@cargo fmt --all

fmt-check: ## 检查格式（不修改）
	@cargo fmt --all --check

lint: ## 运行 clippy（-D warnings）
	@cargo clippy --workspace --all-targets --all-features -- -D warnings

doc: ## 构建文档（含 private items）
	@cargo doc --no-deps --document-private-items

clean: ## 清理构建产物
	@cargo clean

# ── 安全审计 ─────────────────────────────

.PHONY: deny audit
deny: ## 运行 cargo-deny 安全审计
	@cargo deny check

audit: ## 运行 cargo-audit 漏洞扫描
	@cargo audit

# ── 常用组合 ─────────────────────────────

.PHONY: ci update
ci: fmt-check lint test deny ## CI 模拟（本地运行全部门禁）

update: ## 更新依赖
	@cargo update
