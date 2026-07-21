#!/usr/bin/env node
/**
 * gh-sync-i18n-test.mjs — interactiveReview 多语言数据同步测试
 *
 * 覆盖全球主要书写系统在冲突审查面板中的渲染与决策正确性：
 *   CJK (中日韩) | RTL (阿拉伯/希伯来) | 西里尔 | 扩展拉丁
 *   | 天城文/泰文 | 混合脚本 | 双向文本 | 多字节边界
 *   | 变音符号堆叠 | 全角半角 | 不可见字符组合
 *
 * SSOT: scripts/beads/gh-sync.mjs
 */

import { interactiveReview } from "./gh-sync.mjs";

let _pass = 0;
let _fail = 0;
function ok(c, n) { if (c) { _pass++; console.log(`  PASS  ${n}`); } else { _fail++; console.log(`  FAIL  ${n}`); } }
function eq(a, b, n) { const r = JSON.stringify(a) === JSON.stringify(b); ok(r, `${n} (expected ${JSON.stringify(b)}, got ${JSON.stringify(a)})`); return r; }
function makeInput(ans) { let i = 0; return () => ans[i++] || ""; }

function mkConflict(id, b, g) {
  return {
    beadId: id,
    beadIssue: { id, updatedAt: "2026-07-21T12:00:00Z", ...b },
    ghIssue: { ghNumber: 9000 + ++_ghNum, updatedAt: "2026-07-21T13:00:00Z", ...g },
  };
}
let _ghNum = 0;

console.log("╔══════════════════════════════════════════════════════════╗");
console.log("║  interactiveReview 多语言数据同步测试                  ║");
console.log("╚══════════════════════════════════════════════════════════╝");

// ====================== 1. 简体中文 ======================
console.log("\n=== 语言 1: 简体中文 (zh-CN) ===");
{
  const c = mkConflict("zh-1",
    {
      title: "[P0] transport: 修复敏感信息泄露到 Debug 输出",
      status: "in_progress", priority: 0, issueType: "bug",
      labels: ["安全", "传输层", "生产环境"],
      description: "HttpRequest::fmt 实现将 Authorization 请求头原样写入 Debug 输出，\n"
        + "导致凭据泄漏到日志系统。修复方案：为敏感请求头字段实现默认脱敏，\n"
        + "以 \"***REDACTED***\" 替换值。同时增加 safety test 覆盖。\n\n"
        + "关联: ADR-012 安全日志策略",
    },
    {
      title: "[P0] transport: 修复敏感信息泄露 (GitHub 评论)",
      status: "open", priority: 0, issueType: "bug",
      labels: ["安全", "传输层"],
      description: "Code review 建议：脱敏逻辑应当提取到独立的 SensitiveHeader trait，\n"
        + "便于其他 adapter 复用。已在 lifecycle.rs:L142 处标注。",
    }
  );
  const d = interactiveReview([c], {}, makeInput(["b"]));
  eq(d, { "zh-1": "push" }, "简体中文: 决策正确");
}

// ====================== 2. 繁体中文 ======================
console.log("\n=== 语言 2: 繁體中文 (zh-TW) ===");
{
  const c = mkConflict("zh-tw-1",
    {
      title: "feat(kernel): 新增生命週期 on_stop 回呼",
      status: "open", priority: 2, issueType: "feature",
      labels: ["核心", "生命週期", "L0"],
      description: "元件銷毀前觸發 on_stop，允許外部在清理階段注入收尾邏輯。\n"
        + "該回呼在 on_close 之前、資源釋放之前觸發，\n"
        + "保證收尾時依賴仍可用。\n\n"
        + "討論見 RFC-042。",
    },
    {
      title: "feat(kernel): 新增生命週期 on_stop 回呼 (已指派)",
      status: "in_progress", priority: 2, issueType: "feature",
      labels: ["核心", "生命週期", "L0", "進行中"],
      description: "已指派 @dev-b 實現。預計 v0.5.0 發布。",
    }
  );
  const d = interactiveReview([c], {}, makeInput(["g"]));
  eq(d, { "zh-tw-1": "pull" }, "繁體中文: 决策正确");
}

