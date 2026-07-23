# ASD-STE100 文档规范（可选 · 英文交付指南）

> **效力（v1.8.0）**：组织默认 **强制中文**（[`language.md`](https://github.com/xhyperium/.github/blob/main/rulesets/language.md) + 宪章 §4.5）。  
> 本文件**不是**默认交付标准；仅当存在**书面豁免**的英文技术交付物时参考。  
> 宪章锚点：[§4.6 英文技术文档与 ASD-STE100（可选加严）](../../docs/constitution/04-code-standards.md#46-英文技术文档与-asd-ste100可选加严)。  
> 本文件**不是** ASD-STE100 官方全文（官方规范受版权保护，请通过正规渠道获取）。

## 1. 是什么

**ASD-STE100**（Simplified Technical English，简称 **STE**）是面向技术文档的受控自然语言规范。

目标：降低非母语读者理解成本、减少歧义、便于翻译与一致性维护。

## 2. 在本仓库如何适用

| 文档类型 | 语言标准 |
| ---------- | ---------- |
| 仓库治理 / 协作 / 说明（默认） | **中文**（§4.5 + 组织 language.md） |
| 代码注释 / 用户可见错误 | **中文** |
| 代码标识符 | 英文（Rust 惯例） |
| `LICENSE` | 英文原文 |
| **经书面豁免**的对外英文手册 / 英文 API 说明 / 英文 runbook | **建议** STE 风格（本指南） |

**原则**：默认中文交付；英文层可选且须豁免；不得用 STE 压过中文义务。

## 3. 核心写作规则（摘要）

下列规则是 STE 思想的可执行摘要，**不能替代**官方字典与完整规则集。  
**仅在豁免范围内写英文**时至少遵守：

### 3.1 词汇

- 一词一义；避免同一词多种含义
- 优先短词、常用词、可定义的技术名词
- 术语首次出现给出定义；全文保持同一写法

### 3.2 句子

- **一句一个主题**
- 句子尽量短
- 优先 **主动语态**
- 优先 **简单现在时**（描述）/ **祈使语气**（步骤）
- 避免嵌套从句与过长并列

### 3.3 结构

- 标题层级清晰，一步一事
- 程序类文档：编号步骤 + 条件/结果
- 警告 / 注意 放在操作之前
- 列表用于并列项；不要用散文塞满多个动作

### 3.4 示例（对比）

不推荐（冗长、被动、含糊）：

> The configuration file should be carefully modified by the operator so that the service can be restarted afterwards if necessary.

推荐（STE 风格）：

> 1. Open the configuration file.  
> 2. Change the required parameters.  
> 3. Save the file.  
> 4. Restart the service.

（中文母语文档请直接用中文写步骤，不必先英文再译。）

## 4. 与中文文档的关系

- 中文文档**不**套用 STE 英文词表，但可借鉴：**短句、一步一事、少歧义**
- 翻译方向优先：**中文定稿** →（若豁免需要）再写英文 STE 层
- 中英并存时术语与步骤顺序一致

## 5. 自检清单（仅英文豁免交付）

- [ ] 是否已有 PR 书面豁免？
- [ ] 是否英文技术交付物？（是 → 适用本指南）
- [ ] 一词一义？步骤可执行？警告在前？
- [ ] 中文母语说明是否已齐全？

## 6. 参考

- 官方站点与规范获取：通过 ASD-STE100 官方渠道（版权与词典以官方版本为准）
- 组织语言政策：[`rulesets/language.md`](https://github.com/xhyperium/.github/blob/main/rulesets/language.md)
