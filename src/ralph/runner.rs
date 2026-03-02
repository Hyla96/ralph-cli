pub enum RunnerEvent {
    Bytes(Vec<u8>),
    /// `None` means the process was killed; `Some(n)` is the natural exit code.
    Exited(Option<u32>),
    Complete,
    SpawnError(String),
    Resize(u16, u16),
    TokenUsage {
        input_tokens: u64,
        output_tokens: u64,
        cache_read_tokens: u64,
        cache_write_tokens: u64,
        cost_usd: f64,
    },
}