// ====================== 3. 日本語 ======================
console.log("\n=== 言語 3: 日本語 (ja) ===");
{
  const c = mkConflict("ja-1",
    {
      title: "fix(schedulex): タスク重複割り当ての競合を修正",
      status: "blocked", priority: 1, issueType: "bug",
      labels: ["スケジューラ", "競合", "p1"],
      description: "二つの並行呼び出しで claim_task が同じ ID を割り当てる可能性があります。\n"
        + "AtomicCounter の compare_exchange が弱い順序で正しく処理されていません。\n"
        + "Acquire/Release メモリ順序を使った三段階方式に変更します。\n\n"
        + "参考: Rust の Atomics ドキュメント §3.2",
    },
    {
      title: "fix(schedulex): タスク競合修正 [GitHub side]",
      status: "open", priority: 1, issueType: "bug",
      labels: ["スケジューラ", "競合"],
      description: "GitHub コメント: schedule.rs の 142 行目に問題があります。\n"
        + "ロック順序のテストも追加してください。",
    }
  );
  const d = interactiveReview([c], {}, makeInput(["b"]));
  eq(d, { "ja-1": "push" }, "日本語: 决策正确");
}

// ====================== 4. 한국어 ======================
console.log("\n=== 언어 4: 한국어 (ko) ===");
{
  const c = mkConflict("ko-1",
    {
      title: "docs(bootstrap): 부트스트랩 모듈 API 문서 보완",
      status: "open", priority: 3, issueType: "task",
      labels: ["문서화", "부트스트랩", "L1"],
      description: "부트스트랩 모듈은 현재 README 플레이스홀더만 존재합니다.\n"
        + "전체 API 문서와 사용 예제를 추가해야 합니다.\n"
        + "특히 구성 요소 합성 루트 설계 패턴에 중점을 둡니다.",
    },
    {
      title: "docs(bootstrap): API 문서 보완 [GitHub]",
      status: "open", priority: 3, issueType: "task",
      labels: ["문서화", "부트스트랩"],
      description: "GitHub 의견: API 목록뿐만 아니라 설계 사례 연구로 작성해 주세요.",
    }
  );
  const d = interactiveReview([c], {}, makeInput(["s"]));
  eq(d, { "ko-1": "skip" }, "한국어: 决策正确");
}

// ====================== 5. العربية (RTL) ======================
console.log("\n=== لغة 5: العربية (ar / RTL) ===");
{
  const c = mkConflict("ar-1",
    {
      title: "fix(resiliencx): إصلاح تسرب الذاكرة في إعادة المحاولة",
      status: "in_progress", priority: 0, issueType: "bug",
      labels: ["إصلاح", "حرج", "p0"],
      description: "مُعامل إعادة المحاولة لا يقوم بتنظيف المخزن المؤقت الداخلي\n"
        + "بعد تجاوز الحد الأقصى لعدد المحاولات. يؤدي ذلك إلى تسرب في الذاكرة.\n"
        + "الحل: إضافة استدعاء drain() في كتلة finally.",
    },
    {
      title: "fix(resiliencx): تسرب الذاكرة [تمت المراجعة]",
      status: "open", priority: 0, issueType: "bug",
      labels: ["إصلاح", "حرج"],
      description: "مراجعة: LGTM. يُرجى إضافة اختبار تسرب مع valgrind.",
    }
  );
  const d = interactiveReview([c], {}, makeInput(["b"]));
  eq(d, { "ar-1": "push" }, "العربية RTL: 决策正确");
}

// ====================== 6. עברית (RTL) ======================
console.log("\n=== שפה 6: עברית (he / RTL) ===");
{
  const c = mkConflict("he-1",
    {
      title: "feat(configx): הוסף תמיכה בטעינת תצורה מקובץ",
      status: "open", priority: 2, issueType: "feature",
      labels: ["תצורה", "L1"],
      description: "כרגע configx תומך רק באחסון מחרוזות בזיכרון.\n"
        + "דרושה תמיכה בטעינה מקובץ TOML/JSON.\n"
        + "יש לשמור על תאימות לאחור.",
    },
    {
      title: "feat(configx): טעינת תצורה [GitHub]",
      status: "open", priority: 2, issueType: "feature",
      labels: ["תצורה", "L1", "v0.4.0"],
      description: "יעד: v0.4.0. מימוש ראשוני בקובץ TOML בלבד.",
    }
  );
  const d = interactiveReview([c], {}, makeInput(["g"]));
  eq(d, { "he-1": "pull" }, "עברית RTL: 决策正确");
}

