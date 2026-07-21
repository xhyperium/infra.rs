# 七、AI 代理章程

## 7.1 权限边界

- AI **不可** approve 或 merge PR
- AI **不可** 直接推送 `main` 分支（§6.0 Git Main First）
- AI **不可** 在 `main` 上直接开发或提交（§6.0.2）
- AI **不可** 修改 `.github/CODEOWNERS`
- AI **不可** 绕过任何强制门禁

## 7.2 职责范围

- AI 可执行：编码、测试编写、代码审查建议、文档生成、issue 分类
- AI 不可执行：审批、合并、发布、权限变更、CI 配置修改（需人工审查）

## 7.3 输出标准

- AI 生成的代码须与手工代码同等质量
- AI 须明确标注不确定的部分
- AI 修改后须运行 `cargo test` + `cargo fmt --check` + `cargo clippy`
- AI 产出的**注释、中文文档、用户可见错误信息**须为**中文**（§4.5）
- AI 产出的**英文技术文档**须符合 **ASD-STE100**（§4.6）
- AI 写入的文本文件须为 **UTF-8 无 BOM**；不得引入乱码或 `U+FFFD`

---

← [上一章：治理](./06-governance.md) · [索引](./README.md) · 下一章：[八、修订](./08-amendments.md) →
