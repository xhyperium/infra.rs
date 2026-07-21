# goalctl Schema Registry

```text
Registry root:  .agents/ssot/tools/goalctl/schemas/
Cross-repo:     docs/goal/schema/   （方法论与 Authority Policy）
Policy:         DECISION-PACK-001 D06 · CR-20260716
Status:         PR-0A ACTIVE（形状）；实现未授权
```

## 放置规则

| 内容 | 位置 |
|------|------|
| Authority rank / class | `docs/goal/schema/authority-policy.yaml` |
| Goal 方法论 schema | `docs/goal/schema/*` |
| goalctl **运行输出 / 私有合同** | 本目录 `*.schema.json` |

禁止新建第三棵 schema 树。禁止在 Rust 中复制一份不可同步的字段表作为 SSOT。

## 兼容性（D06）

```text
Major: breaking — Reader 遇更高 major → 拒绝
Minor: additive
Patch: 澄清 / 缺陷修复

Reader v1.x:
- 必须读取 1.0～当前 1.x
- 拒绝更高 major

unknown fields:
- Authority / Security / Approval / Capability → deny（fail-closed）
- Operational reports → 版本门控下可保留 extensions
```

## 清单

| Schema | 版本 | 用途 | 最早消费波次 |
|--------|------|------|----------------|
| [cli-output.schema.json](./cli-output.schema.json) | 1.0.0 | 所有 CLI JSON 外壳 | PR-1 |
| [repository-index.schema.json](./repository-index.schema.json) | 1.0.0 | `goalctl index` | PR-1 |
| [authority-snapshot.schema.json](./authority-snapshot.schema.json) | 1.0.0 | `goalctl resolve` | PR-2 |
| [artifact-envelope.schema.json](./artifact-envelope.schema.json) | 1.0.0 | Artifact Control Block / index | PR-2 |
| [approval-record.schema.json](./approval-record.schema.json) | 1.0.0 | 批准事实（≠ Markdown Status） | PR-0A 形状；PR-2+ 强制 |
| [repository-identity.schema.json](./repository-identity.schema.json) | 1.0.0 | 稳定仓库身份 | PR-1 |
| [reconciliation-report.schema.json](./reconciliation-report.schema.json) | 1.0.0 | `goalctl reconcile`（骨架） | PR-3 |
| [task-pack.schema.json](./task-pack.schema.json) | 1.0.0 | `goalctl compile`（骨架） | PR-4 |

骨架 schema 允许后续 minor 增补字段；**不得**在未升 major 时删除/改义 required 字段。

## Canonical JSON（确定性）

实现序列化 MUST：

- UTF-8
- 对象 key 按 Unicode code point 升序（或等价稳定全序，并在测试中固定）
- 整数不写成浮点；禁止 `NaN`/`Infinity`
- 路径为仓库相对，POSIX `/` 分隔
- **禁止**写入：墙钟 `generated_at`（除非字段定义为制品作者提交内容）、随机 UUID、PID、本机绝对路径、用户 home

`updated_at` 若出现在 ArtifactEnvelope 中，MUST 来自制品内容，不得由 CLI 每次运行刷新。

## 校验（无实现时）

```bash
python3 - <<'PY'
import json, sys
from pathlib import Path
root = Path(".agents/ssot/tools/goalctl/schemas")
for p in sorted(root.glob("*.schema.json")):
    json.load(p.open())
    print("ok", p.name)
PY
python3 -c "import yaml; yaml.safe_load(open('docs/goal/schema/authority-policy.yaml')); print('ok authority-policy')"
```