// ====================== 7. Русский (Cyrillic) ======================
console.log("\n=== Язык 7: Русский (ru / Cyrillic) ===");
{
  const c = mkConflict("ru-1",
    {
      title: "perf(observex): оптимизация трассировки спанов",
      status: "in_progress", priority: 2, issueType: "task",
      labels: ["наблюдаемость", "оптимизация", "p2"],
      description: "Создание спанов в горячем пути вызывает избыточные аллокации.\n"
        + "Решение: пул предварительно выделенных спанов с переиспользованием.\n"
        + "Ожидаемое улучшение: -40% аллокаций в критическом пути.",
    },
    {
      title: "perf(observex): спаны [GitHub]",
      status: "open", priority: 2, issueType: "task",
      labels: ["наблюдаемость", "оптимизация"],
      description: "GitHub: Используйте object_pool крейт для реализации пула.",
    }
  );
  const d = interactiveReview([c], {}, makeInput(["b"]));
  eq(d, { "ru-1": "push" }, "Русский: 决策正确");
}

// ====================== 8. Français + English mix ======================
console.log("\n=== Langue 8: Français / English mix ===");
{
  const c = mkConflict("fr-1",
    {
      title: "feat(contracts): ajouter une méthode validate() à Exchange",
      status: "open", priority: 1, issueType: "feature",
      labels: ["contrat", "validation", "breaking-change"],
      description: "La méthode validate() vérifie les préconditions avant d'exécuter un ordre.\n"
        + "Cette méthode doit être appelée avant place_order() pour garantir\n"
        + "l'intégrité des données. Implémentation requise pour tous les adaptateurs.\n\n"
        + "RFC: ADR-023 Ordre Validation Contract",
    },
    {
      title: "feat(contracts): validate() for Exchange trait [BREAKING]",
      status: "open", priority: 1, issueType: "feature",
      labels: ["contrat", "validation", "breaking-change"],
      description: "Breaking change: all Exchange implementors must implement validate().\n"
        + "Migration: add method returning Ok(()) as default behavior.",
    }
  );
  const d = interactiveReview([c], {}, makeInput(["b"]));
  eq(d, { "fr-1": "push" }, "Français/English: 决策正确");
}

// ====================== 9. Deutsch mit Umlauten ======================
console.log("\n=== Sprache 9: Deutsch (de) ===");
{
  const c = mkConflict("de-1",
    {
      title: "fix(decimalx): Überlauf bei Multiplikation großer Beträge",
      status: "in_progress", priority: 0, issueType: "bug",
      labels: ["Währung", "Überlauf", "p0", "kritisch"],
      description: "Die Multiplikation mit großen Beträgen (> 10^12) führt zu einem\n"
        + "stillen Überlauf im Fixed-Point-Format. Lösung: checked_mul()\n"
        + "mit explizitem Panikschutz und Rückgabe eines Result-Typs.\n\n"
        + "Betroffene Methoden: Money::multiply, Money::scale, Decimal::shift",
    },
    {
      title: "fix(decimalx): Überlauf Multiplikation [GitHub fix]",
      status: "open", priority: 0, issueType: "bug",
      labels: ["Währung", "Überlauf", "p0"],
      description: "GitHub: zusätzlich Overflow-Schutz für Division und Addition prüfen.",
    }
  );
  const d = interactiveReview([c], {}, makeInput(["b"]));
  eq(d, { "de-1": "push" }, "Deutsch: 决策正确");
}

