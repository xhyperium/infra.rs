# ASD-STE100 文档规范（项目落地指南）

> **宪章效力**：英文技术文档以 [docs/constitution/04-code-standards.md §4.6](../constitution/04-code-standards.md#46-文档标准asd-ste100强制) 为准。  
> 本文件是**落地指南**，不是 ASD-STE100 官方全文（官方规范受版权保护，请通过正规渠道获取）。

## 1. 是什么

**ASD-STE100**（*Simplified Technical English*，简称 **STE**）是面向技术文档的**受控自然语言**与行业规范，由 ASD（AeroSpace and Defence Industries Association of Europe）维护。

目标：

- 降低非母语读者理解成本
- 减少歧义与误操作
- 便于翻译与一致性维护
- 让步骤、警告、说明可预测、可检查

## 2. 在本仓库如何适用

| 文档类型 | 语言标准 |
| ---------- | ---------- |
| 仓库治理 / 协作 / 中文说明（宪章、AGENTS、PR 模板等） | **中文**（§4.5） |
| **对外 / 可交付的英文技术文档**（用户手册、API 英文说明、运维 runbook 英文版、crate 对外 README 英文层） | **ASD-STE100（STE）**（§4.6） |
| 代码标识符 | 英文（Rust 惯例） |
| `LICENSE` | 英文原文 |

**原则**：中文写「我们怎么协作」；英文技术正文写「系统做什么、用户如何操作」——英文侧用 STE。

## 3. 核心写作规则（摘要）

下列规则是 STE 思想的可执行摘要，**不能替代**官方字典与完整规则集。写作英文技术文档时至少遵守：

### 3.1 词汇

- 一词一义；避免同一词多种含义
- 优先短词、常用词、可定义的技术名词
- 禁止含糊词：如 *approximately*、*about*（数量）在步骤中随意使用
- 术语首次出现给出定义；全文保持同一写法

### 3.2 句子

- **一句一个主题**
- 句子尽量短（描述句建议 ≤ 20 词；程序步骤可更短）
- 优先 **主动语态**
- 优先 **简单现在时**（描述）/ **祈使语气**（步骤）
- 避免嵌套从句与过长并列

### 3.3 结构

- 标题层级清晰，一步一事
- 程序类文档：编号步骤 + 条件/结果
- 警告 / 注意 / 危险 分级明确，放在操作之前
- 列表用于并列项；不要用散文塞满多个动作

### 3.4 示例（对比）

不推荐（冗长、被动、含糊）：

> The configuration file should be carefully modified by the operator so that the service can be restarted afterwards if necessary.

推荐（STE 风格）：

> 1. Open the configuration file.  
> 2. Change the required parameters.  
> 3. Save the file.  
> 4. Restart the service.

## 4. 与中文文档的关系

- 中文文档**不**强制套用 STE 英文词表，但应借鉴其精神：**短句、一步一事、少歧义、可验证**
- 同一主题若有中英双语，**术语与步骤顺序必须一致**
- 翻译方向优先：中文定稿 → 按 STE 写英文；或英文 STE 定稿 → 中文忠实翻译

## 5. AI 与审查清单

写或审英文技术文档时自检：

- [ ] 是否英文技术交付物？（是 → 适用 STE）
- [ ] 一词是否一义？术语是否前后一致？
- [ ] 句子是否够短？是否一句一意？
- [ ] 步骤是否祈使、可执行、可排序？
- [ ] 警告是否在操作之前？
- [ ] 文件是否 UTF-8 无 BOM（§4.5）？

## 6. 参考

- 官方站点与规范获取：通过 ASD-STE100 官方渠道（版权与词典以官方版本为准）
- 项目宪章：`docs/constitution/04-code-standards.md` §4.6
- 语言与编码：`docs/governance/编码与语言约定.md`
