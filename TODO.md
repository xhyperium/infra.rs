# 初始化后续待办（可选）

    适配器（实现 contracts trait）
    crates/adapters/storage/{redis,kafka,nats,postgres,taos,oss,clickhouse}
                                → redisx / kafkax / natsx / postgresx / taosx / ossx / clickhousex

crates/adapters/exchange/{binance,okx}

L1 Infra
crates/{configx,observex,resiliencx,schedulex,transport,bootstrap}
transport 的 package 名为 transportx

契约 / 类型
crates/contracts → contracts
crates/types/decimal → decimalx
crates/types/canonical → canonical

L0
crates/kernel  
 crates/testkit  
 crates/evidence → evidence（SPEC-EVIDENCE-002 Core V1）

工具
tools/{xtask,archgate,schema_codegen,evidence-cli}

touch crates/AGENTS.md

当前工作上下文：worktree 强制开发、.mjs 脚本迁移、SSOT 规则体系、skills 投影同步、分支保护验证。

crates 子模块标准规范
src/
examples/
docs/
tests/
CHANGELOG.md
AGENTS.md
README.md

 
crates/

review 



/goal 深度分析 docs/report/2026-07-22/review-prompt.md 执行



/goal 深度分析 .agents/ssot 优化，补充spec，实现生产级标准
以上重复执行10次
使用agent team 执行
提交，创建pr ,合并，清理