// ====================== 10. 混合脚本 (中+日+英+Emoji) ======================
console.log("\n=== 混合 10: 多脚本混合 ===");
{
  const c = mkConflict("mix-1",
    {
      title: "🚀 feat: 新增 マルチシグ MultiSig Contract · ميزة جديدة ✨",
      status: "open", priority: 1, issueType: "feature",
      labels: ["🔐 security", "📜 contract", "🌐 multi-lang", "v2.0 🎯"],
      description: "本功能支持 3-of-5 多重签名验证。\n\n"
        + "日本語: 3-of-5 マルチシグ検証をサポートします。\n"
        + "English: Supports 3-of-5 multi-signature verification.\n"
        + "한국어: 3-of-5 다중 서명 검증을 지원합니다.\n"
        + "العربية: يدعم التحقق من التوقيع المتعدد 3 من 5.\n\n"
        + "🔑 キー管理 · Key Management · إدارة المفاتيح",
    },
    {
      title: "🚀 feat: MultiSig Contract [GitHub RFC]",
      status: "open", priority: 1, issueType: "feature",
      labels: ["🔐 security", "📜 contract", "🌐 multi-lang"],
      description: "RFC discussion on GitHub. Supporting 2-of-3 as well?\n"
        + "Consider BLS signature aggregation for efficiency.",
    }
  );
  const d = interactiveReview([c], {}, makeInput(["b"]));
  eq(d, { "mix-1": "push" }, "混合脚本: 决策正确");
}

// ====================== 11. 双向文本 (RTL + LTR) ======================
console.log("\n=== Bidi 11: 双向文本混合 ===");
{
  const c = mkConflict("bidi-1",
    {
      title: "fix: RTL layout في Debug output (LTR fix inside Arabic text)",
      status: "open", priority: 1, issueType: "bug",
      labels: ["i18n", "bidi", "rtl"],
      description: "عند عرض النص العربي (Arabic) مع نص إنجليزي (English) في نفس السطر،\n"
        + "يتم عكس اتجاه النص بشكل غير صحيح. This happens when mixing RTL and LTR.\n\n"
        + "الحل: Unicode Bidi Algorithm (UBA) مع أحرف تحكم LRM/RLM.\n"
        + "Solution: Insert Unicode LRM/RLM control characters at boundaries.\n\n"
        + "مرجع: Unicode Standard Annex #9 (UAX#9)",
    },
    {
      title: "fix: RTL bidi text in Debug [GitHub]",
      status: "open", priority: 1, issueType: "bug",
      labels: ["i18n", "bidi"],
      description: "GitHub suggestion: use unicode-bidi crate for automatic handling.",
    }
  );
  const d = interactiveReview([c], {}, makeInput(["b"]));
  eq(d, { "bidi-1": "push" }, "双向文本: 决策正确");
}

// ====================== 12. 天城文 (Devanagari) ======================
console.log("\n=== लिपि 12: देवनागरी (hi / Devanagari) ===");
{
  const c = mkConflict("hi-1",
    {
      title: "docs: दस्तावेज़ीकरण गाइड में बहुभाषी समर्थन जोड़ें",
      status: "open", priority: 3, issueType: "task",
      labels: ["दस्तावेज़", "बहुभाषी", "p3"],
      description: "दस्तावेज़ीकरण गाइड में बहुभाषी उदाहरण जोड़ने की आवश्यकता है।\n"
        + "विशेष रूप से API दस्तावेज़ में कोड उदाहरण।",
    },
    {
      title: "docs: बहुभाषी समर्थन [GitHub]",
      status: "open", priority: 3, issueType: "task",
      labels: ["दस्तावेज़", "बहुभाषी"],
      description: "GitHub: इसे v0.5.0 रिलीज़ से पहले पूरा करें।",
    }
  );
  const d = interactiveReview([c], {}, makeInput(["s"]));
  eq(d, { "hi-1": "skip" }, "देवनागरी: 决策正确");
}

// ====================== 13. ภาษาไทย ======================
console.log("\n=== ภาษา 13: ภาษาไทย (th) ===");
{
  const c = mkConflict("th-1",
    {
      title: "fix: แก้ไขข้อผิดพลาดในการเข้ารหัส UTF-8 ใน transport layer",
      status: "in_progress", priority: 1, issueType: "bug",
      labels: ["การเข้ารหัส", "transport", "utf-8"],
      description: "การส่งข้อมูลภาษาไทยผ่าน WebSocket ทำให้เกิดข้อผิดพลาด\n"
        + "ในการตัดคำที่ตำแหน่ง multi-byte boundary\n"
        + "เนื่องจาก buffer ไม่ได้ตรวจสอบ continuation bytes\n\n"
        + "วิธีแก้: ใช้ char_boundary() ก่อนตัด buffer",
    },
    {
      title: "fix: UTF-8 encoding in transport [GitHub]",
      status: "open", priority: 1, issueType: "bug",
      labels: ["การเข้ารหัส", "transport"],
      description: "GitHub: add test with Thai text to prevent regression.",
    }
  );
  const d = interactiveReview([c], {}, makeInput(["b"]));
  eq(d, { "th-1": "push" }, "ภาษาไทย: 决策正确");
}

