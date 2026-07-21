//! 人类 / 非人类 actor 判定（AI、bot、service account）。
//!
//! 用于 approval registry 与 Authority Plane final-head review 评估：
//! AI/bot **不得**计入人类 approval 目标。

/// 归一化 GitHub handle（去 `@`、小写、trim）。
pub fn normalized_handle(handle: &str) -> String {
    handle.trim().trim_start_matches('@').to_ascii_lowercase()
}

/// 返回 true 时，该 handle **不能**作为人类 approver 计数。
///
/// 覆盖：
/// - 显式 `[bot]` / `-bot` / `_bot` 后缀
/// - 保留 token（ai/agent/copilot/claude/codex/…）
/// - GitHub user type `Bot`（经 `user_type` 参数）
pub fn is_non_human_handle(handle: &str) -> bool {
    is_non_human_actor(handle, None)
}

/// 结合 handle 与可选 GitHub `user.type`（`User` / `Bot` / `Organization`）。
pub fn is_non_human_actor(handle: &str, user_type: Option<&str>) -> bool {
    if let Some(ut) = user_type {
        let ut = ut.trim();
        if ut.eq_ignore_ascii_case("Bot") || ut.eq_ignore_ascii_case("Organization") {
            return true;
        }
    }

    let normalized = normalized_handle(handle);
    if normalized.is_empty() {
        return true;
    }

    let reserved_tokens = [
        "ai",
        "agent",
        "anthropic",
        "bot",
        "chatgpt",
        "claude",
        "codebuddy",
        "codex",
        "copilot",
        "dependabot",
        "gemini",
        "gpt",
        "grok",
        "llm",
        "machine",
        "openai",
        "opencode",
        "xai",
    ];

    if normalized.contains("[bot]")
        || normalized.ends_with("-bot")
        || normalized.ends_with("_bot")
        || normalized.starts_with("machine://")
    {
        return true;
    }

    normalized
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|token| !token.is_empty() && reserved_tokens.contains(&token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn humans_pass() {
        for h in ["alice", "ZoneCNH", "maintainer-one", "risk.owner"] {
            assert!(!is_non_human_handle(h), "{h}");
        }
    }

    #[test]
    fn bots_and_ai_fail() {
        for h in [
            "dependabot[bot]",
            "github-actions[bot]",
            "copilot",
            "claude-code",
            "codex-bot",
            "my_bot",
            "xai-grok",
            "openai-codex",
            "machine://xtask/approval-auto",
            "AI_AGENT",
        ] {
            assert!(is_non_human_handle(h), "{h}");
        }
    }

    #[test]
    fn user_type_bot_fails_even_with_human_login() {
        assert!(is_non_human_actor("alice", Some("Bot")));
        assert!(!is_non_human_actor("alice", Some("User")));
    }
}
