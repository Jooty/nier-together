pub fn wall_limit_passed(since_ms: Option<u64>, now_ms: u64, limit_ms: u64) -> bool {
    since_ms.is_some_and(|since| now_ms.saturating_sub(since) >= limit_ms)
}