// ====================== 14. 全角半角混合 ======================
console.log("\n=== 符号 14: 全角半角混合 ===");
{
  const c = mkConflict("fw-1",
    {
      title: "fix: 全角數字１２３４５ vs 半角数字12345 のバグ",
      status: "open", priority: 2, issueType: "bug",
      labels: ["全角", "半角", "入力検証"],
      description: "ユーザー入力の全角数字（１２３）が半角数字（123）として\n"
        + "正規化されていません。＄１００ → $100 の変換が必要です。\n\n"
        + "対象文字: ０-９ → 0-9, Ａ-Ｚ → A-Z, ａ-ｚ → a-z\n"
        + "記号: ！＂＃＄％＆＇（）＊＋，－．／：；＜＝＞？＠［＼］＾＿｀｛｜｝～",
    },
    {
      title: "fix: fullwidth/halfwidth normalization [GitHub]",
      status: "open", priority: 2, issueType: "bug",
      labels: ["全角", "半角"],
      description: "GitHub: use unicode-normalization crate with NFKC.",
    }
  );
  const d = interactiveReview([c], {}, makeInput(["g"]));
  eq(d, { "fw-1": "pull" }, "全角半角: 决策正确");
}

// ====================== 15. 变音符号堆叠 ======================
console.log("\n=== Diacritics 15: 变音符号堆叠 ===");
{
  const c = mkConflict("diac-1",
    {
      title: "feat: ạ̉ ệ̉ ộ ụ̃ — Vietnamese tone marks in canonical types",
      status: "open", priority: 3, issueType: "feature",
      labels: ["i18n", "vietnamese", "diacritics", "canonical"],
      description: "Tiếng Việt có dấu (Vietnamese with tone marks) cần được hỗ trợ\n"
        + "trong canonical types. Các ký tự như ạ, ả, ã, á, à, ặ, ẳ, ẵ, ắ, ằ\n"
        + "với các nguyên âm: a, ă, â, e, ê, i, o, ô, ơ, u, ư, y\n\n"
        + "Cần kiểm tra NFC normalization: ế (single codepoint) vs ế (decomposed)",
    },
    {
      title: "feat: Vietnamese diacritics in canonical [GitHub]",
      status: "open", priority: 3, issueType: "feature",
      labels: ["i18n", "vietnamese"],
      description: "GitHub: ensure NFC normalization before storage.",
    }
  );
  const d = interactiveReview([c], {}, makeInput(["b"]));
  eq(d, { "diac-1": "push" }, "变音符号: 决策正确");
}

// ====================== 16. 零宽字符和不可见 Unicode ======================
console.log("\n=== Invisible 16: 零宽/不可见字符 ===");
{
  const c = mkConflict("zw-1",
    {
      title: "fix: strip zero-width characters from input\u200B\u200C\u200D\uFEFF",
      status: "open", priority: 2, issueType: "bug",
      labels: ["sanitize", "unicode", "security"],
      description: "Input may contain invisible characters:\n"
        + "ZWSP (U+200B): [\u200B] Zero Width Space\n"
        + "ZWNJ (U+200C): [\u200C] Zero Width Non-Joiner\n"
        + "ZWJ  (U+200D): [\u200D] Zero Width Joiner\n"
        + "BOM  (U+FEFF): [\uFEFF] Byte Order Mark\n"
        + "WJ   (U+2060): [\u2060] Word Joiner\n\n"
        + "These should be stripped or normalized before storage.",
    },
    {
      title: "fix: zero-width chars [GitHub]",
      status: "open", priority: 2, issueType: "bug",
      labels: ["sanitize", "unicode"],
      description: "GitHub: also strip bidi override characters (U+202A-U+202E).",
    }
  );
  const d = interactiveReview([c], {}, makeInput(["b"]));
  eq(d, { "zw-1": "push" }, "零宽字符: 决策正确");
}

// ====================== 17. 自右向左的标签 ======================
console.log("\n=== RTL 17: RTL 标签名称 ===");
{
  const c = mkConflict("rtl-label-1",
    {
      title: "Labels in RTL scripts",
      status: "open", priority: 1, issueType: "task",
      labels: ["עִבְרִית", "العربية", "فارسی", "اردو", "ⵣⴰⵢⵔ"],
      description: "Testing RTL label rendering in the conflict panel.\n"
        + "Labels in Hebrew, Arabic, Persian, Urdu, and Tifinagh scripts.",
    },
    {
      title: "Labels in RTL scripts [GitHub]",
      status: "open", priority: 1, issueType: "task",
      labels: ["עִבְרִית", "العربية", "فارسی", "ⵣⴰⵢⵔ"],
      description: "GitHub labels: missing اردو label.",
    }
  );
  const d = interactiveReview([c], {}, makeInput(["b"]));
  eq(d, { "rtl-label-1": "push" }, "RTL 标签: 决策正确");
}

// ====================== 18. 多语言状态值 ======================
console.log("\n=== Status 18: 多语言状态值 ===");
{
  const languages = [
    { code: "zh", status: "进行中", title: "中文进行中" },
    { code: "ja", status: "進行中", title: "日本語進行中" },
    { code: "ko", status: "진행중", title: "한국어진행중" },
    { code: "ar", status: "قيد التنفيذ", title: "بالعربية" },
    { code: "he", status: "בתהליך", title: "בעברית" },
    { code: "ru", status: "в работе", title: "на русском" },
    { code: "de", status: "in Bearbeitung", title: "auf Deutsch" },
  ];

  const conf = languages.map((l) => mkConflict(`status-${l.code}`,
    {
      title: `${l.title} status display test`,
      status: "in_progress",
      priority: 2, issueType: "task",
      labels: [l.status, `lang-${l.code}`],
      description: `This issue tracks work that is "${l.status}" in the native language.\n`
        + `Language: ${l.code}. Status label uses native script.`,
      updatedAt: "2026-07-21T12:00:00Z",
    },
    {
      title: `${l.title} [GitHub]`,
      status: "open",
      priority: 2, issueType: "task",
      labels: [l.status, `lang-${l.code}`],
      description: `GitHub side: "${l.status}" (${l.code})`,
      updatedAt: "2026-07-21T13:00:00Z",
    }
  ));

  const d = interactiveReview(conf, {}, makeInput(Array(7).fill("a")));
  ok(Object.keys(d).length === 7, "多语言状态: 全部 7 个处理");
  ok(Object.values(d).every((v) => v === "push"), "多语言状态: 全部 push");
}

// ====================== 19. CJK 字符宽度混淆 ======================
console.log("\n=== CJK 19: CJK 字符宽度 ===");
{
  const c = mkConflict("cjk-width-1",
    {
      title: "fix: ＣＪＫ　全角英数字の正規化 (Fullwidth CJK normalization)",
      status: "open", priority: 2, issueType: "bug",
      labels: ["CJK", "normalization", "utf-8"],
      description: "全角英数字 (Fullwidth alphanumeric) の正規化について：\n"
        + "１２３４５ → 12345\n"
        + "ＡＢＣＤＥ → ABCDE\n"
        + "ａｂｃｄｅ → abcde\n"
        + "！＂＃＄％＆＇（）＊＋，－．／：；＜＝＞？＠［＼］＾＿｀｛｜｝～\n"
        + "　(U+3000 IDEOGRAPHIC SPACE) →  (U+0020 SPACE)",
    },
    {
      title: "fix: CJK fullwidth normalization [GitHub]",
      status: "open", priority: 2, issueType: "bug",
      labels: ["CJK", "normalization"],
      description: "GitHub: NFKC handles this. Add option to preserve intentional fullwidth.",
    }
  );
  const d = interactiveReview([c], {}, makeInput(["b"]));
  eq(d, { "cjk-width-1": "push" }, "CJK 宽度: 决策正确");
}

// ====================== Summary ======================
console.log(`\n=== I18N RESULTS: ${_pass} passed, ${_fail} failed ===`);
process.exit(_fail > 0 ? 1 : 0);
